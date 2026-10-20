// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Browser IndexedDB adapter for the validated offline-journal snapshot.
//!
//! Every write updates the operation snapshot and its indexes in one
//! readwrite transaction. A `DispatchPermit` is returned only after the
//! transaction's `complete` event. It contains the canonical bytes retained
//! in the entry, so retry code need not reconstruct a body from UI state.
//!
//! This module is deliberately a persistence boundary, not a transport and not
//! a claim of device-loss backup. IndexedDB durability is a user-agent hint;
//! storage eviction, user clearing, profile loss and hardware failure remain
//! possible and must be handled by the product's recovery UX.

use std::{
    cell::{Cell, RefCell},
    error::Error,
    fmt,
    rc::Rc,
};

use js_sys::{Array, Date, Function, Object, Promise, Reflect};
use sol_atlas_core::offline_journal::{
    JournalEntry, JournalEntrySnapshotV1, JournalError, JournalState, OperationEnvelope,
    ReconcileOutcome, ServiceReply,
};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;

/// Object-store names are part of database schema v1. This adapter uses
/// out-of-line keys so the ID is independently checked during recovery.
const DATABASE_NAME: &str = "sol-atlas-offline-journal";
const DATABASE_VERSION: u32 = 2;
const OPERATIONS: &str = "operations";
const IDEMPOTENCY_KEYS: &str = "idempotency_keys";
const PENDING_OPERATIONS: &str = "pending_operations";
const DISPATCH_LEASES: &str = "dispatch_leases";
const STORE_NAMES: [&str; 3] = [OPERATIONS, IDEMPOTENCY_KEYS, PENDING_OPERATIONS];
const UPGRADE_STORE_NAMES: [&str; 4] = [
    OPERATIONS,
    IDEMPOTENCY_KEYS,
    PENDING_OPERATIONS,
    DISPATCH_LEASES,
];
const DISPATCH_LEASE_SCHEMA_VERSION: u16 = 1;
const DISPATCH_LEASE_DURATION_MS: u64 = 30_000;

/// Whether a browser accepted strict transaction durability options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurabilityPolicy {
    StrictRequested,
    BrowserDefaultFallback,
}

/// Adapter-level errors. Errors do not imply that a write committed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedDbJournalError(pub String);

impl fmt::Display for IndexedDbJournalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for IndexedDbJournalError {}

impl From<JournalError> for IndexedDbJournalError {
    fn from(value: JournalError) -> Self {
        Self(format!("journal validation failed: {value}"))
    }
}

/// A dispatch token issued only after the pre-dispatch snapshot transaction
/// has committed. This token carries the exact canonical body to send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchPermit {
    operation_id: String,
    idempotency_key: String,
    payload_digest_sha256: String,
    canonical_payload_utf8: Vec<u8>,
    attempt_count: u32,
    lease_token: String,
    lease_expires_at_ms: u64,
    durability_policy: DurabilityPolicy,
}

impl DispatchPermit {
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }

    pub fn payload_digest_sha256(&self) -> &str {
        &self.payload_digest_sha256
    }

    pub fn canonical_payload_utf8(&self) -> &[u8] {
        &self.canonical_payload_utf8
    }

    pub fn attempt_count(&self) -> u32 {
        self.attempt_count
    }

    /// Opaque fencing token for this browser-local dispatch lease.
    pub fn lease_token(&self) -> &str {
        &self.lease_token
    }

    /// Epoch-millisecond expiry. Renew before expiry for long-running requests.
    pub fn lease_expires_at_ms(&self) -> u64 {
        self.lease_expires_at_ms
    }

    /// Durability policy used for the transaction that granted this permit.
    pub fn durability_policy(&self) -> DurabilityPolicy {
        self.durability_policy
    }
}

/// A bounded page of pending operations. A later recovery pass should start
/// with no cursor; this page is a view, not a snapshot-isolation guarantee.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedDbJournalPage {
    pub entries: Vec<JournalEntry>,
    pub next_after_operation_id: Option<String>,
}

/// Versioned, browser-local dispatch lease. The operation store and this
/// record are updated in the same transaction before a dispatch permit exists.
/// The lease reduces competing sends; the server's idempotency ledger remains
/// the only authority against duplicate effects after pauses or expiry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DispatchLeaseRecordV1 {
    schema_version: u16,
    operation_id: String,
    owner_token: String,
    acquired_at_ms: u64,
    expires_at_ms: u64,
}

impl DispatchLeaseRecordV1 {
    fn validate(&self, expected_operation_id: &str) -> Result<(), IndexedDbJournalError> {
        if self.schema_version != DISPATCH_LEASE_SCHEMA_VERSION {
            return Err(IndexedDbJournalError("unsupported dispatch lease schema".into()));
        }
        if self.operation_id != expected_operation_id {
            return Err(IndexedDbJournalError("dispatch lease operation ID mismatch".into()));
        }
        if self.owner_token.trim().is_empty() || self.expires_at_ms <= self.acquired_at_ms {
            return Err(IndexedDbJournalError("invalid dispatch lease record".into()));
        }
        Ok(())
    }

    fn is_active_at(&self, now_ms: u64) -> bool {
        self.expires_at_ms > now_ms
    }
}

/// Connected browser journal database.
#[derive(Clone)]
pub struct IndexedDbJournal {
    database: JsValue,
    last_durability_policy: Rc<Cell<DurabilityPolicy>>,
}

impl IndexedDbJournal {
    /// Opens schema v1, creating all stores in the same version-upgrade event.
    pub async fn open() -> Result<Self, IndexedDbJournalError> {
        let window = web_sys::window()
            .ok_or_else(|| IndexedDbJournalError("window is unavailable".into()))?;
        let factory =
            Reflect::get(window.as_ref(), &JsValue::from_str("indexedDB")).map_err(js_error)?;
        if factory.is_null() || factory.is_undefined() {
            return Err(IndexedDbJournalError(
                "IndexedDB is unavailable in this browser context".into(),
            ));
        }

        let request = call_method(
            &factory,
            "open",
            &[
                JsValue::from_str(DATABASE_NAME),
                JsValue::from_f64(DATABASE_VERSION as f64),
            ],
        )
        .map_err(js_error)?;

        let open_request = request.clone();
        let open_promise = Promise::new(&mut |resolve, reject| {
            let success_resolve = resolve.clone();
            let success_reject = reject.clone();
            let error_reject = reject.clone();
            let upgrade_request = open_request.clone();
            let upgrade = Closure::once_into_js(move |_event: JsValue| {
                let result = Reflect::get(&upgrade_request, &JsValue::from_str("result"));
                match result.and_then(|database| ensure_stores(&database).map(|_| database)) {
                    Ok(_) => {}
                    Err(error) => {
                        if let Ok(transaction) =
                            Reflect::get(&upgrade_request, &JsValue::from_str("transaction"))
                        {
                            let _ = call_method(&transaction, "abort", &[]);
                        }
                    }
                }
            });
            let _ = Reflect::set(
                &open_request,
                &JsValue::from_str("onupgradeneeded"),
                &upgrade,
            );

            let success_request = open_request.clone();
            let on_success = Closure::once_into_js(move |_event: JsValue| {
                match Reflect::get(&success_request, &JsValue::from_str("result")) {
                    Ok(database) => {
                        let _ = success_resolve.call1(&JsValue::UNDEFINED, &database);
                    }
                    Err(error) => {
                        let _ = success_reject.call1(&JsValue::UNDEFINED, &error);
                    }
                }
            });
            let _ = Reflect::set(&open_request, &JsValue::from_str("onsuccess"), &on_success);

            let error_request = open_request.clone();
            let on_error = Closure::once_into_js(move |_event: JsValue| {
                let error = Reflect::get(&error_request, &JsValue::from_str("error"))
                    .unwrap_or(JsValue::UNDEFINED);
                let _ = error_reject.call1(&JsValue::UNDEFINED, &error);
            });
            let _ = Reflect::set(&open_request, &JsValue::from_str("onerror"), &on_error);
        });

        let database = JsFuture::from(open_promise).await.map_err(js_error)?;
        Ok(Self {
            database,
            last_durability_policy: Rc::new(Cell::new(DurabilityPolicy::StrictRequested)),
        })
    }

    /// Last requested policy. A transaction may report a default fallback if
    /// the runtime does not support the strict durability option.
    pub fn last_transaction_durability_policy(&self) -> DurabilityPolicy {
        self.last_durability_policy.get()
    }

    fn transaction(
        &self,
        store_names: &[&str],
        mode: &str,
    ) -> Result<(JsValue, DurabilityPolicy), IndexedDbJournalError> {
        let (transaction, policy) = begin_transaction(&self.database, store_names, mode)?;
        self.last_durability_policy.set(policy);
        Ok((transaction, policy))
    }

    /// Atomically writes a new prepared snapshot and enforces operation-ID and
    /// idempotency-key uniqueness. Identical re-insertion is idempotent and
    /// returns the entry as currently stored; changed identity fails closed.
    pub async fn insert_prepared(
        &self,
        envelope: OperationEnvelope,
        payload_json: &str,
    ) -> Result<JournalEntry, IndexedDbJournalError> {
        let new_entry = JournalEntry::prepare(envelope, payload_json)?;
        let new_snapshot = new_entry.to_snapshot()?;
        let new_snapshot_json = serde_json::to_string(&new_snapshot).map_err(|error| {
            IndexedDbJournalError(format!("snapshot serialization failed: {error}"))
        })?;
        let operation_id = new_entry.envelope().operation_id.clone();
        let idempotency_key = new_entry.envelope().idempotency_key.clone();

        let (transaction, _policy) = self.transaction(&STORE_NAMES, "readwrite")?;
        let transaction_done = transaction_promise(transaction.clone());
        let operations = object_store(&transaction, OPERATIONS)?;
        let idempotency_keys = object_store(&transaction, IDEMPOTENCY_KEYS)?;
        let pending = object_store(&transaction, PENDING_OPERATIONS)?;

        let operation_request =
            call_method(&operations, "get", &[JsValue::from_str(&operation_id)])
                .map_err(js_error)?;
        let key_request = call_method(
            &idempotency_keys,
            "get",
            &[JsValue::from_str(&idempotency_key)],
        )
        .map_err(js_error)?;

        let shared = Rc::new(RefCell::new(InsertReadState::default()));
        let output: Rc<RefCell<Option<Result<Option<String>, IndexedDbJournalError>>>> =
            Rc::new(RefCell::new(None));

        {
            let request = operation_request.clone();
            let shared = shared.clone();
            let output = output.clone();
            let operations = operations.clone();
            let idempotency_keys = idempotency_keys.clone();
            let pending = pending.clone();
            let transaction = transaction.clone();
            let op_id = operation_id.clone();
            let idem_key = idempotency_key.clone();
            let new_snapshot = new_snapshot.clone();
            let new_snapshot_json = new_snapshot_json.clone();
            let payload_bytes = new_entry.canonical_payload_utf8().to_vec();
            let on_success = Closure::once_into_js(move |_event: JsValue| {
                let result = Reflect::get(&request, &JsValue::from_str("result"));
                match result {
                    Ok(value) => shared.borrow_mut().operation_read = Some(value),
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(js_error(error)));
                        let _ = call_method(&transaction, "abort", &[]);
                        return;
                    }
                }
                finish_insert_reads(
                    &shared,
                    &output,
                    &operations,
                    &idempotency_keys,
                    &pending,
                    &transaction,
                    &op_id,
                    &idem_key,
                    &new_snapshot,
                    &new_snapshot_json,
                    &payload_bytes,
                );
            });
            Reflect::set(
                &operation_request,
                &JsValue::from_str("onsuccess"),
                &on_success,
            )
            .map_err(js_error)?;
        }

        {
            let request = key_request.clone();
            let shared = shared.clone();
            let output = output.clone();
            let operations = operations.clone();
            let idempotency_keys = idempotency_keys.clone();
            let pending = pending.clone();
            let transaction = transaction.clone();
            let op_id = operation_id.clone();
            let idem_key = idempotency_key.clone();
            let new_snapshot = new_snapshot.clone();
            let new_snapshot_json = new_snapshot_json.clone();
            let payload_bytes = new_entry.canonical_payload_utf8().to_vec();
            let on_success = Closure::once_into_js(move |_event: JsValue| {
                let result = Reflect::get(&request, &JsValue::from_str("result"));
                match result {
                    Ok(value) => shared.borrow_mut().idempotency_read = Some(value),
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(js_error(error)));
                        let _ = call_method(&transaction, "abort", &[]);
                        return;
                    }
                }
                finish_insert_reads(
                    &shared,
                    &output,
                    &operations,
                    &idempotency_keys,
                    &pending,
                    &transaction,
                    &op_id,
                    &idem_key,
                    &new_snapshot,
                    &new_snapshot_json,
                    &payload_bytes,
                );
            });
            Reflect::set(&key_request, &JsValue::from_str("onsuccess"), &on_success)
                .map_err(js_error)?;
        }

        JsFuture::from(transaction_done).await.map_err(js_error)?;

        match output.borrow_mut().take() {
            Some(Ok(Some(existing_snapshot))) => {
                let snapshot: JournalEntrySnapshotV1 = serde_json::from_str(&existing_snapshot)
                    .map_err(|error| {
                        IndexedDbJournalError(format!("stored snapshot JSON invalid: {error}"))
                    })?;
                JournalEntry::from_snapshot_for_operation(&operation_id, snapshot)
                    .map_err(IndexedDbJournalError::from)
            }
            Some(Ok(None)) => Ok(new_entry),
            Some(Err(error)) => Err(error),
            None => Err(IndexedDbJournalError(
                "insert transaction completed without an outcome".into(),
            )),
        }
    }

    /// Reads an operation and validates both its snapshot and its database key.
    pub async fn load(
        &self,
        operation_id: &str,
    ) -> Result<Option<JournalEntry>, IndexedDbJournalError> {
        let (transaction, _policy) = self.transaction(&[OPERATIONS], "readonly")?;
        let transaction_done = transaction_promise(transaction.clone());
        let operations = object_store(&transaction, OPERATIONS)?;
        let request = call_method(&operations, "get", &[JsValue::from_str(operation_id)])
            .map_err(js_error)?;
        let result = JsFuture::from(request_promise(request))
            .await
            .map_err(js_error)?;
        JsFuture::from(transaction_done).await.map_err(js_error)?;

        if result.is_undefined() || result.is_null() {
            return Ok(None);
        }
        let snapshot_json = result.as_string().ok_or_else(|| {
            IndexedDbJournalError("operation store contains a non-string snapshot".into())
        })?;
        let snapshot: JournalEntrySnapshotV1 =
            serde_json::from_str(&snapshot_json).map_err(|error| {
                IndexedDbJournalError(format!("stored snapshot JSON invalid: {error}"))
            })?;
        JournalEntry::from_snapshot_for_operation(operation_id, snapshot)
            .map(Some)
            .map_err(IndexedDbJournalError::from)
    }

    /// Persists the OutcomeUnknown transition, pending index, attempt count and
    /// canonical body together. Returns no DispatchPermit until transaction
    /// completion has been observed by this future.
    /// Atomically claims a cross-tab lease and persists OutcomeUnknown before
    /// returning permission to send. A second tab sees the active lease and
    /// receives an error rather than a second permit.
    ///
    /// Lease expiry is a duplicate-suppression mechanism, not an exactly-once
    /// guarantee: a suspended tab can resume after expiry. Server-side idempotency
    /// must still bind each key to the same immutable request fingerprint.
    pub async fn begin_attempt(
        &self,
        operation_id: &str,
    ) -> Result<DispatchPermit, IndexedDbJournalError> {
        let lease_token = secure_random_token()?;
        let store_names = [OPERATIONS, PENDING_OPERATIONS, DISPATCH_LEASES];
        let (transaction, durability_policy) = self.transaction(&store_names, "readwrite")?;
        let transaction_done = transaction_promise(transaction.clone());
        let operations = object_store(&transaction, OPERATIONS)?;
        let pending = object_store(&transaction, PENDING_OPERATIONS)?;
        let leases = object_store(&transaction, DISPATCH_LEASES)?;
        let request = call_method(&operations, "get", &[JsValue::from_str(operation_id)])
            .map_err(js_error)?;
        let output: Rc<
            RefCell<Option<Result<(String, DispatchLeaseRecordV1), IndexedDbJournalError>>>,
        > = Rc::new(RefCell::new(None));

        {
            let request_for_callback = request.clone();
            let output = output.clone();
            let operations = operations.clone();
            let pending = pending.clone();
            let leases = leases.clone();
            let transaction = transaction.clone();
            let expected_id = operation_id.to_owned();
            let lease_token = lease_token.clone();
            let callback = Closure::once_into_js(move |_event: JsValue| {
                let raw = match Reflect::get(&request_for_callback, &JsValue::from_str("result")) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(js_error(error)));
                        return;
                    }
                };
                if raw.is_null() || raw.is_undefined() {
                    *output.borrow_mut() = Some(Err(IndexedDbJournalError(format!(
                        "operation {expected_id} was not found"
                    ))));
                    return;
                }
                let snapshot_json = match raw.as_string() {
                    Some(value) => value,
                    None => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError(
                            "operation store contains a non-string snapshot".into(),
                        )));
                        return;
                    }
                };
                let snapshot: JournalEntrySnapshotV1 = match serde_json::from_str(&snapshot_json) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError(format!(
                            "stored snapshot JSON invalid: {error}"
                        ))));
                        return;
                    }
                };
                let entry =
                    match JournalEntry::from_snapshot_for_operation(&expected_id, snapshot) {
                        Ok(value) => value,
                        Err(error) => {
                            *output.borrow_mut() = Some(Err(IndexedDbJournalError::from(error)));
                            return;
                        }
                    };

                // Chain the second read inside the first request's success event,
                // keeping the transaction active without awaiting a Promise.
                let lease_request = match call_method(
                    &leases,
                    "get",
                    &[JsValue::from_str(&expected_id)],
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(js_error(error)));
                        let _ = call_method(&transaction, "abort", &[]);
                        return;
                    }
                };
                let lease_request_for_callback = lease_request.clone();
                let output = output.clone();
                let operations = operations.clone();
                let pending = pending.clone();
                let leases = leases.clone();
                let transaction = transaction.clone();
                let expected_id = expected_id.clone();
                let lease_token = lease_token.clone();
                let callback = Closure::once_into_js(move |_lease_event: JsValue| {
                    let raw_lease = match Reflect::get(
                        &lease_request_for_callback,
                        &JsValue::from_str("result"),
                    ) {
                        Ok(value) => value,
                        Err(error) => {
                            *output.borrow_mut() = Some(Err(js_error(error)));
                            let _ = call_method(&transaction, "abort", &[]);
                            return;
                        }
                    };

                    let existing_lease = if raw_lease.is_null() || raw_lease.is_undefined() {
                        None
                    } else {
                        let serialized = match raw_lease.as_string() {
                            Some(value) => value,
                            None => {
                                *output.borrow_mut() = Some(Err(IndexedDbJournalError(
                                    "dispatch lease store contains a non-string record".into(),
                                )));
                                let _ = call_method(&transaction, "abort", &[]);
                                return;
                            }
                        };
                        let record: DispatchLeaseRecordV1 =
                            match serde_json::from_str(&serialized) {
                                Ok(value) => value,
                                Err(error) => {
                                    *output.borrow_mut() = Some(Err(IndexedDbJournalError(
                                        format!("stored dispatch lease JSON invalid: {error}"),
                                    )));
                                    let _ = call_method(&transaction, "abort", &[]);
                                    return;
                                }
                            };
                        if let Err(error) = record.validate(&expected_id) {
                            *output.borrow_mut() = Some(Err(error));
                            let _ = call_method(&transaction, "abort", &[]);
                            return;
                        }
                        Some(record)
                    };

                    let now_ms = match epoch_now_ms() {
                        Ok(value) => value,
                        Err(error) => {
                            *output.borrow_mut() = Some(Err(error));
                            let _ = call_method(&transaction, "abort", &[]);
                            return;
                        }
                    };
                    if let Some(existing) = existing_lease.as_ref() {
                        if existing.is_active_at(now_ms) {
                            *output.borrow_mut() = Some(Err(IndexedDbJournalError(format!(
                                "dispatch lease is held until epoch-ms {}",
                                existing.expires_at_ms
                            ))));
                            return;
                        }
                    }

                    let lease = DispatchLeaseRecordV1 {
                        schema_version: DISPATCH_LEASE_SCHEMA_VERSION,
                        operation_id: expected_id.clone(),
                        owner_token: lease_token,
                        acquired_at_ms: now_ms,
                        expires_at_ms: now_ms.saturating_add(DISPATCH_LEASE_DURATION_MS),
                    };
                    let lease_json = match serde_json::to_string(&lease) {
                        Ok(value) => value,
                        Err(error) => {
                            *output.borrow_mut() = Some(Err(IndexedDbJournalError(format!(
                                "dispatch lease serialization failed: {error}"
                            ))));
                            let _ = call_method(&transaction, "abort", &[]);
                            return;
                        }
                    };

                    let mut entry = entry;
                    if let Err(error) = entry.mark_attempt_started() {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError::from(error)));
                        return;
                    }
                    let saved = match entry.to_snapshot().and_then(|value| {
                        serde_json::to_string(&value)
                            .map_err(|_| JournalError::InvalidSnapshot("snapshot_serialization_failed"))
                    }) {
                        Ok(value) => value,
                        Err(error) => {
                            *output.borrow_mut() = Some(Err(IndexedDbJournalError::from(error)));
                            return;
                        }
                    };

                    // Both writes plus the pending marker commit or abort as a unit.
                    for (store, value) in [
                        (&leases, lease_json),
                        (&operations, saved.clone()),
                    ] {
                        if let Err(error) = call_method(
                            store,
                            "put",
                            &[JsValue::from_str(&value), JsValue::from_str(&expected_id)],
                        ) {
                            *output.borrow_mut() = Some(Err(js_error(error)));
                            let _ = call_method(&transaction, "abort", &[]);
                            return;
                        }
                    }
                    if let Err(error) = call_method(
                        &pending,
                        "put",
                        &[JsValue::from_bool(true), JsValue::from_str(&expected_id)],
                    ) {
                        *output.borrow_mut() = Some(Err(js_error(error)));
                        let _ = call_method(&transaction, "abort", &[]);
                        return;
                    }
                    *output.borrow_mut() = Some(Ok((saved, lease)));
                });
                if let Err(error) = Reflect::set(
                    &lease_request,
                    &JsValue::from_str("onsuccess"),
                    &callback,
                ) {
                    *output.borrow_mut() = Some(Err(js_error(error)));
                    let _ = call_method(&transaction, "abort", &[]);
                }
            });
            Reflect::set(&request, &JsValue::from_str("onsuccess"), &callback)
                .map_err(js_error)?;
        }

        JsFuture::from(transaction_done).await.map_err(js_error)?;
        let (saved, lease) = output.borrow_mut().take().ok_or_else(|| {
            IndexedDbJournalError("attempt transaction completed without an outcome".into())
        })??;
        let snapshot: JournalEntrySnapshotV1 = serde_json::from_str(&saved).map_err(|error| {
            IndexedDbJournalError(format!("stored snapshot JSON invalid: {error}"))
        })?;
        let entry = JournalEntry::from_snapshot_for_operation(operation_id, snapshot)
            .map_err(IndexedDbJournalError::from)?;
        Ok(DispatchPermit {
            operation_id: entry.envelope().operation_id.clone(),
            idempotency_key: entry.envelope().idempotency_key.clone(),
            payload_digest_sha256: entry.envelope().payload_digest_sha256.clone(),
            canonical_payload_utf8: entry.canonical_payload_utf8().to_vec(),
            attempt_count: entry.attempt_count(),
            lease_token: lease.owner_token,
            lease_expires_at_ms: lease.expires_at_ms,
            durability_policy,
        })
    }

    /// Applies a matching reply and its pending-index change in one transaction.
    /// Rejected/misbound replies leave the stored snapshot and indexes untouched.
    pub async fn apply_reply(
        &self,
        operation_id: &str,
        reply: &ServiceReply,
    ) -> Result<ReconcileOutcome, IndexedDbJournalError> {
        let store_names = [OPERATIONS, PENDING_OPERATIONS];
        let (transaction, _policy) = self.transaction(&store_names, "readwrite")?;
        let transaction_done = transaction_promise(transaction.clone());
        let operations = object_store(&transaction, OPERATIONS)?;
        let pending = object_store(&transaction, PENDING_OPERATIONS)?;
        let request = call_method(&operations, "get", &[JsValue::from_str(operation_id)])
            .map_err(js_error)?;
        let output: Rc<RefCell<Option<Result<ReconcileOutcome, IndexedDbJournalError>>>> =
            Rc::new(RefCell::new(None));

        {
            let request_for_callback = request.clone();
            let output = output.clone();
            let operations = operations.clone();
            let pending = pending.clone();
            let transaction = transaction.clone();
            let op_id = operation_id.to_owned();
            let reply = reply.clone();
            let callback = Closure::once_into_js(move |_event: JsValue| {
                let raw = match Reflect::get(&request_for_callback, &JsValue::from_str("result")) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(js_error(error)));
                        return;
                    }
                };
                if raw.is_null() || raw.is_undefined() {
                    *output.borrow_mut() = Some(Err(IndexedDbJournalError(format!(
                        "operation {op_id} was not found"
                    ))));
                    return;
                }
                let snapshot_json = match raw.as_string() {
                    Some(value) => value,
                    None => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError(
                            "operation store contains a non-string snapshot".into(),
                        )));
                        return;
                    }
                };
                let snapshot: JournalEntrySnapshotV1 = match serde_json::from_str(&snapshot_json) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError(format!(
                            "stored snapshot JSON invalid: {error}"
                        ))));
                        return;
                    }
                };
                let mut entry = match JournalEntry::from_snapshot_for_operation(&op_id, snapshot) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError::from(error)));
                        return;
                    }
                };
                let outcome = match entry.reconcile(&reply) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError::from(error)));
                        return;
                    }
                };
                let new_snapshot = match entry.to_snapshot() {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError::from(error)));
                        return;
                    }
                };
                let serialized = match serde_json::to_string(&new_snapshot) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError(format!(
                            "snapshot serialization failed: {error}"
                        ))));
                        return;
                    }
                };
                if let Err(error) = call_method(
                    &operations,
                    "put",
                    &[JsValue::from_str(&serialized), JsValue::from_str(&op_id)],
                ) {
                    *output.borrow_mut() = Some(Err(js_error(error)));
                    let _ = call_method(&transaction, "abort", &[]);
                    return;
                }
                if is_pending(entry.state()) {
                    if let Err(error) = call_method(
                        &pending,
                        "put",
                        &[JsValue::from_bool(true), JsValue::from_str(&op_id)],
                    ) {
                        *output.borrow_mut() = Some(Err(js_error(error)));
                        let _ = call_method(&transaction, "abort", &[]);
                        return;
                    }
                } else if let Err(error) =
                    call_method(&pending, "delete", &[JsValue::from_str(&op_id)])
                {
                    *output.borrow_mut() = Some(Err(js_error(error)));
                    let _ = call_method(&transaction, "abort", &[]);
                    return;
                }
                *output.borrow_mut() = Some(Ok(outcome));
            });
            Reflect::set(&request, &JsValue::from_str("onsuccess"), &callback).map_err(js_error)?;
        }

        JsFuture::from(transaction_done).await.map_err(js_error)?;
        output.borrow_mut().take().ok_or_else(|| {
            IndexedDbJournalError("reply transaction completed without an outcome".into())
        })?
    }

    /// Lists a bounded pending page. The pending index is deliberately only a
    /// scheduling hint; every row is reloaded and validated before being returned.
    pub async fn list_pending(
        &self,
        after_operation_id: Option<&str>,
        limit: usize,
    ) -> Result<IndexedDbJournalPage, IndexedDbJournalError> {
        if limit == 0 {
            return Err(IndexedDbJournalError(
                "page size must be greater than zero".into(),
            ));
        }
        let (transaction, _policy) = self.transaction(&[PENDING_OPERATIONS], "readonly")?;
        let transaction_done = transaction_promise(transaction.clone());
        let pending = object_store(&transaction, PENDING_OPERATIONS)?;
        let request = call_method(&pending, "getAllKeys", &[]).map_err(js_error)?;
        let values = JsFuture::from(request_promise(request))
            .await
            .map_err(js_error)?;
        JsFuture::from(transaction_done).await.map_err(js_error)?;

        let mut ids = Array::from(&values)
            .iter()
            .filter_map(|value| value.as_string())
            .filter(|id| after_operation_id.map_or(true, |after| id.as_str() > after))
            .collect::<Vec<_>>();
        ids.sort();
        let has_more = ids.len() > limit;
        ids.truncate(limit);
        let mut entries = Vec::with_capacity(ids.len());
        for id in ids.iter() {
            let entry = self.load(id).await?.ok_or_else(|| {
                IndexedDbJournalError(format!("pending index points to missing operation {id}"))
            })?;
            if !is_pending(entry.state()) {
                return Err(IndexedDbJournalError(format!(
                    "pending index points to non-pending operation {id}"
                )));
            }
            entries.push(entry);
        }
        Ok(IndexedDbJournalPage {
            entries,
            next_after_operation_id: if has_more { ids.last().cloned() } else { None },
        })
    }
}

#[derive(Default)]
struct InsertReadState {
    operation_read: Option<JsValue>,
    idempotency_read: Option<JsValue>,
    decided: bool,
}

fn finish_insert_reads(
    shared: &Rc<RefCell<InsertReadState>>,
    output: &Rc<RefCell<Option<Result<Option<String>, IndexedDbJournalError>>>>,
    operations: &JsValue,
    idempotency_keys: &JsValue,
    pending: &JsValue,
    transaction: &JsValue,
    operation_id: &str,
    idempotency_key: &str,
    new_snapshot: &JournalEntrySnapshotV1,
    new_snapshot_json: &str,
    new_payload_bytes: &[u8],
) {
    let (existing, existing_idempotency, ready) = {
        let mut state = shared.borrow_mut();
        if state.decided || state.operation_read.is_none() || state.idempotency_read.is_none() {
            return;
        }
        state.decided = true;
        (
            state.operation_read.clone().unwrap_or(JsValue::UNDEFINED),
            state.idempotency_read.clone().unwrap_or(JsValue::UNDEFINED),
            true,
        )
    };
    if !ready {
        return;
    }

    let result = if !existing.is_undefined() && !existing.is_null() {
        match existing.as_string() {
            None => Err(IndexedDbJournalError(
                "operation store contains a non-string snapshot".into(),
            )),
            Some(serialized) => {
                let snapshot: Result<JournalEntrySnapshotV1, _> = serde_json::from_str(&serialized);
                match snapshot {
                    Err(error) => Err(IndexedDbJournalError(format!(
                        "stored snapshot JSON invalid: {error}"
                    ))),
                    Ok(snapshot) => match JournalEntry::from_snapshot_for_operation(
                        operation_id,
                        snapshot.clone(),
                    ) {
                        Err(error) => Err(IndexedDbJournalError::from(error)),
                        Ok(existing_entry) => {
                            let same_envelope = snapshot.envelope == new_snapshot.envelope;
                            let same_payload =
                                existing_entry.canonical_payload_utf8() == new_payload_bytes;
                            if !same_envelope || !same_payload {
                                Err(IndexedDbJournalError(
                                    "operation ID is already bound to a different immutable envelope/payload".into(),
                                ))
                            } else if existing_idempotency.as_string().as_deref()
                                != Some(operation_id)
                            {
                                Err(IndexedDbJournalError(
                                    "idempotency index is missing or points to another operation"
                                        .into(),
                                ))
                            } else {
                                Ok(Some(serialized))
                            }
                        }
                    },
                }
            }
        }
    } else if !existing_idempotency.is_undefined() && !existing_idempotency.is_null() {
        Err(IndexedDbJournalError(format!(
            "idempotency key {idempotency_key} is already bound to another operation"
        )))
    } else {
        let operations_add = call_method(
            operations,
            "add",
            &[
                JsValue::from_str(new_snapshot_json),
                JsValue::from_str(operation_id),
            ],
        );
        let key_add = call_method(
            idempotency_keys,
            "add",
            &[
                JsValue::from_str(operation_id),
                JsValue::from_str(idempotency_key),
            ],
        );
        let pending_add = call_method(
            pending,
            "add",
            &[JsValue::from_bool(true), JsValue::from_str(operation_id)],
        );
        match (operations_add, key_add, pending_add) {
            (Ok(_), Ok(_), Ok(_)) => Ok(None),
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
                let _ = call_method(transaction, "abort", &[]);
                Err(js_error(error))
            }
        }
    };
    *output.borrow_mut() = Some(result);
}

fn is_pending(state: &JournalState) -> bool {
    matches!(state, JournalState::Prepared | JournalState::OutcomeUnknown)
}

fn ensure_stores(database: &JsValue) -> Result<(), JsValue> {
    let names = Reflect::get(database, &JsValue::from_str("objectStoreNames"))?;
    for name in UPGRADE_STORE_NAMES {
        let exists = call_method(&names, "contains", &[JsValue::from_str(name)])?
            .as_bool()
            .unwrap_or(false);
        if !exists {
            call_method(database, "createObjectStore", &[JsValue::from_str(name)])?;
        }
    }
    Ok(())
}

fn begin_transaction(
    database: &JsValue,
    store_names: &[&str],
    mode: &str,
) -> Result<(JsValue, DurabilityPolicy), IndexedDbJournalError> {
    let names = Array::new();
    for name in store_names {
        names.push(&JsValue::from_str(name));
    }
    let options = Object::new();
    Reflect::set(
        &options,
        &JsValue::from_str("durability"),
        &JsValue::from_str("strict"),
    )
    .map_err(js_error)?;
    match call_method(
        database,
        "transaction",
        &[
            names.clone().into(),
            JsValue::from_str(mode),
            options.into(),
        ],
    ) {
        Ok(transaction) => Ok((transaction, DurabilityPolicy::StrictRequested)),
        Err(error) if js_error_name(&error).as_deref() == Some("TypeError") => call_method(
            database,
            "transaction",
            &[names.into(), JsValue::from_str(mode)],
        )
        .map(|transaction| (transaction, DurabilityPolicy::BrowserDefaultFallback))
        .map_err(js_error),
        Err(error) => Err(js_error(error)),
    }
}

fn object_store(transaction: &JsValue, name: &str) -> Result<JsValue, IndexedDbJournalError> {
    call_method(transaction, "objectStore", &[JsValue::from_str(name)]).map_err(js_error)
}

fn call_method(receiver: &JsValue, name: &str, arguments: &[JsValue]) -> Result<JsValue, JsValue> {
    let method = Reflect::get(receiver, &JsValue::from_str(name))?.dyn_into::<Function>()?;
    let array = Array::new();
    for argument in arguments {
        array.push(argument);
    }
    method.apply(receiver, &array)
}

fn request_promise(request: JsValue) -> Promise {
    Promise::new(&mut |resolve, reject| {
        let success_resolve = resolve.clone();
        let success_reject = reject.clone();
        let error_reject = reject.clone();
        let success_request = request.clone();
        let on_success = Closure::once_into_js(move |_event: JsValue| {
            match Reflect::get(&success_request, &JsValue::from_str("result")) {
                Ok(value) => {
                    let _ = success_resolve.call1(&JsValue::UNDEFINED, &value);
                }
                Err(error) => {
                    let _ = success_reject.call1(&JsValue::UNDEFINED, &error);
                }
            }
        });
        let _ = Reflect::set(&request, &JsValue::from_str("onsuccess"), &on_success);

        let error_request = request.clone();
        let on_error = Closure::once_into_js(move |_event: JsValue| {
            let error = Reflect::get(&error_request, &JsValue::from_str("error"))
                .unwrap_or(JsValue::UNDEFINED);
            let _ = error_reject.call1(&JsValue::UNDEFINED, &error);
        });
        let _ = Reflect::set(&request, &JsValue::from_str("onerror"), &on_error);
    })
}

fn transaction_promise(transaction: JsValue) -> Promise {
    Promise::new(&mut |resolve, reject| {
        let completion_resolve = resolve.clone();
        let abort_reject = reject.clone();
        let on_complete = Closure::once_into_js(move |_event: JsValue| {
            let _ = completion_resolve.call1(&JsValue::UNDEFINED, &JsValue::UNDEFINED);
        });
        let _ = Reflect::set(&transaction, &JsValue::from_str("oncomplete"), &on_complete);

        let failed_transaction = transaction.clone();
        let on_abort = Closure::once_into_js(move |_event: JsValue| {
            let error = Reflect::get(&failed_transaction, &JsValue::from_str("error"))
                .unwrap_or(JsValue::UNDEFINED);
            let _ = abort_reject.call1(&JsValue::UNDEFINED, &error);
        });
        let _ = Reflect::set(&transaction, &JsValue::from_str("onabort"), &on_abort);
    })
}

fn js_error(value: JsValue) -> IndexedDbJournalError {
    let message = value.as_string().unwrap_or_else(|| {
        Reflect::get(&value, &JsValue::from_str("message"))
            .ok()
            .and_then(|message| message.as_string())
            .unwrap_or_else(|| format!("JavaScript IndexedDB failure: {value:?}"))
    });
    IndexedDbJournalError(message)
}

fn js_error_name(value: &JsValue) -> Option<String> {
    Reflect::get(value, &JsValue::from_str("name"))
        .ok()
        .and_then(|name| name.as_string())
}
