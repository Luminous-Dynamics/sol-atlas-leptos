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
use sol_atlas_core::offline_journal_bundle::{
    OfflineJournalBundleError, OfflineJournalBundleRecordV1, OfflineJournalBundleV1,
};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;

/// Object-store names are part of database schema v1. This adapter uses
/// out-of-line keys so the ID is independently checked during recovery.
const DATABASE_NAME: &str = "sol-atlas-offline-journal";
const DATABASE_VERSION: u32 = 4;
const OPERATIONS: &str = "operations";
const IDEMPOTENCY_KEYS: &str = "idempotency_keys";
const PENDING_OPERATIONS: &str = "pending_operations";
const DISPATCH_LEASES: &str = "dispatch_leases";
const STAGED_IMPORTS: &str = "staged_imports";
const STAGED_IMPORT_IDEMPOTENCY_KEYS: &str = "staged_import_idempotency_keys";
const STORE_NAMES: [&str; 3] = [OPERATIONS, IDEMPOTENCY_KEYS, PENDING_OPERATIONS];
const UPGRADE_STORE_NAMES: [&str; 6] = [
    OPERATIONS,
    IDEMPOTENCY_KEYS,
    PENDING_OPERATIONS,
    DISPATCH_LEASES,
    STAGED_IMPORTS,
    STAGED_IMPORT_IDEMPOTENCY_KEYS,
];
const DISPATCH_LEASE_SCHEMA_VERSION: u16 = 1;
const STORED_OPERATION_SCHEMA_VERSION: u16 = 1;
const STAGED_IMPORT_SCHEMA_VERSION: u16 = 1;
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
    owner_scope: String,
    operation_namespace: String,
    request_fingerprint_sha256: String,
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

    /// Immutable account/tenant scope the operation was created under.
    pub fn owner_scope(&self) -> &str {
        &self.owner_scope
    }

    /// Immutable operation namespace used for dispatch and fingerprinting.
    pub fn operation_namespace(&self) -> &str {
        &self.operation_namespace
    }

    /// Canonical fingerprint over scope, namespace and immutable envelope bindings.
    pub fn request_fingerprint_sha256(&self) -> &str {
        &self.request_fingerprint_sha256
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

/// Result of staging a portable bundle for authoritative reconciliation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BundleStageReport {
    /// New records written to the isolated staging store, never the dispatch queue.
    pub staged_records: usize,
    /// Records that exactly matched an existing staged record.
    pub exact_duplicates: usize,
    /// Records already present in the active journal with matching immutable content.
    pub already_present_records: usize,
}

/// A staged snapshot which has not yet been reconciled with the authoritative service.
///
/// The type carries no dispatch permit and does not make the contained state an
/// authenticated result. Callers must display it as untrusted imported material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedJournalOperation {
    pub owner_scope: String,
    pub operation_namespace: String,
    pub source_bundle_sha256: String,
    pub snapshot: JournalEntrySnapshotV1,
}

/// Immutable owner/namespace binding around the journal snapshot.
///
/// This record is stored under the operation ID and is revalidated before
/// every read used for dispatch. Legacy snapshot-only rows have no trustworthy
/// owner scope and are intentionally not eligible for automatic dispatch.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredOperationV1 {
    schema_version: u16,
    owner_scope: String,
    operation_namespace: String,
    request_fingerprint_schema_version: u16,
    request_fingerprint_sha256: String,
    snapshot: JournalEntrySnapshotV1,
}

impl StoredOperationV1 {
    fn validate(&self) -> Result<(), IndexedDbJournalError> {
        if self.schema_version != STORED_OPERATION_SCHEMA_VERSION {
            return Err(IndexedDbJournalError(
                "unsupported stored operation record schema".into(),
            ));
        }
        if self.owner_scope.trim().is_empty() {
            return Err(IndexedDbJournalError(
                "stored operation has an empty owner scope".into(),
            ));
        }
        if self.operation_namespace.trim().is_empty() {
            return Err(IndexedDbJournalError(
                "stored operation has an empty namespace".into(),
            ));
        }
        if self.request_fingerprint_schema_version
            != sol_atlas_core::offline_idempotency_fingerprint::REQUEST_FINGERPRINT_SCHEMA_VERSION
        {
            return Err(IndexedDbJournalError(
                "unsupported stored request fingerprint schema".into(),
            ));
        }
        if !is_sha256_lower_hex(&self.request_fingerprint_sha256) {
            return Err(IndexedDbJournalError(
                "stored request fingerprint must be 64 lowercase hexadecimal characters".into(),
            ));
        }
        Ok(())
    }

    fn matches_scope(&self, owner_scope: &str, operation_namespace: &str) -> bool {
        self.owner_scope == owner_scope && self.operation_namespace == operation_namespace
    }

    /// Rebuilds the immutable entry and verifies the stored request fingerprint
    /// over the exact canonical payload bytes. A changed scope, body, envelope,
    /// or fingerprint must fail closed before dispatch or reply reconciliation.
    fn validate_entry(&self, operation_id: &str) -> Result<JournalEntry, IndexedDbJournalError> {
        self.validate()?;
        let entry = JournalEntry::from_snapshot_for_operation(operation_id, self.snapshot.clone())
            .map_err(IndexedDbJournalError::from)?;
        let canonical_payload =
            std::str::from_utf8(entry.canonical_payload_utf8()).map_err(|error| {
                IndexedDbJournalError(format!("canonical payload is not UTF-8: {error}"))
            })?;
        let actual =
            sol_atlas_core::offline_idempotency_fingerprint::canonical_request_fingerprint_sha256(
                &self.owner_scope,
                &self.operation_namespace,
                entry.envelope(),
                canonical_payload,
            )
            .map_err(|error| IndexedDbJournalError(error.to_string()))?;
        if actual != self.request_fingerprint_sha256 {
            return Err(IndexedDbJournalError(
                "stored request fingerprint does not match immutable operation content".into(),
            ));
        }
        Ok(entry)
    }
}

/// Untrusted imported operation held outside the active journal and dispatch indexes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StagedImportRecordV1 {
    schema_version: u16,
    source_bundle_sha256: String,
    operation: StoredOperationV1,
}

impl StagedImportRecordV1 {
    fn validate(&self, expected_operation_id: &str) -> Result<JournalEntry, IndexedDbJournalError> {
        if self.schema_version != STAGED_IMPORT_SCHEMA_VERSION {
            return Err(IndexedDbJournalError(
                "unsupported staged import record schema".into(),
            ));
        }
        if !is_sha256_lower_hex(&self.source_bundle_sha256) {
            return Err(IndexedDbJournalError(
                "staged import source bundle digest is invalid".into(),
            ));
        }
        self.operation.validate_entry(expected_operation_id)
    }

    fn matches_scope(&self, owner_scope: &str, operation_namespace: &str) -> bool {
        self.operation.matches_scope(owner_scope, operation_namespace)
    }

    fn to_public(&self) -> StagedJournalOperation {
        StagedJournalOperation {
            owner_scope: self.operation.owner_scope.clone(),
            operation_namespace: self.operation.operation_namespace.clone(),
            source_bundle_sha256: self.source_bundle_sha256.clone(),
            snapshot: self.operation.snapshot.clone(),
        }
    }
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
            return Err(IndexedDbJournalError(
                "unsupported dispatch lease schema".into(),
            ));
        }
        if self.operation_id != expected_operation_id {
            return Err(IndexedDbJournalError(
                "dispatch lease operation ID mismatch".into(),
            ));
        }
        if self.owner_token.trim().is_empty() || self.expires_at_ms <= self.acquired_at_ms {
            return Err(IndexedDbJournalError(
                "invalid dispatch lease record".into(),
            ));
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
    /// Atomically imports a validated portable bundle for the authenticated owner.
    ///
    /// All operation IDs, scoped idempotency keys and pending markers are
    /// preflighted in the same readwrite transaction that performs inserts.
    /// Exact duplicates are no-ops; any immutable-state conflict aborts the
    /// entire transaction. Existing operation snapshots are never overwritten,
    /// so importing a stale bundle cannot roll a locally progressed row back.
    /// Dispatch leases are deliberately not imported.
    /// Stages a validated bundle outside the active journal.
    ///
    /// Bundles have an unkeyed integrity digest, so their state/receipt fields
    /// are not proof that an operation was or was not applied by the service.
    /// This method therefore never writes imported rows to the live operation,
    /// idempotency or pending stores, and never makes them dispatchable. Records
    /// must be reconciled against the authoritative service before promotion.
    pub async fn stage_bundle(
        &self,
        bundle: &OfflineJournalBundleV1,
        authenticated_owner_scope: &str,
    ) -> Result<BundleStageReport, IndexedDbJournalError> {
        bundle
            .validate_for_owner(authenticated_owner_scope)
            .map_err(|error| IndexedDbJournalError(error.to_string()))?;
        if bundle.records.is_empty() {
            return Ok(BundleStageReport::default());
        }

        let mut seen_scoped_keys = std::collections::HashSet::with_capacity(bundle.records.len());
        let mut candidates = Vec::with_capacity(bundle.records.len());
        for record in &bundle.records {
            let operation_id = record.operation_id().to_owned();
            let entry = JournalEntry::from_snapshot_for_operation(
                &operation_id,
                record.snapshot.clone(),
            )
            .map_err(IndexedDbJournalError::from)?;
            let canonical_payload =
                std::str::from_utf8(entry.canonical_payload_utf8()).map_err(|error| {
                    IndexedDbJournalError(format!("canonical payload is not UTF-8: {error}"))
                })?;
            let request_fingerprint_sha256 =
                sol_atlas_core::offline_idempotency_fingerprint::canonical_request_fingerprint_sha256(
                    authenticated_owner_scope,
                    &record.operation_namespace,
                    entry.envelope(),
                    canonical_payload,
                )
                .map_err(|error| IndexedDbJournalError(error.to_string()))?;
            let stored = StoredOperationV1 {
                schema_version: STORED_OPERATION_SCHEMA_VERSION,
                owner_scope: authenticated_owner_scope.to_owned(),
                operation_namespace: record.operation_namespace.clone(),
                request_fingerprint_schema_version:
                    sol_atlas_core::offline_idempotency_fingerprint::REQUEST_FINGERPRINT_SCHEMA_VERSION,
                request_fingerprint_sha256,
                snapshot: record.snapshot.clone(),
            };
            stored.validate_entry(&operation_id)?;
            let idempotency_storage_key = scoped_idempotency_key(
                authenticated_owner_scope,
                &record.operation_namespace,
                &entry.envelope().idempotency_key,
            )?;
            if !seen_scoped_keys.insert(idempotency_storage_key.clone()) {
                return Err(IndexedDbJournalError(format!(
                    "bundle reuses scoped idempotency key for operation {operation_id}"
                )));
            }
            let staged = StagedImportRecordV1 {
                schema_version: STAGED_IMPORT_SCHEMA_VERSION,
                source_bundle_sha256: bundle.integrity_sha256.clone(),
                operation: stored,
            };
            staged.validate(&operation_id)?;
            let serialized_staged = serde_json::to_string(&staged).map_err(|error| {
                IndexedDbJournalError(format!("staged record serialization failed: {error}"))
            })?;
            candidates.push(BundleStageCandidate {
                operation_id,
                idempotency_storage_key,
                staged,
                serialized_staged,
            });
        }

        let store_names = [
            OPERATIONS,
            IDEMPOTENCY_KEYS,
            STAGED_IMPORTS,
            STAGED_IMPORT_IDEMPOTENCY_KEYS,
        ];
        let (transaction, _policy) = self.transaction(&store_names, "readwrite")?;
        let transaction_done = transaction_promise(transaction.clone());
        let operations = object_store(&transaction, OPERATIONS)?;
        let idempotency_keys = object_store(&transaction, IDEMPOTENCY_KEYS)?;
        let staged_imports = object_store(&transaction, STAGED_IMPORTS)?;
        let staged_idempotency_keys = object_store(&transaction, STAGED_IMPORT_IDEMPOTENCY_KEYS)?;

        let mut requests = Vec::with_capacity(candidates.len() * 4);
        for (index, candidate) in candidates.iter().enumerate() {
            requests.push((
                call_method(
                    &operations,
                    "get",
                    &[JsValue::from_str(&candidate.operation_id)],
                )
                .map_err(js_error)?,
                index * 4,
            ));
            requests.push((
                call_method(
                    &idempotency_keys,
                    "get",
                    &[JsValue::from_str(&candidate.idempotency_storage_key)],
                )
                .map_err(js_error)?,
                index * 4 + 1,
            ));
            requests.push((
                call_method(
                    &staged_imports,
                    "get",
                    &[JsValue::from_str(&candidate.operation_id)],
                )
                .map_err(js_error)?,
                index * 4 + 2,
            ));
            requests.push((
                call_method(
                    &staged_idempotency_keys,
                    "get",
                    &[JsValue::from_str(&candidate.idempotency_storage_key)],
                )
                .map_err(js_error)?,
                index * 4 + 3,
            ));
        }

        let shared = Rc::new(RefCell::new(BundleStageReadState {
            candidates,
            results: vec![None; requests.len()],
            completed: 0,
            decided: false,
        }));
        let output: Rc<RefCell<Option<Result<BundleStageReport, IndexedDbJournalError>>>> =
            Rc::new(RefCell::new(None));

        for (request, result_index) in requests {
            let request_for_callback = request.clone();
            let shared = shared.clone();
            let output = output.clone();
            let operations = operations.clone();
            let idempotency_keys = idempotency_keys.clone();
            let staged_imports = staged_imports.clone();
            let staged_idempotency_keys = staged_idempotency_keys.clone();
            let transaction = transaction.clone();
            let callback = Closure::once_into_js(move |_event: JsValue| {
                let value = match Reflect::get(
                    &request_for_callback,
                    &JsValue::from_str("result"),
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(js_error(error)));
                        let _ = call_method(&transaction, "abort", &[]);
                        return;
                    }
                };
                let all_reads_completed = {
                    let mut state = shared.borrow_mut();
                    if state.decided {
                        return;
                    }
                    state.results[result_index] = Some(value);
                    state.completed += 1;
                    if state.completed == state.results.len() {
                        state.decided = true;
                        true
                    } else {
                        false
                    }
                };
                if all_reads_completed {
                    finish_bundle_stage_reads(
                        &shared,
                        &output,
                        &operations,
                        &idempotency_keys,
                        &staged_imports,
                        &staged_idempotency_keys,
                        &transaction,
                    );
                }
            });
            if let Err(error) = Reflect::set(
                &request,
                &JsValue::from_str("onsuccess"),
                &callback,
            ) {
                let _ = call_method(&transaction, "abort", &[]);
                return Err(js_error(error));
            }
        }

        let transaction_result = JsFuture::from(transaction_done).await;
        match transaction_result {
            Ok(_) => output.borrow_mut().take().ok_or_else(|| {
                IndexedDbJournalError(
                    "bundle staging transaction completed without an outcome".into(),
                )
            })?,
            Err(transaction_error) => match output.borrow_mut().take() {
                Some(Err(preflight_error)) => Err(preflight_error),
                _ => Err(js_error(transaction_error)),
            },
        }
    }

    /// Lists untrusted, owner-scoped staged records for an explicit reconciliation UI.
    /// It does not add the records to the dispatch queue or assert their outcomes.
    pub async fn list_staged_imports(
        &self,
        owner_scope: &str,
    ) -> Result<Vec<StagedJournalOperation>, IndexedDbJournalError> {
        if owner_scope.trim().is_empty() {
            return Err(IndexedDbJournalError(
                "owner scope must not be empty for staged-import listing".into(),
            ));
        }
        let (transaction, _policy) = self.transaction(&[STAGED_IMPORTS], "readonly")?;
        let transaction_done = transaction_promise(transaction.clone());
        let staged = object_store(&transaction, STAGED_IMPORTS)?;
        let keys_request = call_method(&staged, "getAllKeys", &[]).map_err(js_error)?;
        let values_request = call_method(&staged, "getAll", &[]).map_err(js_error)?;
        let keys = JsFuture::from(request_promise(keys_request))
            .await
            .map_err(js_error)?;
        let values = JsFuture::from(request_promise(values_request))
            .await
            .map_err(js_error)?;
        JsFuture::from(transaction_done).await.map_err(js_error)?;
        let keys = Array::from(&keys);
        let values = Array::from(&values);
        if keys.length() != values.length() {
            return Err(IndexedDbJournalError(
                "staged-import keys and values do not align".into(),
            ));
        }

        let mut records = Vec::new();
        for index in 0..keys.length() {
            let operation_id = keys.get(index).as_string().ok_or_else(|| {
                IndexedDbJournalError("staged-import store contains a non-string key".into())
            })?;
            let serialized = values.get(index).as_string().ok_or_else(|| {
                IndexedDbJournalError("staged-import store contains a non-string record".into())
            })?;
            let staged: StagedImportRecordV1 = serde_json::from_str(&serialized).map_err(|error| {
                IndexedDbJournalError(format!("staged record {operation_id} is invalid: {error}"))
            })?;
            staged.validate(&operation_id)?;
            if staged.operation.owner_scope == owner_scope {
                records.push(staged.to_public());
            }
        }
        records.sort_by(|left, right| {
            left.snapshot
                .envelope
                .operation_id
                .cmp(&right.snapshot.envelope.operation_id)
        });
        Ok(records)
    }

    /// Exports a consistent, validated read snapshot for one owner scope.
    ///
    /// Both operation keys and values are read through the same readonly
    /// transaction. Records for other owners are omitted, but every stored row
    /// is parsed and validated before scope filtering so corrupt legacy data
    /// fails closed instead of silently disappearing from an export.
    ///
    /// This method only creates an integrity-checked bundle; it does not
    /// encrypt the returned JSON or authenticate its source.
    pub async fn export_bundle(
        &self,
        owner_scope: &str,
    ) -> Result<OfflineJournalBundleV1, IndexedDbJournalError> {
        if owner_scope.trim().is_empty() {
            return Err(IndexedDbJournalError(
                "owner scope must not be empty for journal export".into(),
            ));
        }
        let (transaction, _policy) = self.transaction(&[OPERATIONS], "readonly")?;
        let transaction_done = transaction_promise(transaction.clone());
        let operations = object_store(&transaction, OPERATIONS)?;
        let keys_request = call_method(&operations, "getAllKeys", &[]).map_err(js_error)?;
        let values_request = call_method(&operations, "getAll", &[]).map_err(js_error)?;

        let keys = JsFuture::from(request_promise(keys_request))
            .await
            .map_err(js_error)?;
        let values = JsFuture::from(request_promise(values_request))
            .await
            .map_err(js_error)?;
        JsFuture::from(transaction_done).await.map_err(js_error)?;

        let keys = Array::from(&keys);
        let values = Array::from(&values);
        if keys.length() != values.length() {
            return Err(IndexedDbJournalError(
                "IndexedDB getAllKeys/getAll result lengths do not match".into(),
            ));
        }

        let mut records = Vec::new();
        for index in 0..keys.length() {
            let operation_id = keys.get(index).as_string().ok_or_else(|| {
                IndexedDbJournalError("operation store contains a non-string key".into())
            })?;
            let serialized = values.get(index).as_string().ok_or_else(|| {
                IndexedDbJournalError(
                    "operation store contains a non-string owner-bound record".into(),
                )
            })?;
            let stored: StoredOperationV1 = serde_json::from_str(&serialized).map_err(|error| {
                IndexedDbJournalError(format!(
                    "stored operation {operation_id} is not a valid owner-bound record: {error}"
                ))
            })?;
            stored.validate_entry(&operation_id)?;

            if stored.owner_scope == owner_scope {
                records.push(OfflineJournalBundleRecordV1 {
                    operation_namespace: stored.operation_namespace,
                    snapshot: stored.snapshot,
                });
            }
        }

        OfflineJournalBundleV1::create(owner_scope, records)
            .map_err(|error: OfflineJournalBundleError| IndexedDbJournalError(error.to_string()))
    }

    pub async fn insert_prepared(
        &self,
        owner_scope: &str,
        operation_namespace: &str,
        envelope: OperationEnvelope,
        payload_json: &str,
    ) -> Result<JournalEntry, IndexedDbJournalError> {
        if owner_scope.trim().is_empty() || operation_namespace.trim().is_empty() {
            return Err(IndexedDbJournalError(
                "owner scope and operation namespace must not be empty".into(),
            ));
        }
        let new_entry = JournalEntry::prepare(envelope, payload_json)?;
        let new_snapshot = new_entry.to_snapshot()?;
        let canonical_payload =
            std::str::from_utf8(new_entry.canonical_payload_utf8()).map_err(|error| {
                IndexedDbJournalError(format!("canonical payload is not UTF-8: {error}"))
            })?;
        let request_fingerprint_sha256 =
            sol_atlas_core::offline_idempotency_fingerprint::canonical_request_fingerprint_sha256(
                owner_scope,
                operation_namespace,
                new_entry.envelope(),
                canonical_payload,
            )
            .map_err(|error| IndexedDbJournalError(error.to_string()))?;
        let new_record = StoredOperationV1 {
            schema_version: STORED_OPERATION_SCHEMA_VERSION,
            owner_scope: owner_scope.to_owned(),
            operation_namespace: operation_namespace.to_owned(),
            request_fingerprint_schema_version:
                sol_atlas_core::offline_idempotency_fingerprint::REQUEST_FINGERPRINT_SCHEMA_VERSION,
            request_fingerprint_sha256,
            snapshot: new_snapshot,
        };
        new_record.validate()?;
        let new_record_json = serde_json::to_string(&new_record).map_err(|error| {
            IndexedDbJournalError(format!("stored operation serialization failed: {error}"))
        })?;
        let operation_id = new_entry.envelope().operation_id.clone();
        let idempotency_key = scoped_idempotency_key(
            owner_scope,
            operation_namespace,
            &new_entry.envelope().idempotency_key,
        )?;

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
            let new_record = new_record.clone();
            let new_record_json = new_record_json.clone();
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
                    &new_record,
                    &new_record_json,
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
            let new_record = new_record.clone();
            let new_record_json = new_record_json.clone();
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
                    &new_record,
                    &new_record_json,
                    &payload_bytes,
                );
            });
            Reflect::set(&key_request, &JsValue::from_str("onsuccess"), &on_success)
                .map_err(js_error)?;
        }

        JsFuture::from(transaction_done).await.map_err(js_error)?;

        match output.borrow_mut().take() {
            Some(Ok(Some(existing_record_json))) => {
                let existing: StoredOperationV1 = serde_json::from_str(&existing_record_json)
                    .map_err(|error| {
                        IndexedDbJournalError(format!("stored operation JSON invalid: {error}"))
                    })?;
                if !existing.matches_scope(owner_scope, operation_namespace) {
                    return Err(IndexedDbJournalError(
                        "operation ID is already bound to another owner scope or namespace".into(),
                    ));
                }
                existing.validate_entry(&operation_id)
            }
            Some(Ok(None)) => Ok(new_entry),
            Some(Err(error)) => Err(error),
            None => Err(IndexedDbJournalError(
                "insert transaction completed without an outcome".into(),
            )),
        }
    }

    /// Reads and validates an operation record without revealing its owner scope.
    async fn read_stored_operation(
        &self,
        operation_id: &str,
    ) -> Result<Option<StoredOperationV1>, IndexedDbJournalError> {
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
        let stored_json = result.as_string().ok_or_else(|| {
            IndexedDbJournalError("operation store contains a non-string stored record".into())
        })?;
        let stored: StoredOperationV1 = serde_json::from_str(&stored_json).map_err(|error| {
            IndexedDbJournalError(format!(
                "stored operation is not an owner-bound v1 record; automatic dispatch is blocked: {error}"
            ))
        })?;
        stored.validate_entry(operation_id)?;
        Ok(Some(stored))
    }

    /// Reads an operation only when its immutable owner and namespace match the
    /// caller's current authenticated/account context. A mismatch appears absent
    /// rather than revealing another owner's stored record.
    pub async fn load(
        &self,
        operation_id: &str,
        owner_scope: &str,
        operation_namespace: &str,
    ) -> Result<Option<JournalEntry>, IndexedDbJournalError> {
        require_scope(owner_scope, operation_namespace)?;
        let Some(stored) = self.read_stored_operation(operation_id).await? else {
            return Ok(None);
        };
        if !stored.matches_scope(owner_scope, operation_namespace) {
            return Ok(None);
        }
        JournalEntry::from_snapshot_for_operation(operation_id, stored.snapshot)
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
        owner_scope: &str,
        operation_namespace: &str,
    ) -> Result<DispatchPermit, IndexedDbJournalError> {
        require_scope(owner_scope, operation_namespace)?;
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
            let expected_owner_scope = owner_scope.to_owned();
            let expected_namespace = operation_namespace.to_owned();
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
                let stored_json = match raw.as_string() {
                    Some(value) => value,
                    None => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError(
                            "operation store contains a non-string owner-bound record".into(),
                        )));
                        return;
                    }
                };
                let stored: StoredOperationV1 = match serde_json::from_str(&stored_json) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError(format!(
                            "stored operation is not an owner-bound v1 record: {error}"
                        ))));
                        return;
                    }
                };
                if let Err(error) = stored.validate() {
                    *output.borrow_mut() = Some(Err(error));
                    return;
                }
                if !stored.matches_scope(&expected_owner_scope, &expected_namespace) {
                    *output.borrow_mut() = Some(Err(IndexedDbJournalError(
                        "operation is bound to a different owner scope or namespace".into(),
                    )));
                    return;
                }
                let entry = match stored.validate_entry(&expected_id) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(error));
                        return;
                    }
                };

                // Chain the second read inside the first request's success event,
                // keeping the transaction active without awaiting a Promise.
                let lease_request =
                    match call_method(&leases, "get", &[JsValue::from_str(&expected_id)]) {
                        Ok(value) => value,
                        Err(error) => {
                            *output.borrow_mut() = Some(Err(js_error(error)));
                            let _ = call_method(&transaction, "abort", &[]);
                            return;
                        }
                    };
                let lease_request_for_callback = lease_request.clone();
                let stored_record = stored;
                let output_for_registration_error = output.clone();
                let transaction_for_registration_error = transaction.clone();
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
                        let record: DispatchLeaseRecordV1 = match serde_json::from_str(&serialized)
                        {
                            Ok(value) => value,
                            Err(error) => {
                                *output.borrow_mut() = Some(Err(IndexedDbJournalError(format!(
                                    "stored dispatch lease JSON invalid: {error}"
                                ))));
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
                    let mut stored_record = stored_record;
                    stored_record.snapshot = match entry.to_snapshot() {
                        Ok(value) => value,
                        Err(error) => {
                            *output.borrow_mut() = Some(Err(IndexedDbJournalError::from(error)));
                            return;
                        }
                    };
                    let saved = match serde_json::to_string(&stored_record) {
                        Ok(value) => value,
                        Err(error) => {
                            *output.borrow_mut() = Some(Err(IndexedDbJournalError(format!(
                                "stored operation serialization failed: {error}"
                            ))));
                            return;
                        }
                    };

                    // Both writes plus the pending marker commit or abort as a unit.
                    for (store, value) in [(&leases, lease_json), (&operations, saved.clone())] {
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
                if let Err(error) =
                    Reflect::set(&lease_request, &JsValue::from_str("onsuccess"), &callback)
                {
                    *output_for_registration_error.borrow_mut() = Some(Err(js_error(error)));
                    let _ = call_method(&transaction_for_registration_error, "abort", &[]);
                }
            });
            Reflect::set(&request, &JsValue::from_str("onsuccess"), &callback).map_err(js_error)?;
        }

        JsFuture::from(transaction_done).await.map_err(js_error)?;
        let (saved, lease) = output.borrow_mut().take().ok_or_else(|| {
            IndexedDbJournalError("attempt transaction completed without an outcome".into())
        })??;
        let stored: StoredOperationV1 = serde_json::from_str(&saved).map_err(|error| {
            IndexedDbJournalError(format!("stored operation JSON invalid: {error}"))
        })?;
        if !stored.matches_scope(owner_scope, operation_namespace) {
            return Err(IndexedDbJournalError(
                "operation owner scope changed before dispatch permit creation".into(),
            ));
        }
        let request_fingerprint_sha256 = stored.request_fingerprint_sha256.clone();
        let entry = stored.validate_entry(operation_id)?;
        Ok(DispatchPermit {
            operation_id: entry.envelope().operation_id.clone(),
            idempotency_key: entry.envelope().idempotency_key.clone(),
            payload_digest_sha256: entry.envelope().payload_digest_sha256.clone(),
            canonical_payload_utf8: entry.canonical_payload_utf8().to_vec(),
            attempt_count: entry.attempt_count(),
            owner_scope: owner_scope.to_owned(),
            operation_namespace: operation_namespace.to_owned(),
            request_fingerprint_sha256,
            lease_token: lease.owner_token,
            lease_expires_at_ms: lease.expires_at_ms,
            durability_policy,
        })
    }

    /// Renew the current permit's lease without incrementing the attempt count.
    ///
    /// This is a compare-and-set operation. A lease that has expired or whose
    /// owner token changed cannot be revived by a stale or suspended tab.
    pub async fn renew_dispatch_lease(
        &self,
        permit: &mut DispatchPermit,
    ) -> Result<(), IndexedDbJournalError> {
        if self
            .load(
                &permit.operation_id,
                &permit.owner_scope,
                &permit.operation_namespace,
            )
            .await?
            .is_none()
        {
            return Err(IndexedDbJournalError(
                "operation owner scope changed; refusing dispatch lease renewal".into(),
            ));
        }
        let (transaction, durability_policy) = self.transaction(&[DISPATCH_LEASES], "readwrite")?;
        let transaction_done = transaction_promise(transaction.clone());
        let leases = object_store(&transaction, DISPATCH_LEASES)?;
        let request = call_method(&leases, "get", &[JsValue::from_str(&permit.operation_id)])
            .map_err(js_error)?;
        let output: Rc<RefCell<Option<Result<u64, IndexedDbJournalError>>>> =
            Rc::new(RefCell::new(None));

        {
            let request_for_callback = request.clone();
            let output = output.clone();
            let leases = leases.clone();
            let transaction = transaction.clone();
            let operation_id = permit.operation_id.clone();
            let owner_token = permit.lease_token.clone();
            let callback = Closure::once_into_js(move |_event: JsValue| {
                let raw = match Reflect::get(&request_for_callback, &JsValue::from_str("result")) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(js_error(error)));
                        return;
                    }
                };
                if raw.is_null() || raw.is_undefined() {
                    *output.borrow_mut() = Some(Err(IndexedDbJournalError(
                        "dispatch lease no longer exists".into(),
                    )));
                    return;
                }
                let serialized = match raw.as_string() {
                    Some(value) => value,
                    None => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError(
                            "dispatch lease record is not a string".into(),
                        )));
                        return;
                    }
                };
                let mut lease: DispatchLeaseRecordV1 = match serde_json::from_str(&serialized) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError(format!(
                            "stored dispatch lease JSON invalid: {error}"
                        ))));
                        let _ = call_method(&transaction, "abort", &[]);
                        return;
                    }
                };
                if let Err(error) = lease.validate(&operation_id) {
                    *output.borrow_mut() = Some(Err(error));
                    let _ = call_method(&transaction, "abort", &[]);
                    return;
                }
                let now_ms = match epoch_now_ms() {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(error));
                        let _ = call_method(&transaction, "abort", &[]);
                        return;
                    }
                };
                if lease.owner_token != owner_token || !lease.is_active_at(now_ms) {
                    *output.borrow_mut() = Some(Err(IndexedDbJournalError(
                        "dispatch lease ownership lost; reacquire before sending".into(),
                    )));
                    return;
                }
                lease.expires_at_ms = now_ms.saturating_add(DISPATCH_LEASE_DURATION_MS);
                let new_expiry = lease.expires_at_ms;
                let encoded = match serde_json::to_string(&lease) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError(format!(
                            "dispatch lease serialization failed: {error}"
                        ))));
                        let _ = call_method(&transaction, "abort", &[]);
                        return;
                    }
                };
                if let Err(error) = call_method(
                    &leases,
                    "put",
                    &[
                        JsValue::from_str(&encoded),
                        JsValue::from_str(&operation_id),
                    ],
                ) {
                    *output.borrow_mut() = Some(Err(js_error(error)));
                    let _ = call_method(&transaction, "abort", &[]);
                    return;
                }
                *output.borrow_mut() = Some(Ok(new_expiry));
            });
            Reflect::set(&request, &JsValue::from_str("onsuccess"), &callback).map_err(js_error)?;
        }

        JsFuture::from(transaction_done).await.map_err(js_error)?;
        permit.lease_expires_at_ms = output.borrow_mut().take().ok_or_else(|| {
            IndexedDbJournalError("lease renewal completed without an outcome".into())
        })??;
        permit.durability_policy = durability_policy;
        Ok(())
    }

    /// Applies a matching reply and its pending-index change in one transaction.
    /// Rejected/misbound replies leave the stored snapshot and indexes untouched.
    pub async fn apply_reply(
        &self,
        operation_id: &str,
        owner_scope: &str,
        operation_namespace: &str,
        reply: &ServiceReply,
    ) -> Result<ReconcileOutcome, IndexedDbJournalError> {
        require_scope(owner_scope, operation_namespace)?;
        let store_names = [OPERATIONS, PENDING_OPERATIONS, DISPATCH_LEASES];
        let (transaction, _policy) = self.transaction(&store_names, "readwrite")?;
        let transaction_done = transaction_promise(transaction.clone());
        let operations = object_store(&transaction, OPERATIONS)?;
        let pending = object_store(&transaction, PENDING_OPERATIONS)?;
        let leases = object_store(&transaction, DISPATCH_LEASES)?;
        let request = call_method(&operations, "get", &[JsValue::from_str(operation_id)])
            .map_err(js_error)?;
        let output: Rc<RefCell<Option<Result<ReconcileOutcome, IndexedDbJournalError>>>> =
            Rc::new(RefCell::new(None));

        {
            let request_for_callback = request.clone();
            let output = output.clone();
            let operations = operations.clone();
            let pending = pending.clone();
            let leases = leases.clone();
            let transaction = transaction.clone();
            let op_id = operation_id.to_owned();
            let expected_owner_scope = owner_scope.to_owned();
            let expected_namespace = operation_namespace.to_owned();
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
                let stored_json = match raw.as_string() {
                    Some(value) => value,
                    None => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError(
                            "operation store contains a non-string owner-bound record".into(),
                        )));
                        return;
                    }
                };
                let mut stored: StoredOperationV1 = match serde_json::from_str(&stored_json) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError(format!(
                            "stored operation JSON invalid: {error}"
                        ))));
                        return;
                    }
                };
                if let Err(error) = stored.validate() {
                    *output.borrow_mut() = Some(Err(error));
                    return;
                }
                if !stored.matches_scope(&expected_owner_scope, &expected_namespace) {
                    *output.borrow_mut() = Some(Err(IndexedDbJournalError(
                        "operation is bound to a different owner scope or namespace".into(),
                    )));
                    return;
                }
                let mut entry = match stored.validate_entry(&op_id) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(error));
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
                stored.snapshot = match entry.to_snapshot() {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError::from(error)));
                        return;
                    }
                };
                let serialized = match serde_json::to_string(&stored) {
                    Ok(value) => value,
                    Err(error) => {
                        *output.borrow_mut() = Some(Err(IndexedDbJournalError(format!(
                            "stored operation serialization failed: {error}"
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
                if let Err(error) = call_method(&leases, "delete", &[JsValue::from_str(&op_id)]) {
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
        owner_scope: &str,
        operation_namespace: &str,
        after_operation_id: Option<&str>,
        limit: usize,
    ) -> Result<IndexedDbJournalPage, IndexedDbJournalError> {
        require_scope(owner_scope, operation_namespace)?;
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

        // The pending index is shared by all account scopes. Scan by operation ID
        // but only return records bound to this authenticated owner + namespace.
        // An empty page may still return a cursor when the scanned IDs belonged
        // to other scopes; callers continue until the cursor is None.
        let mut entries = Vec::with_capacity(ids.len());
        for id in ids.iter() {
            let stored = self.read_stored_operation(id).await?.ok_or_else(|| {
                IndexedDbJournalError(format!("pending index points to missing operation {id}"))
            })?;
            if !stored.matches_scope(owner_scope, operation_namespace) {
                continue;
            }
            let entry = JournalEntry::from_snapshot_for_operation(id, stored.snapshot)
                .map_err(IndexedDbJournalError::from)?;
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

#[derive(Debug, Clone)]
struct BundleStageCandidate {
    operation_id: String,
    idempotency_storage_key: String,
    staged: StagedImportRecordV1,
    serialized_staged: String,
}

struct BundleStageReadState {
    candidates: Vec<BundleStageCandidate>,
    results: Vec<Option<JsValue>>,
    completed: usize,
    decided: bool,
}

fn same_immutable_operation(left: &StoredOperationV1, right: &StoredOperationV1) -> bool {
    left.owner_scope == right.owner_scope
        && left.operation_namespace == right.operation_namespace
        && left.snapshot.envelope == right.snapshot.envelope
        && left.snapshot.canonical_payload_json == right.snapshot.canonical_payload_json
}

fn finish_bundle_stage_reads(
    shared: &Rc<RefCell<BundleStageReadState>>,
    output: &Rc<RefCell<Option<Result<BundleStageReport, IndexedDbJournalError>>>>,
    operations: &JsValue,
    idempotency_keys: &JsValue,
    staged_imports: &JsValue,
    staged_idempotency_keys: &JsValue,
    transaction: &JsValue,
) {
    let (candidates, results) = {
        let state = shared.borrow();
        (
            state.candidates.clone(),
            state
                .results
                .iter()
                .map(|value| value.clone().unwrap_or(JsValue::UNDEFINED))
                .collect::<Vec<_>>(),
        )
    };

    let mut inserts = Vec::new();
    let mut stale_staging_to_remove = Vec::<(String, String)>::new();
    let mut report = BundleStageReport::default();

    let preflight = (|| -> Result<(), IndexedDbJournalError> {
        for (index, candidate) in candidates.iter().enumerate() {
            let offset = index * 4;
            let live_operation = &results[offset];
            let live_key = &results[offset + 1];
            let staged_operation = &results[offset + 2];
            let staged_key = &results[offset + 3];
            let live_exists = !live_operation.is_null() && !live_operation.is_undefined();
            let live_key_exists = !live_key.is_null() && !live_key.is_undefined();
            let staged_exists = !staged_operation.is_null() && !staged_operation.is_undefined();
            let staged_key_exists = !staged_key.is_null() && !staged_key.is_undefined();

            if live_exists {
                let serialized = live_operation.as_string().ok_or_else(|| {
                    IndexedDbJournalError(format!(
                        "active operation {} is not a string record",
                        candidate.operation_id
                    ))
                })?;
                let live: StoredOperationV1 =
                    serde_json::from_str(&serialized).map_err(|error| {
                        IndexedDbJournalError(format!(
                            "active operation {} is invalid: {error}",
                            candidate.operation_id
                        ))
                    })?;
                live.validate_entry(&candidate.operation_id)?;
                if !same_immutable_operation(&live, &candidate.staged.operation) {
                    return Err(IndexedDbJournalError(format!(
                        "operation {} is already active with different immutable content or owner scope",
                        candidate.operation_id
                    )));
                }
                if !live_key_exists
                    || live_key.as_string().as_deref()
                        != Some(candidate.operation_id.as_str())
                {
                    return Err(IndexedDbJournalError(format!(
                        "active operation {} has a missing or conflicting scoped idempotency index",
                        candidate.operation_id
                    )));
                }

                if staged_exists {
                    let serialized_staged = staged_operation.as_string().ok_or_else(|| {
                        IndexedDbJournalError(format!(
                            "staged operation {} is not a string record",
                            candidate.operation_id
                        ))
                    })?;
                    let staged: StagedImportRecordV1 =
                        serde_json::from_str(&serialized_staged).map_err(|error| {
                            IndexedDbJournalError(format!(
                                "staged operation {} is invalid: {error}",
                                candidate.operation_id
                            ))
                        })?;
                    staged.validate(&candidate.operation_id)?;
                    if !staged.matches_scope(
                        &candidate.staged.operation.owner_scope,
                        &candidate.staged.operation.operation_namespace,
                    ) || !same_immutable_operation(&staged.operation, &live)
                    {
                        return Err(IndexedDbJournalError(format!(
                            "active and staged operation {} have conflicting immutable identity",
                            candidate.operation_id
                        )));
                    }
                    if staged.operation.snapshot != live.snapshot {
                        return Err(IndexedDbJournalError(format!(
                            "active and staged operation {} have different state/attempt lineage; keep staged evidence for explicit review",
                            candidate.operation_id
                        )));
                    }
                    if !staged_key_exists
                        || staged_key.as_string().as_deref()
                            != Some(candidate.operation_id.as_str())
                    {
                        return Err(IndexedDbJournalError(format!(
                            "staged operation {} has a missing or conflicting scoped idempotency index",
                            candidate.operation_id
                        )));
                    }
                    stale_staging_to_remove.push((
                        candidate.operation_id.clone(),
                        candidate.idempotency_storage_key.clone(),
                    ));
                } else if staged_key_exists {
                    return Err(IndexedDbJournalError(format!(
                        "staged idempotency index for {} has no staged operation",
                        candidate.operation_id
                    )));
                }
                report.already_present_records += 1;
                continue;
            }

            if live_key_exists {
                return Err(IndexedDbJournalError(format!(
                    "bundle operation {} conflicts with an active or stale idempotency index",
                    candidate.operation_id
                )));
            }

            if staged_exists {
                let serialized = staged_operation.as_string().ok_or_else(|| {
                    IndexedDbJournalError(format!(
                        "staged operation {} is not a string record",
                        candidate.operation_id
                    ))
                })?;
                let existing: StagedImportRecordV1 =
                    serde_json::from_str(&serialized).map_err(|error| {
                        IndexedDbJournalError(format!(
                            "staged operation {} is invalid: {error}",
                            candidate.operation_id
                        ))
                    })?;
                existing.validate(&candidate.operation_id)?;
                if !existing.matches_scope(
                    &candidate.staged.operation.owner_scope,
                    &candidate.staged.operation.operation_namespace,
                ) {
                    return Err(IndexedDbJournalError(format!(
                        "staged operation {} belongs to another owner or namespace",
                        candidate.operation_id
                    )));
                }
                if existing.operation != candidate.staged.operation {
                    return Err(IndexedDbJournalError(format!(
                        "staged operation {} differs from this bundle; refusing to replace untrusted staged evidence",
                        candidate.operation_id
                    )));
                }
                if !staged_key_exists
                    || staged_key.as_string().as_deref()
                        != Some(candidate.operation_id.as_str())
                {
                    return Err(IndexedDbJournalError(format!(
                        "staged operation {} has a missing or conflicting scoped idempotency index",
                        candidate.operation_id
                    )));
                }
                report.exact_duplicates += 1;
                continue;
            }

            if staged_key_exists {
                return Err(IndexedDbJournalError(format!(
                    "staged idempotency index for {} points to a missing operation",
                    candidate.operation_id
                )));
            }
            inserts.push(index);
            report.staged_records += 1;
        }
        Ok(())
    })();

    if let Err(error) = preflight {
        *output.borrow_mut() = Some(Err(error));
        let _ = call_method(transaction, "abort", &[]);
        return;
    }

    for index in inserts {
        let candidate = &candidates[index];
        if let Err(error) = call_method(
            staged_imports,
            "add",
            &[
                JsValue::from_str(&candidate.serialized_staged),
                JsValue::from_str(&candidate.operation_id),
            ],
        ) {
            *output.borrow_mut() = Some(Err(js_error(error)));
            let _ = call_method(transaction, "abort", &[]);
            return;
        }
        if let Err(error) = call_method(
            staged_idempotency_keys,
            "add",
            &[
                JsValue::from_str(&candidate.operation_id),
                JsValue::from_str(&candidate.idempotency_storage_key),
            ],
        ) {
            *output.borrow_mut() = Some(Err(js_error(error)));
            let _ = call_method(transaction, "abort", &[]);
            return;
        }
    }

    for (operation_id, idempotency_storage_key) in stale_staging_to_remove {
        if let Err(error) =
            call_method(staged_imports, "delete", &[JsValue::from_str(&operation_id)])
        {
            *output.borrow_mut() = Some(Err(js_error(error)));
            let _ = call_method(transaction, "abort", &[]);
            return;
        }
        if let Err(error) = call_method(
            staged_idempotency_keys,
            "delete",
            &[JsValue::from_str(&idempotency_storage_key)],
        ) {
            *output.borrow_mut() = Some(Err(js_error(error)));
            let _ = call_method(transaction, "abort", &[]);
            return;
        }
    }

    *output.borrow_mut() = Some(Ok(report));
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
    new_record: &StoredOperationV1,
    new_record_json: &str,
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
                "operation store contains a non-string stored record".into(),
            )),
            Some(serialized) => {
                let stored: Result<StoredOperationV1, _> = serde_json::from_str(&serialized);
                match stored {
                    Err(error) => Err(IndexedDbJournalError(format!(
                        "stored operation JSON invalid: {error}"
                    ))),
                    Ok(stored) => match stored.validate() {
                        Err(error) => Err(error),
                        Ok(()) => match stored.validate_entry(operation_id) {
                            Err(error) => Err(error),
                            Ok(existing_entry) => {
                                let same_envelope =
                                    stored.snapshot.envelope == new_record.snapshot.envelope;
                                let same_payload =
                                    existing_entry.canonical_payload_utf8() == new_payload_bytes;
                                let same_owner = stored.owner_scope == new_record.owner_scope;
                                let same_namespace =
                                    stored.operation_namespace == new_record.operation_namespace;
                                if !same_envelope || !same_payload || !same_owner || !same_namespace
                                {
                                    Err(IndexedDbJournalError(
                                        "operation ID is already bound to different immutable content or owner scope".into(),
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
                JsValue::from_str(new_record_json),
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

fn require_scope(
    owner_scope: &str,
    operation_namespace: &str,
) -> Result<(), IndexedDbJournalError> {
    if owner_scope.trim().is_empty() {
        return Err(IndexedDbJournalError(
            "owner scope must not be empty".into(),
        ));
    }
    if operation_namespace.trim().is_empty() {
        return Err(IndexedDbJournalError(
            "operation namespace must not be empty".into(),
        ));
    }
    Ok(())
}

/// Serializes the unique IndexedDB idempotency index key as a JSON tuple so
/// the scope cannot collide through delimiter ambiguity. The raw key itself
/// remains in the immutable envelope and on the wire.
fn scoped_idempotency_key(
    owner_scope: &str,
    operation_namespace: &str,
    idempotency_key: &str,
) -> Result<String, IndexedDbJournalError> {
    require_scope(owner_scope, operation_namespace)?;
    if idempotency_key.trim().is_empty() {
        return Err(IndexedDbJournalError(
            "idempotency key must not be empty".into(),
        ));
    }
    serde_json::to_string(&(owner_scope, operation_namespace, idempotency_key)).map_err(|error| {
        IndexedDbJournalError(format!("idempotency index key encoding failed: {error}"))
    })
}

fn is_sha256_lower_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
}

fn is_pending(state: &JournalState) -> bool {
    matches!(state, JournalState::Prepared | JournalState::OutcomeUnknown)
}

fn secure_random_token() -> Result<String, IndexedDbJournalError> {
    let window =
        web_sys::window().ok_or_else(|| IndexedDbJournalError("window is unavailable".into()))?;
    let crypto = Reflect::get(window.as_ref(), &JsValue::from_str("crypto")).map_err(js_error)?;
    if crypto.is_null() || crypto.is_undefined() {
        return Err(IndexedDbJournalError(
            "secure browser crypto is unavailable; refusing to mint a dispatch lease".into(),
        ));
    }
    let token = call_method(&crypto, "randomUUID", &[])
        .map_err(js_error)?
        .as_string()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            IndexedDbJournalError(
                "crypto.randomUUID is unavailable; refusing to mint a dispatch lease".into(),
            )
        })?;
    Ok(token)
}

fn epoch_now_ms() -> Result<u64, IndexedDbJournalError> {
    let now = Date::now();
    if !now.is_finite() || now < 0.0 || now > u64::MAX as f64 {
        return Err(IndexedDbJournalError(
            "browser wall clock is not representable as epoch milliseconds".into(),
        ));
    }
    Ok(now.floor() as u64)
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

#[cfg(test)]
mod dispatch_lease_tests {
    use super::{DISPATCH_LEASE_DURATION_MS, DISPATCH_LEASE_SCHEMA_VERSION, DispatchLeaseRecordV1};

    fn valid_lease() -> DispatchLeaseRecordV1 {
        DispatchLeaseRecordV1 {
            schema_version: DISPATCH_LEASE_SCHEMA_VERSION,
            operation_id: "op-lease-test".to_owned(),
            owner_token: "opaque-random-token".to_owned(),
            acquired_at_ms: 1_000,
            expires_at_ms: 1_000 + DISPATCH_LEASE_DURATION_MS,
        }
    }

    #[test]
    fn lease_is_active_until_but_not_at_expiry() {
        let lease = valid_lease();
        assert!(lease.is_active_at(lease.expires_at_ms - 1));
        assert!(!lease.is_active_at(lease.expires_at_ms));
        assert!(!lease.is_active_at(lease.expires_at_ms + 1));
    }

    #[test]
    fn lease_record_rejects_wrong_operation_or_unknown_schema() {
        let lease = valid_lease();
        assert!(lease.validate("op-lease-test").is_ok());
        assert!(lease.validate("other-operation").is_err());

        let mut unknown = valid_lease();
        unknown.schema_version += 1;
        assert!(unknown.validate("op-lease-test").is_err());
    }

    #[test]
    fn lease_record_rejects_empty_token_and_nonpositive_lifetime() {
        let mut empty_token = valid_lease();
        empty_token.owner_token = "  ".to_owned();
        assert!(empty_token.validate("op-lease-test").is_err());

        let mut no_lifetime = valid_lease();
        no_lifetime.expires_at_ms = no_lifetime.acquired_at_ms;
        assert!(no_lifetime.validate("op-lease-test").is_err());
    }
}

#[cfg(test)]
mod owner_scope_tests {
    use super::{
        STORED_OPERATION_SCHEMA_VERSION, StoredOperationV1, require_scope, scoped_idempotency_key,
    };
    use sol_atlas_core::offline_journal::{
        JournalEntry, OFFLINE_JOURNAL_SCHEMA_VERSION, OperationEnvelope,
        RequiredNullableOperationId,
    };
    use sol_atlas_core::offline_payload_digest::canonical_payload_digest_sha256;

    const PAYLOAD: &str = r#"{"x":1}"#;

    fn stored_operation() -> StoredOperationV1 {
        let envelope = OperationEnvelope {
            schema_version: OFFLINE_JOURNAL_SCHEMA_VERSION,
            operation_id: "op-owner-test".to_owned(),
            aggregate_id: "aggregate-1".to_owned(),
            expected_revision: 0,
            payload_digest_sha256: canonical_payload_digest_sha256(PAYLOAD).unwrap(),
            idempotency_key: "idem-1".to_owned(),
            supersedes_operation_id: RequiredNullableOperationId::Null(()),
        };
        let entry = JournalEntry::prepare(envelope, PAYLOAD).unwrap();
        let canonical_payload = std::str::from_utf8(entry.canonical_payload_utf8()).unwrap();
        let request_fingerprint_sha256 =
            sol_atlas_core::offline_idempotency_fingerprint::canonical_request_fingerprint_sha256(
                "tenant-a",
                "work-order.apply",
                entry.envelope(),
                canonical_payload,
            )
            .unwrap();
        StoredOperationV1 {
            schema_version: STORED_OPERATION_SCHEMA_VERSION,
            owner_scope: "tenant-a".to_owned(),
            operation_namespace: "work-order.apply".to_owned(),
            request_fingerprint_schema_version:
                sol_atlas_core::offline_idempotency_fingerprint::REQUEST_FINGERPRINT_SCHEMA_VERSION,
            request_fingerprint_sha256,
            snapshot: entry.to_snapshot().unwrap(),
        }
    }

    #[test]
    fn stored_operation_requires_exact_owner_and_namespace() {
        let stored = stored_operation();
        assert!(stored.validate().is_ok());
        assert!(stored.matches_scope("tenant-a", "work-order.apply"));
        assert!(!stored.matches_scope("tenant-b", "work-order.apply"));
        assert!(!stored.matches_scope("tenant-a", "work-order.cancel"));

        assert!(require_scope("tenant-a", "work-order.apply").is_ok());
        assert!(require_scope("  ", "work-order.apply").is_err());
        assert!(require_scope("tenant-a", "").is_err());
    }

    #[test]
    fn idempotency_index_keys_are_unique_by_owner_namespace_and_raw_key() {
        let a = scoped_idempotency_key("tenant-a", "work-order.apply", "same-key").unwrap();
        let b = scoped_idempotency_key("tenant-b", "work-order.apply", "same-key").unwrap();
        let c = scoped_idempotency_key("tenant-a", "work-order.cancel", "same-key").unwrap();
        let d = scoped_idempotency_key("tenant-a", "work-order.apply", "other-key").unwrap();
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);

        // JSON tuple serialization avoids collisions caused by ad-hoc delimiters.
        let left = scoped_idempotency_key("tenant:a", "work-order.apply", "key").unwrap();
        let right = scoped_idempotency_key("tenant", "a:work-order.apply", "key").unwrap();
        assert_ne!(left, right);
    }

    #[test]
    fn owner_bound_record_roundtrips_and_legacy_snapshot_is_not_a_record() {
        let stored = stored_operation();
        let serialized = serde_json::to_string(&stored).unwrap();
        let decoded: StoredOperationV1 = serde_json::from_str(&serialized).unwrap();
        assert_eq!(decoded, stored);

        let legacy_snapshot = serde_json::to_string(&stored.snapshot).unwrap();
        assert!(serde_json::from_str::<StoredOperationV1>(&legacy_snapshot).is_err());
    }

    #[test]
    fn request_fingerprint_is_bound_to_persisted_owner_and_namespace() {
        let stored = stored_operation();
        assert!(stored.validate_entry("op-owner-test").is_ok());

        let mut wrong_owner = stored.clone();
        wrong_owner.owner_scope = "tenant-b".to_owned();
        assert!(wrong_owner.validate_entry("op-owner-test").is_err());

        let mut wrong_namespace = stored.clone();
        wrong_namespace.operation_namespace = "work-order.cancel".to_owned();
        assert!(wrong_namespace.validate_entry("op-owner-test").is_err());

        let mut changed_fingerprint = stored.clone();
        changed_fingerprint.request_fingerprint_sha256 = "0".repeat(64);
        assert!(changed_fingerprint.validate_entry("op-owner-test").is_err());

        let mut changed_payload_binding = stored.clone();
        changed_payload_binding.snapshot.envelope.aggregate_id = "aggregate-other".to_owned();
        assert!(
            changed_payload_binding
                .validate_entry("op-owner-test")
                .is_err()
        );
    }

    #[test]
    fn owner_record_rejects_unknown_storage_schema_and_empty_scope() {
        let mut unknown = stored_operation();
        unknown.schema_version += 1;
        assert!(unknown.validate().is_err());

        let mut empty_owner = stored_operation();
        empty_owner.owner_scope = " ".to_owned();
        assert!(empty_owner.validate().is_err());

        let mut empty_namespace = stored_operation();
        empty_namespace.operation_namespace.clear();
        assert!(empty_namespace.validate().is_err());
    }
}
