use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs, process,
};

const ARCHITECTURE_ID: &str = "NEIGHBORHOOD_RESILIENCE_COMMONS";
const SAMPLE: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/NEIGHBORHOOD_COMMONS_REFERENCE_MANIFEST_V0.1.json"
);
const STRESS_MATRIX: &str = include_str!(
    "../../docs/research/highveld-neighborhood-commons/NEIGHBORHOOD_COMMONS_STRESS_MATRIX_V0.1.json"
);

#[derive(Default)]
struct Audit {
    errors: Vec<Value>,
    warnings: Vec<Value>,
}

impl Audit {
    fn fail(&mut self, code: &str, path: &str, detail: &str) {
        self.errors
            .push(json!({"code": code, "path": path, "detail": detail}));
    }

    fn warn(&mut self, code: &str, path: &str, detail: &str) {
        self.warnings
            .push(json!({"code": code, "path": path, "detail": detail}));
    }
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
}

fn number(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn list<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn unique_ids(items: &[Value], key: &str, path: &str, audit: &mut Audit) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        match text(item, key) {
            Some(id) if ids.insert(id.to_owned()) => {}
            Some(id) => audit.fail(
                "DUPLICATE_ID",
                &format!("{path}[{index}].{key}"),
                &format!("duplicate identifier {id}"),
            ),
            None => audit.fail(
                "ID_MISSING",
                &format!("{path}[{index}].{key}"),
                "identifier must be a non-empty string",
            ),
        }
    }
    ids
}

fn audit_manifest(root: &Value) -> Value {
    let mut audit = Audit::default();

    if text(root, "schema_version") != Some("0.1.0") {
        audit.fail(
            "SCHEMA_VERSION_MISMATCH",
            "$.schema_version",
            "expected 0.1.0",
        );
    }
    if text(root, "architecture_id") != Some(ARCHITECTURE_ID) {
        audit.fail(
            "ARCHITECTURE_ID_MISMATCH",
            "$.architecture_id",
            "unexpected architecture identifier",
        );
    }
    if text(root, "maturity") != Some("PROPOSED_NOT_EVALUATED") {
        audit.fail(
            "MATURITY_CLAIM_ESCALATION",
            "$.maturity",
            "this candidate must remain PROPOSED_NOT_EVALUATED until separately accepted and evaluated",
        );
    }
    if text(root, "evidence_class") != Some("SyntheticControlledDesign") {
        audit.fail(
            "EVIDENCE_CLASS_MISMATCH",
            "$.evidence_class",
            "reference manifest must remain explicitly synthetic",
        );
    }
    if text(root, "claim_ceiling").map_or(true, |s| s.len() < 30) {
        audit.fail(
            "CLAIM_CEILING_MISSING",
            "$.claim_ceiling",
            "a meaningful claim ceiling is required",
        );
    }

    let households = list(root, "households");
    let assets = list(root, "assets");
    let contracts = list(root, "service_contracts");
    let policies = list(root, "allocation_policies");
    let maintenance_contracts = list(root, "maintenance_contracts");
    let accounts = list(root, "resource_accounts");
    let transfers = list(root, "resource_transfers");
    let service_outcomes = list(root, "household_service_outcomes");
    let consent_records = list(root, "consent_records");
    let household_data_requests = list(root, "household_data_access_requests");

    let household_ids = unique_ids(households, "household_id", "$.households", &mut audit);
    let asset_ids = unique_ids(assets, "asset_id", "$.assets", &mut audit);
    let service_ids: BTreeSet<String> = contracts
        .iter()
        .filter_map(|contract| text(contract, "service_id").map(ToOwned::to_owned))
        .collect();
    let contract_ids = unique_ids(contracts, "contract_id", "$.service_contracts", &mut audit);
    let policy_ids = unique_ids(policies, "policy_id", "$.allocation_policies", &mut audit);
    let maintenance_ids = unique_ids(
        maintenance_contracts,
        "maintenance_contract_id",
        "$.maintenance_contracts",
        &mut audit,
    );
    let account_ids = unique_ids(accounts, "account_id", "$.resource_accounts", &mut audit);
    let transfer_ids = unique_ids(transfers, "transfer_id", "$.resource_transfers", &mut audit);
    let outcome_ids = unique_ids(
        service_outcomes,
        "outcome_id",
        "$.household_service_outcomes",
        &mut audit,
    );
    let consent_ids = unique_ids(
        consent_records,
        "consent_id",
        "$.consent_records",
        &mut audit,
    );
    let data_request_ids = unique_ids(
        household_data_requests,
        "request_id",
        "$.household_data_access_requests",
        &mut audit,
    );
    let mut source_event_ids = BTreeSet::new();

    if households.is_empty() {
        audit.fail(
            "HOUSEHOLDS_EMPTY",
            "$.households",
            "at least one synthetic household is required",
        );
    }
    if assets.is_empty() {
        audit.fail("ASSETS_EMPTY", "$.assets", "at least one asset is required");
    }
    if contracts.is_empty() {
        audit.fail(
            "SERVICE_CONTRACTS_EMPTY",
            "$.service_contracts",
            "at least one shared service contract is required",
        );
    }

    let mut asset_domains: BTreeMap<String, String> = BTreeMap::new();
    let mut asset_kinds: BTreeMap<String, String> = BTreeMap::new();
    for (index, asset) in assets.iter().enumerate() {
        let id = text(asset, "asset_id").unwrap_or("<missing>");
        match text(asset, "failure_domain_id") {
            Some(domain) => {
                asset_domains.insert(id.to_owned(), domain.to_owned());
            }
            None => audit.fail(
                "ASSET_FAILURE_DOMAIN_MISSING",
                &format!("$.assets[{index}].failure_domain_id"),
                "every asset must declare a failure domain",
            ),
        }
        if let Some(kind) = text(asset, "asset_kind") {
            asset_kinds.insert(id.to_owned(), kind.to_owned());
        } else {
            audit.fail(
                "ASSET_KIND_MISSING",
                &format!("$.assets[{index}].asset_kind"),
                "asset_kind is required",
            );
        }
    }

    let mut policy_by_id: BTreeMap<String, &Value> = BTreeMap::new();
    for policy in policies {
        if let Some(id) = text(policy, "policy_id") {
            policy_by_id.insert(id.to_owned(), policy);
        }
    }
    let mut maintenance_by_id: BTreeMap<String, &Value> = BTreeMap::new();
    for maintenance in maintenance_contracts {
        if let Some(id) = text(maintenance, "maintenance_contract_id") {
            maintenance_by_id.insert(id.to_owned(), maintenance);
        }
    }

    let mut provider_by_service: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (index, contract) in contracts.iter().enumerate() {
        let path = format!("$.service_contracts[{index}]");
        let service_id = text(contract, "service_id");
        let provider = text(contract, "provider_asset_id");
        let contract_unit = text(contract, "capacity_unit");
        let capacity = number(contract, "capacity_per_window");
        let demand = number(contract, "peak_demand_per_window");

        if service_id.is_none() {
            audit.fail(
                "SERVICE_ID_MISSING",
                &format!("{path}.service_id"),
                "service_id is required",
            );
        }
        if provider.map_or(true, |id| !asset_ids.contains(id)) {
            audit.fail(
                "UNKNOWN_SERVICE_PROVIDER",
                &format!("{path}.provider_asset_id"),
                "provider asset must exist",
            );
        }
        if let (Some(service), Some(asset)) = (service_id, provider) {
            if let Some(domain) = asset_domains.get(asset) {
                provider_by_service
                    .entry(service.to_owned())
                    .or_default()
                    .push((asset.to_owned(), domain.clone()));
            }
        }
        if contract_unit.is_none() || capacity.is_none() || demand.is_none() {
            audit.fail(
                "SERVICE_CAPACITY_INCOMPLETE",
                &path,
                "capacity, peak demand and explicit capacity_unit are required",
            );
        }
        if let (Some(capacity), Some(demand)) = (capacity, demand) {
            if demand > capacity {
                let queue = contract.get("queue_policy");
                let enabled = queue
                    .and_then(|q| q.get("enabled"))
                    .and_then(Value::as_bool)
                    == Some(true);
                let max_wait = queue.and_then(|q| number(q, "maximum_wait_s"));
                let disposition = queue.and_then(|q| text(q, "overflow_disposition"));
                if !enabled
                    || max_wait.is_none()
                    || !matches!(
                        disposition,
                        Some("BLOCK_AND_ESCALATE" | "RECORD_UNMET_DEMAND")
                    )
                {
                    audit.fail(
                        "CAPACITY_OVERCOMMIT_UNMODELED",
                        &format!("{path}.queue_policy"),
                        "peak demand exceeds service capacity without an explicit bounded queue and unmet-demand disposition",
                    );
                }
            }
        }
        let access = contract.get("access_path");
        let access_state = access.and_then(|v| text(v, "qualification_state"));
        let access_evidence = access.and_then(|v| text(v, "evidence_id"));
        let allocation_enabled =
            contract.get("allocation_enabled").and_then(Value::as_bool) == Some(true);
        if allocation_enabled && (access_state != Some("QUALIFIED") || access_evidence.is_none()) {
            audit.fail(
                "UNQUALIFIED_ACCESS_ALLOCATED",
                &format!("{path}.access_path"),
                "an allocated shared service needs qualified access evidence",
            );
        }
        if !matches!(
            access_state,
            Some("QUALIFIED" | "UNVERIFIED" | "FAILED" | "UNKNOWN")
        ) {
            audit.fail(
                "ACCESS_STATE_INVALID",
                &format!("{path}.access_path.qualification_state"),
                "access state must be QUALIFIED, UNVERIFIED, FAILED or UNKNOWN",
            );
        }

        let policy_id = text(contract, "allocation_policy_id");
        let maintenance_id = text(contract, "maintenance_contract_id");
        if allocation_enabled {
            match maintenance_id.and_then(|id| maintenance_by_id.get(id).copied()) {
                None => audit.fail(
                    "MAINTENANCE_CONTRACT_NOT_QUALIFIED",
                    &format!("{path}.maintenance_contract_id"),
                    "allocated shared service needs a registered maintenance/spares contract",
                ),
                Some(maintenance) => {
                    if text(maintenance, "asset_id") != provider
                        || text(maintenance, "spares_state") != Some("AVAILABLE")
                        || text(maintenance, "repair_access_state") != Some("QUALIFIED")
                        || text(maintenance, "maintainer_id").is_none()
                        || text(maintenance, "requalification_evidence_id").is_none()
                        || number(maintenance, "maximum_repair_time_s").is_none()
                    {
                        audit.fail(
                            "MAINTENANCE_CONTRACT_NOT_QUALIFIED",
                            &format!("{path}.maintenance_contract_id"),
                            "maintenance contract must bind the provider asset, qualified access, available spares, maintainer, repair bound and requalification evidence",
                        );
                    }
                }
            }
            match policy_id.and_then(|id| policy_by_id.get(id).copied()) {
                None => audit.fail(
                    "ALLOCATION_POLICY_MISSING",
                    &format!("{path}.allocation_policy_id"),
                    "enabled allocation must reference a registered policy",
                ),
                Some(policy) => {
                    if text(policy, "approval_state") != Some("APPROVED")
                        || text(policy, "final_authority") != Some("HUMAN")
                        || policy.get("appeal_path").and_then(Value::as_bool) != Some(true)
                        || policy.get("resident_oversight").and_then(Value::as_bool) != Some(true)
                        || policy
                            .get("algorithmic_priority_authority")
                            .and_then(Value::as_bool)
                            != Some(false)
                    {
                        audit.fail(
                            "ALLOCATION_GOVERNANCE_NOT_QUALIFIED",
                            &format!("{path}.allocation_policy_id"),
                            "allocation requires approved resident oversight, an appeal path and human final authority",
                        );
                    }
                }
            }
        }
    }

    for (index, household) in households.iter().enumerate() {
        let path = format!("$.households[{index}]");
        let household_id = text(household, "household_id").unwrap_or("<missing>");
        let floors = list(household, "essential_service_floors");
        if floors.is_empty() {
            audit.fail(
                "ESSENTIAL_FLOORS_EMPTY",
                &format!("{path}.essential_service_floors"),
                "each household needs declared essential service floors",
            );
        }
        for (floor_index, floor) in floors.iter().enumerate() {
            let floor_path = format!("{path}.essential_service_floors[{floor_index}]");
            let service_id = text(floor, "service_id");
            if floor.get("essential").and_then(Value::as_bool) != Some(true)
                || floor.get("must_survive_hub_loss").and_then(Value::as_bool) != Some(true)
            {
                audit.fail(
                    "HOUSEHOLD_FLOOR_NOT_PROTECTED",
                    &floor_path,
                    "every declared essential service must remain protected during hub loss",
                );
            }
            if number(floor, "maximum_unserved_s").is_none()
                || number(floor, "maximum_wait_s").is_none()
            {
                audit.fail(
                    "SERVICE_FLOOR_BOUNDARY_MISSING",
                    &floor_path,
                    "maximum unserved time and maximum wait must be explicit",
                );
            }
            let fallback = floor.get("local_fallback");
            let fallback_asset = fallback.and_then(|v| text(v, "asset_id"));
            let fallback_domain = fallback.and_then(|v| text(v, "failure_domain_id"));
            if fallback_asset.map_or(true, |id| !asset_ids.contains(id)) {
                audit.fail(
                    "LOCAL_FALLBACK_MISSING",
                    &format!("{floor_path}.local_fallback.asset_id"),
                    "a household-local fallback asset must exist",
                );
            }
            if fallback.and_then(|v| text(v, "qualification_state")) != Some("QUALIFIED")
                || fallback.and_then(|v| text(v, "evidence_id")).is_none()
            {
                audit.fail(
                    "LOCAL_FALLBACK_UNQUALIFIED",
                    &format!("{floor_path}.local_fallback"),
                    "local fallback requires qualified evidence and an evidence identifier",
                );
            }
            if fallback
                .and_then(|v| v.get("depends_on_commons"))
                .and_then(Value::as_bool)
                != Some(false)
            {
                audit.fail(
                    "LOCAL_FALLBACK_DEPENDS_ON_COMMONS",
                    &format!("{floor_path}.local_fallback.depends_on_commons"),
                    "essential household fallback cannot depend on the commons",
                );
            }
            if fallback
                .and_then(|v| v.get("depends_on_network"))
                .and_then(Value::as_bool)
                != Some(false)
            {
                audit.fail(
                    "LOCAL_FALLBACK_DEPENDS_ON_NETWORK",
                    &format!("{floor_path}.local_fallback.depends_on_network"),
                    "household-local fallback must remain usable without external network services",
                );
            }
            if fallback
                .and_then(|v| v.get("manual_mode_available"))
                .and_then(Value::as_bool)
                != Some(true)
            {
                audit.fail(
                    "LOCAL_FALLBACK_NO_MANUAL_MODE",
                    &format!("{floor_path}.local_fallback.manual_mode_available"),
                    "local degraded operation must have a declared manual mode",
                );
            }
            if let Some(asset) = fallback_asset {
                if asset_kinds.get(asset).map(String::as_str) == Some("COMMONS_SHARED_ASSET") {
                    audit.fail(
                        "FALLBACK_IS_COMMONS_ASSET",
                        &format!("{floor_path}.local_fallback.asset_id"),
                        "a household-local fallback cannot be a commons shared asset",
                    );
                }
                if let Some(expected_domain) = asset_domains.get(asset) {
                    if fallback_domain != Some(expected_domain.as_str()) {
                        audit.fail(
                            "FALLBACK_FAILURE_DOMAIN_MISMATCH",
                            &format!("{floor_path}.local_fallback.failure_domain_id"),
                            "fallback failure domain must match its referenced asset",
                        );
                    }
                }
            }
            if let Some(service) = service_id {
                let providers = provider_by_service.get(service);
                if providers.is_none_or(Vec::is_empty) {
                    audit.fail(
                        "ESSENTIAL_SERVICE_NOT_CONTRACTED",
                        &format!("{floor_path}.service_id"),
                        "declared essential service must map to a commons service contract",
                    );
                } else if let Some(fallback_domain) = fallback_domain {
                    if let Some(providers) = providers {
                        for (_, provider_domain) in providers {
                            if provider_domain == fallback_domain {
                                audit.fail(
                                    "LOCAL_FALLBACK_COMMON_CAUSE",
                                    &format!("{floor_path}.local_fallback.failure_domain_id"),
                                    &format!("household {household_id} fallback shares failure domain with the commons provider"),
                                );
                            }
                        }
                    }
                }
                for contract in contracts
                    .iter()
                    .filter(|c| text(c, "service_id") == Some(service))
                {
                    let queue_max = contract
                        .get("queue_policy")
                        .and_then(|q| number(q, "maximum_wait_s"));
                    let floor_max = number(floor, "maximum_wait_s");
                    if queue_max
                        .zip(floor_max)
                        .is_some_and(|(queue, floor_wait)| queue > floor_wait)
                    {
                        audit.fail(
                            "QUEUE_EXCEEDS_RESIDENT_FLOOR",
                            &floor_path,
                            "shared-service queue wait exceeds this household's declared maximum",
                        );
                    }
                    let access_max = contract
                        .get("access_path")
                        .and_then(|a| number(a, "maximum_access_time_s"));
                    let floor_access_max = number(floor, "maximum_access_time_s");
                    if access_max
                        .zip(floor_access_max)
                        .is_some_and(|(access, floor_access)| access > floor_access)
                    {
                        audit.fail(
                            "ACCESS_EXCEEDS_RESIDENT_FLOOR",
                            &floor_path,
                            "shared-service access time exceeds this household's declared access boundary",
                        );
                    }
                }
            }
        }
    }

    // One transfer is counted once in the account balance calculation. Repeated
    // source-event identities are rejected to prevent duplicate telemetry/receipt
    // ingestion from inflating usable community resources.
    let mut transfer_in: BTreeMap<String, u64> = BTreeMap::new();
    let mut transfer_out: BTreeMap<String, u64> = BTreeMap::new();
    for (index, transfer) in transfers.iter().enumerate() {
        let path = format!("$.resource_transfers[{index}]");
        if let Some(source_event_id) = text(transfer, "source_event_id") {
            if !source_event_ids.insert(source_event_id.to_owned()) {
                audit.fail(
                    "DUPLICATE_RESOURCE_SOURCE_EVENT",
                    &format!("{path}.source_event_id"),
                    "one source event must not be counted more than once in the resource ledger",
                );
            }
        } else {
            audit.fail(
                "RESOURCE_SOURCE_EVENT_MISSING",
                &format!("{path}.source_event_id"),
                "source_event_id is required",
            );
        }

        let from = text(transfer, "from_account_id");
        let to = text(transfer, "to_account_id");
        let resource = text(transfer, "resource_id");
        let unit = text(transfer, "unit");
        let quantity = number(transfer, "quantity");
        if from == to {
            audit.fail(
                "SELF_TRANSFER",
                &path,
                "resource transfer must have distinct source and destination accounts",
            );
        }
        if from.map_or(true, |id| !account_ids.contains(id))
            || to.map_or(true, |id| !account_ids.contains(id))
        {
            audit.fail(
                "UNKNOWN_TRANSFER_ACCOUNT",
                &path,
                "both transfer accounts must be registered",
            );
        }
        if let Some(quantity) = quantity {
            if quantity == 0 {
                audit.fail(
                    "ZERO_RESOURCE_TRANSFER",
                    &format!("{path}.quantity"),
                    "transfer quantity must be positive",
                );
            }
            if let Some(account) = from {
                let sum = transfer_out.entry(account.to_owned()).or_default();
                match sum.checked_add(quantity) {
                    Some(next) => *sum = next,
                    None => audit.fail(
                        "RESOURCE_LEDGER_OVERFLOW",
                        &path,
                        "outgoing transfer sum overflowed",
                    ),
                }
            }
            if let Some(account) = to {
                let sum = transfer_in.entry(account.to_owned()).or_default();
                match sum.checked_add(quantity) {
                    Some(next) => *sum = next,
                    None => audit.fail(
                        "RESOURCE_LEDGER_OVERFLOW",
                        &path,
                        "incoming transfer sum overflowed",
                    ),
                }
            }
        } else {
            audit.fail(
                "TRANSFER_QUANTITY_MISSING",
                &format!("{path}.quantity"),
                "quantity must be a non-negative integer",
            );
        }
        if resource.is_none() || unit.is_none() {
            audit.fail(
                "TRANSFER_UNIT_OR_RESOURCE_MISSING",
                &path,
                "resource_id and unit must be explicit; no implicit conversion is allowed",
            );
        }
    }

    let mut account_resource_units: BTreeMap<String, (String, String)> = BTreeMap::new();
    for (index, account) in accounts.iter().enumerate() {
        let path = format!("$.resource_accounts[{index}]");
        let id = text(account, "account_id").unwrap_or("<missing>");
        let asset = text(account, "asset_id");
        let resource = text(account, "resource_id");
        let unit = text(account, "unit");
        if asset.map_or(true, |asset_id| !asset_ids.contains(asset_id)) {
            audit.fail(
                "RESOURCE_ACCOUNT_ASSET_UNKNOWN",
                &format!("{path}.asset_id"),
                "resource account must belong to a registered asset",
            );
        }
        if let (Some(resource), Some(unit)) = (resource, unit) {
            account_resource_units.insert(id.to_owned(), (resource.to_owned(), unit.to_owned()));
        } else {
            audit.fail(
                "RESOURCE_ACCOUNT_UNIT_MISSING",
                &path,
                "resource and unit must be explicit",
            );
        }

        let opening = number(account, "opening_quantity");
        let inflow = number(account, "external_inflow_quantity");
        let committed = number(account, "committed_outflow_quantity");
        let losses = number(account, "known_loss_quantity");
        let reserve = number(account, "reserve_floor_quantity");
        if let (Some(opening), Some(inflow), Some(committed), Some(losses), Some(reserve)) =
            (opening, inflow, committed, losses, reserve)
        {
            let available = opening
                .checked_add(inflow)
                .and_then(|v| v.checked_add(*transfer_in.get(id).unwrap_or(&0)));
            let used = committed
                .checked_add(losses)
                .and_then(|v| v.checked_add(*transfer_out.get(id).unwrap_or(&0)));
            match available.zip(used).and_then(|(a, u)| a.checked_sub(u)) {
                Some(remaining) if remaining >= reserve => {}
                Some(remaining) => audit.fail(
                    "RESOURCE_RESERVE_BREACH",
                    &path,
                    &format!(
                        "remaining quantity {remaining} is below required reserve floor {reserve}"
                    ),
                ),
                None => audit.fail(
                    "RESOURCE_BALANCE_NEGATIVE_OR_OVERFLOW",
                    &path,
                    "resource balance is negative or arithmetic overflowed",
                ),
            }
        } else {
            audit.fail(
                "RESOURCE_ACCOUNT_BALANCE_INCOMPLETE",
                &path,
                "opening, external inflow, committed outflow, known loss and reserve floor are required integer quantities",
            );
        }
    }

    for (index, transfer) in transfers.iter().enumerate() {
        let path = format!("$.resource_transfers[{index}]");
        let from = text(transfer, "from_account_id").unwrap_or("");
        let to = text(transfer, "to_account_id").unwrap_or("");
        let declared = (text(transfer, "resource_id"), text(transfer, "unit"));
        for account_id in [from, to] {
            if let Some(actual) = account_resource_units.get(account_id) {
                if declared != (Some(actual.0.as_str()), Some(actual.1.as_str())) {
                    audit.fail(
                        "RESOURCE_TRANSFER_UNIT_MISMATCH",
                        &path,
                        "transfer resource/unit must match both ledger accounts; implicit conversions are prohibited",
                    );
                }
            }
        }
    }

    // Resource accounts and transfer provenance remain separate from delivered
    // service observations. A reservoir/grid/hub signal is never substituted for
    // a household service outcome.
    for (index, account) in accounts.iter().enumerate() {
        if text(account, "resource_id").is_none() {
            audit.fail(
                "RESOURCE_ID_MISSING",
                &format!("$.resource_accounts[{index}].resource_id"),
                "resource_id is required",
            );
        }
    }

    let mut contract_by_service: BTreeMap<String, &Value> = BTreeMap::new();
    for contract in contracts {
        if let Some(service_id) = text(contract, "service_id") {
            contract_by_service.insert(service_id.to_owned(), contract);
        }
    }

    let mut outcome_keys: BTreeSet<(String, String, String)> = BTreeSet::new();
    let mut outcome_rows: Vec<Value> = Vec::new();
    let mut household_floor_breach_count = 0_usize;
    for (index, outcome) in service_outcomes.iter().enumerate() {
        let path = format!("$.household_service_outcomes[{index}]");
        let household_id = text(outcome, "household_id");
        let service_id = text(outcome, "service_id");
        let window_id = text(outcome, "window_id");
        let delivery_state = text(outcome, "delivery_state");
        let floor_state = text(outcome, "hard_floor_state");
        let unit = text(outcome, "unit");
        let outcome_id = text(outcome, "outcome_id").unwrap_or("<missing>");

        if household_id.is_none_or(|id| !household_ids.contains(id)) {
            audit.fail(
                "OUTCOME_HOUSEHOLD_UNKNOWN",
                &format!("{path}.household_id"),
                "service outcome must reference a registered household",
            );
        }
        let floor = household_id
            .and_then(|hid| {
                households
                    .iter()
                    .find(|h| text(h, "household_id") == Some(hid))
            })
            .and_then(|h| {
                list(h, "essential_service_floors")
                    .iter()
                    .find(|f| text(f, "service_id") == service_id)
            });
        if floor.is_none() {
            audit.fail(
                "OUTCOME_NOT_A_DECLARED_ESSENTIAL_FLOOR",
                &path,
                "outcome must correspond to one declared household essential-service floor",
            );
        }
        let Some(service_id) = service_id else {
            audit.fail(
                "OUTCOME_SERVICE_ID_MISSING",
                &format!("{path}.service_id"),
                "service_id is required",
            );
            continue;
        };
        let Some(household_id) = household_id else {
            continue;
        };
        let Some(window_id) = window_id else {
            audit.fail(
                "OUTCOME_WINDOW_ID_MISSING",
                &format!("{path}.window_id"),
                "window_id is required",
            );
            continue;
        };
        let key = (
            household_id.to_owned(),
            service_id.to_owned(),
            window_id.to_owned(),
        );
        if !outcome_keys.insert(key) {
            audit.fail(
                "DUPLICATE_HOUSEHOLD_SERVICE_OUTCOME",
                &path,
                "there must be exactly one outcome per household, service and window",
            );
        }

        let contract = contract_by_service.get(service_id).copied();
        if contract.is_none() {
            audit.fail(
                "OUTCOME_SERVICE_NOT_CONTRACTED",
                &format!("{path}.service_id"),
                "outcome service must be present in a commons service contract",
            );
        } else if unit != contract.and_then(|c| text(c, "capacity_unit")) {
            audit.fail(
                "OUTCOME_UNIT_MISMATCH",
                &format!("{path}.unit"),
                "outcome unit must exactly match the service contract unit; no implicit conversion is allowed",
            );
        }

        if text(outcome, "evidence_class") != Some("SyntheticKnownAnswer") {
            audit.fail(
                "OUTCOME_EVIDENCE_CLASS_MISMATCH",
                &format!("{path}.evidence_class"),
                "v0.1 outcome fixtures must remain SyntheticKnownAnswer; empirical delivery requires a new admission path",
            );
        }
        if text(outcome, "evidence_id").is_none() {
            audit.fail(
                "OUTCOME_EVIDENCE_ID_MISSING",
                &format!("{path}.evidence_id"),
                "every known-answer outcome must identify its synthetic evidence row",
            );
        }

        let requested = number(outcome, "requested_quantity");
        let delivered = number(outcome, "delivered_quantity");
        let quantity_state_valid = match delivery_state {
            Some("DELIVERED") => requested.is_some() && delivered == requested,
            Some("PARTIAL") => {
                requested.is_some_and(|r| r > 0)
                    && delivered.is_some_and(|d| d > 0 && Some(d) < requested)
            }
            Some("UNMET") => requested.is_some_and(|r| r > 0) && delivered == Some(0),
            Some("UNKNOWN") => requested.is_some_and(|r| r > 0) && delivered.is_none(),
            _ => false,
        };
        if !quantity_state_valid {
            audit.fail(
                "OUTCOME_QUANTITY_STATE_MISMATCH",
                &format!("{path}.delivery_state"),
                "delivery_state must agree with requested_quantity and delivered_quantity (UNKNOWN requires null delivered_quantity)",
            );
        }

        let mut valid_floor_satisfied = false;
        if let Some(floor) = floor {
            let unserved = number(outcome, "unserved_s");
            let access = number(outcome, "access_time_s");
            let queue = number(outcome, "queue_wait_s");
            let within_limits = unserved
                .zip(number(floor, "maximum_unserved_s"))
                .is_some_and(|(a, b)| a <= b)
                && access
                    .zip(number(floor, "maximum_access_time_s"))
                    .is_some_and(|(a, b)| a <= b)
                && queue
                    .zip(number(floor, "maximum_wait_s"))
                    .is_some_and(|(a, b)| a <= b);
            valid_floor_satisfied = delivery_state == Some("DELIVERED") && within_limits;
            match floor_state {
                Some("SATISFIED") if !valid_floor_satisfied => audit.fail(
                    "HOUSEHOLD_FLOOR_FALSE_SATISFIED",
                    &format!("{path}.hard_floor_state"),
                    "SATISFIED requires full delivery and unserved/access/queue times within the household's declared limits",
                ),
                Some("SATISFIED") => {}
                Some("BREACHED") => {
                    household_floor_breach_count += 1;
                    if valid_floor_satisfied {
                        audit.fail(
                            "HOUSEHOLD_FLOOR_FALSE_BREACH",
                            &format!("{path}.hard_floor_state"),
                            "BREACHED conflicts with fully delivered service inside declared limits",
                        );
                    }
                }
                Some("UNKNOWN") => {
                    if delivery_state != Some("UNKNOWN") {
                        audit.fail(
                            "HOUSEHOLD_FLOOR_UNKNOWN_MISMATCH",
                            &format!("{path}.hard_floor_state"),
                            "UNKNOWN hard floor requires UNKNOWN service delivery",
                        );
                    }
                }
                _ => audit.fail(
                    "HOUSEHOLD_FLOOR_STATE_INVALID",
                    &format!("{path}.hard_floor_state"),
                    "hard_floor_state must be SATISFIED, BREACHED or UNKNOWN",
                ),
            }
        }

        outcome_rows.push(json!({
            "outcome_id": outcome_id,
            "household_id": household_id,
            "service_id": service_id,
            "window_id": window_id,
            "delivery_state": delivery_state,
            "hard_floor_state": floor_state,
            "requested_quantity": requested,
            "delivered_quantity": outcome.get("delivered_quantity").cloned().unwrap_or(Value::Null),
            "unit": unit,
            "valid_floor_satisfied": valid_floor_satisfied
        }));
    }

    // Every declared household-essential service must retain its own outcome row.
    // The qualifier preserves per-household results and does not compute a
    // neighbourhood-average score that could compensate for an individual breach.
    for household in households {
        let Some(household_id) = text(household, "household_id") else {
            continue;
        };
        for floor in list(household, "essential_service_floors") {
            let Some(service_id) = text(floor, "service_id") else {
                continue;
            };
            let covered = outcome_keys
                .iter()
                .any(|(hid, sid, _)| hid == household_id && sid == service_id);
            if !covered {
                audit.fail(
                    "ESSENTIAL_FLOOR_OUTCOME_MISSING",
                    "$.household_service_outcomes",
                    &format!("missing per-household outcome for {household_id}/{service_id}"),
                );
            }
        }
    }

    // Household data is never shared merely because a household uses a shared
    // neighborhood service. Each explicit access request must match one consent
    // record for the same household, data category, purpose and service.
    // The decision timestamp must fall after grant and before revocation.
    let mut authorized_data_requests = 0_usize;
    for (index, request) in household_data_requests.iter().enumerate() {
        let path = format!("$.household_data_access_requests[{index}]");
        let household_id = text(request, "household_id");
        let data_category = text(request, "data_category");
        let purpose = text(request, "purpose");
        let service_id = text(request, "service_id");
        let decision_time = number(request, "decision_as_of_offset_s");

        if household_id.is_none_or(|id| !household_ids.contains(id)) {
            audit.fail(
                "DATA_REQUEST_HOUSEHOLD_UNKNOWN",
                &format!("{path}.household_id"),
                "data request must reference a registered household",
            );
        }
        let service_exists = service_id.is_some_and(|sid| service_ids.contains(sid));
        if !service_exists {
            audit.fail(
                "DATA_REQUEST_SERVICE_UNKNOWN",
                &format!("{path}.service_id"),
                "data request must be scoped to a registered service",
            );
        }
        if data_category.is_none() || purpose.is_none() || decision_time.is_none() {
            audit.fail(
                "DATA_REQUEST_SCOPE_INCOMPLETE",
                &path,
                "data category, purpose and decision-as-of time are required",
            );
            continue;
        }
        let Some(decision_time) = decision_time else {
            continue;
        };

        let matching: Vec<&Value> = consent_records
            .iter()
            .filter(|consent| {
                text(consent, "household_id") == household_id
                    && text(consent, "data_category") == data_category
                    && text(consent, "purpose") == purpose
                    && text(consent, "service_id") == service_id
            })
            .collect();

        if matching.len() != 1 {
            audit.fail(
                "HOUSEHOLD_DATA_CONSENT_MISSING_OR_AMBIGUOUS",
                &path,
                "data access requires exactly one matching household consent record; absent or ambiguous consent fails closed",
            );
            continue;
        }

        let consent = matching[0];
        let granted_at = number(consent, "granted_at_offset_s");
        let revoked_at = consent.get("revoked_at_offset_s").and_then(Value::as_u64);
        let consent_state = text(consent, "consent_state");

        if granted_at.is_none_or(|granted| decision_time < granted) {
            audit.fail(
                "HOUSEHOLD_DATA_ACCESS_BEFORE_GRANT",
                &path,
                "data cannot be accessed before the matching consent grant",
            );
            continue;
        }
        if consent_state == Some("REVOKED") {
            match revoked_at {
                Some(revoked) if decision_time >= revoked => {
                    audit.fail(
                        "HOUSEHOLD_DATA_ACCESS_AFTER_REVOCATION",
                        &path,
                        "new data access after consent revocation is prohibited",
                    );
                    continue;
                }
                Some(_) => {}
                None => {
                    audit.fail(
                        "CONSENT_REVOCATION_TIME_MISSING",
                        &path,
                        "revoked consent requires an explicit revocation time",
                    );
                    continue;
                }
            }
        } else if consent_state != Some("GRANTED") || revoked_at.is_some() {
            audit.fail(
                "HOUSEHOLD_DATA_CONSENT_STATE_INVALID",
                &path,
                "consent must be GRANTED with no revocation time, or REVOKED with a revocation time",
            );
            continue;
        }

        if text(consent, "consent_revision").is_none()
            || text(consent, "consent_record_evidence_id").is_none()
            || text(request, "evidence_id").is_none()
        {
            audit.fail(
                "HOUSEHOLD_DATA_CONSENT_RECEIPT_INCOMPLETE",
                &path,
                "data access must retain consent revision, consent evidence and request evidence identity",
            );
            continue;
        }
        authorized_data_requests += 1;
    }

    for (index, consent) in consent_records.iter().enumerate() {
        let path = format!("$.consent_records[{index}]");
        let household_id = text(consent, "household_id");
        let service_id = text(consent, "service_id");
        let granted_at = number(consent, "granted_at_offset_s");
        let revoked_at = consent.get("revoked_at_offset_s").and_then(Value::as_u64);
        if household_id.is_none_or(|id| !household_ids.contains(id)) {
            audit.fail(
                "CONSENT_HOUSEHOLD_UNKNOWN",
                &format!("{path}.household_id"),
                "consent must reference a registered household",
            );
        }
        if service_id.is_none_or(|id| !service_ids.contains(id)) {
            audit.fail(
                "CONSENT_SERVICE_UNKNOWN",
                &format!("{path}.service_id"),
                "consent must be scoped to a registered service",
            );
        }
        match text(consent, "consent_state") {
            Some("GRANTED") if granted_at.is_some() && revoked_at.is_none() => {}
            Some("REVOKED")
                if granted_at.zip(revoked_at).is_some_and(|(grant, revoke)| revoke >= grant) => {}
            _ => audit.fail(
                "HOUSEHOLD_DATA_CONSENT_STATE_INVALID",
                &path,
                "GRANTED requires a grant time and null revocation; REVOKED requires ordered grant and revocation times",
            ),
        }
    }

    let result = json!({
        "status": if audit.errors.is_empty() { "PASS" } else { "FAIL" },
        "protocol": "highveld-neighborhood-commons-qualifier-v0.1",
        "architecture_id": ARCHITECTURE_ID,
        "maturity": "PROPOSED_NOT_EVALUATED",
        "households_checked": household_ids.len(),
        "assets_checked": asset_ids.len(),
        "service_contracts_checked": contract_ids.len(),
        "allocation_policies_checked": policy_ids.len(),
        "maintenance_contracts_checked": maintenance_ids.len(),
        "household_service_outcomes_checked": outcome_ids.len(),
        "household_floor_breach_count": household_floor_breach_count,
        "consent_records_checked": consent_ids.len(),
        "household_data_access_requests_checked": data_request_ids.len(),
        "authorized_household_data_access_requests": authorized_data_requests,
        "data_sharing_default": "DENY_UNLESS_MATCHING_CONSENT_VALID_AT_DECISION_TIME",
        "household_service_outcomes": outcome_rows,
        "neighborhood_aggregate_compensation": "NOT_COMPUTED",
        "resource_accounts_checked": account_ids.len(),
        "resource_transfers_checked": transfer_ids.len(),
        "error_count": audit.errors.len(),
        "warning_count": audit.warnings.len(),
        "errors": audit.errors,
        "warnings": audit.warnings,
        "claim_ceiling": "Synthetic architecture-contract qualification only. No measured neighborhood, household access, resource delivery, safety, equity, cost or resilience result is established."
    });
    result
}

fn sample_manifest() -> Value {
    serde_json::from_str(SAMPLE).unwrap_or_else(|_| Value::Null)
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.as_slice() == ["--self-test"] {
        let report = audit_manifest(&sample_manifest());
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_owned())
        );
        if report["status"].as_str() != Some("PASS") {
            process::exit(1);
        }
        return;
    }
    if args.len() == 2 && args[0] == "--input" {
        let content = match fs::read_to_string(&args[1]) {
            Ok(content) => content,
            Err(error) => {
                eprintln!("failed to read {}: {error}", args[1]);
                process::exit(2);
            }
        };
        let manifest = match serde_json::from_str::<Value>(&content) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("invalid JSON in {}: {error}", args[1]);
                process::exit(2);
            }
        };
        let report = audit_manifest(&manifest);
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_owned())
        );
        if report["status"].as_str() != Some("PASS") {
            process::exit(1);
        }
        return;
    }
    eprintln!(
        "Usage:\n  cargo run --bin highveld-commons-qualifier -- --self-test\n  cargo run --bin highveld-commons-qualifier -- --input <manifest.json>"
    );
    process::exit(2);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Value {
        sample_manifest()
    }

    fn has_code(report: &Value, code: &str) -> bool {
        report["errors"].as_array().is_some_and(|items| {
            items
                .iter()
                .any(|item| item.get("code").and_then(Value::as_str) == Some(code))
        })
    }

    #[test]
    fn household_data_access_without_consent_is_denied() {
        let mut manifest = base();
        manifest["household_data_access_requests"] = json!([{
            "request_id":"request-no-consent",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"water-service-planning",
            "service_id":"water-access",
            "decision_as_of_offset_s":100,
            "evidence_id":"synthetic-data-request-1"
        }]);
        assert!(has_code(
            &audit_manifest(&manifest),
            "HOUSEHOLD_DATA_CONSENT_MISSING_OR_AMBIGUOUS"
        ));
    }

    #[test]
    fn household_data_access_after_revocation_is_denied() {
        let mut manifest = base();
        manifest["consent_records"] = json!([{
            "consent_id":"consent-a1",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"water-service-planning",
            "service_id":"water-access",
            "consent_state":"REVOKED",
            "consent_revision":"consent-r2",
            "granted_at_offset_s":0,
            "revoked_at_offset_s":90,
            "consent_record_evidence_id":"synthetic-consent-evidence"
        }]);
        manifest["household_data_access_requests"] = json!([{
            "request_id":"request-after-revoke",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"water-service-planning",
            "service_id":"water-access",
            "decision_as_of_offset_s":100,
            "evidence_id":"synthetic-data-request-2"
        }]);
        assert!(has_code(
            &audit_manifest(&manifest),
            "HOUSEHOLD_DATA_ACCESS_AFTER_REVOCATION"
        ));
    }

    #[test]
    fn household_data_access_before_grant_is_denied() {
        let mut manifest = base();
        manifest["consent_records"] = json!([{
            "consent_id":"consent-a1",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"water-service-planning",
            "service_id":"water-access",
            "consent_state":"GRANTED",
            "consent_revision":"consent-r1",
            "granted_at_offset_s":200,
            "revoked_at_offset_s":null,
            "consent_record_evidence_id":"synthetic-consent-evidence"
        }]);
        manifest["household_data_access_requests"] = json!([{
            "request_id":"request-before-grant",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"water-service-planning",
            "service_id":"water-access",
            "decision_as_of_offset_s":100,
            "evidence_id":"synthetic-data-request-3"
        }]);
        assert!(has_code(
            &audit_manifest(&manifest),
            "HOUSEHOLD_DATA_ACCESS_BEFORE_GRANT"
        ));
    }

    #[test]
    fn household_data_access_must_match_the_declared_purpose() {
        let mut manifest = base();
        manifest["consent_records"] = json!([{
            "consent_id":"consent-a1",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"water-service-planning",
            "service_id":"water-access",
            "consent_state":"GRANTED",
            "consent_revision":"consent-r1",
            "granted_at_offset_s":0,
            "revoked_at_offset_s":null,
            "consent_record_evidence_id":"synthetic-consent-evidence"
        }]);
        manifest["household_data_access_requests"] = json!([{
            "request_id":"request-wrong-purpose",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"insurance-profiling",
            "service_id":"water-access",
            "decision_as_of_offset_s":100,
            "evidence_id":"synthetic-data-request-4"
        }]);
        assert!(has_code(
            &audit_manifest(&manifest),
            "HOUSEHOLD_DATA_CONSENT_MISSING_OR_AMBIGUOUS"
        ));
    }

    #[test]
    fn ambiguous_overlapping_consent_records_fail_closed() {
        let mut manifest = base();
        let consent = json!({
            "consent_id":"consent-a1",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"water-service-planning",
            "service_id":"water-access",
            "consent_state":"GRANTED",
            "consent_revision":"consent-r1",
            "granted_at_offset_s":0,
            "revoked_at_offset_s":null,
            "consent_record_evidence_id":"synthetic-consent-evidence"
        });
        manifest["consent_records"] = json!([
            consent,
            {
                "consent_id":"consent-a2",
                "household_id":"synthetic-household-a",
                "data_category":"occupancy_schedule",
                "purpose":"water-service-planning",
                "service_id":"water-access",
                "consent_state":"GRANTED",
                "consent_revision":"consent-r2",
                "granted_at_offset_s":0,
                "revoked_at_offset_s":null,
                "consent_record_evidence_id":"synthetic-consent-evidence-2"
            }
        ]);
        manifest["household_data_access_requests"] = json!([{
            "request_id":"request-ambiguous-consent",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"water-service-planning",
            "service_id":"water-access",
            "decision_as_of_offset_s":100,
            "evidence_id":"synthetic-data-request-7"
        }]);
        assert!(has_code(
            &audit_manifest(&manifest),
            "HOUSEHOLD_DATA_CONSENT_MISSING_OR_AMBIGUOUS"
        ));
    }

    #[test]
    fn household_data_consent_must_reference_registered_service() {
        let mut manifest = base();
        manifest["consent_records"] = json!([{
            "consent_id":"consent-a1",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"water-service-planning",
            "service_id":"unregistered-service",
            "consent_state":"GRANTED",
            "consent_revision":"consent-r1",
            "granted_at_offset_s":0,
            "revoked_at_offset_s":null,
            "consent_record_evidence_id":"synthetic-consent-evidence"
        }]);
        assert!(has_code(
            &audit_manifest(&manifest),
            "CONSENT_SERVICE_UNKNOWN"
        ));
    }

    #[test]
    fn household_data_access_must_match_service_scope() {
        let mut manifest = base();
        manifest["consent_records"] = json!([{
            "consent_id":"consent-a1",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"water-service-planning",
            "service_id":"energy-essential",
            "consent_state":"GRANTED",
            "consent_revision":"consent-r1",
            "granted_at_offset_s":0,
            "revoked_at_offset_s":null,
            "consent_record_evidence_id":"synthetic-consent-evidence"
        }]);
        manifest["household_data_access_requests"] = json!([{
            "request_id":"request-wrong-service",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"water-service-planning",
            "service_id":"water-access",
            "decision_as_of_offset_s":100,
            "evidence_id":"synthetic-data-request-6"
        }]);
        assert!(has_code(
            &audit_manifest(&manifest),
            "HOUSEHOLD_DATA_CONSENT_MISSING_OR_AMBIGUOUS"
        ));
    }

    #[test]
    fn consent_revocation_must_follow_the_grant_time() {
        let mut manifest = base();
        manifest["consent_records"] = json!([{
            "consent_id":"consent-a1",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"water-service-planning",
            "service_id":"water-access",
            "consent_state":"REVOKED",
            "consent_revision":"consent-r2",
            "granted_at_offset_s":200,
            "revoked_at_offset_s":100,
            "consent_record_evidence_id":"synthetic-consent-evidence"
        }]);
        assert!(has_code(
            &audit_manifest(&manifest),
            "HOUSEHOLD_DATA_CONSENT_STATE_INVALID"
        ));
    }

    #[test]
    fn no_household_data_is_shared_by_default() {
        let manifest = base();
        assert!(
            manifest["consent_records"]
                .as_array()
                .is_some_and(Vec::is_empty)
        );
        assert!(
            manifest["household_data_access_requests"]
                .as_array()
                .is_some_and(Vec::is_empty)
        );
        let report = audit_manifest(&manifest);
        assert_eq!(
            report["data_sharing_default"].as_str(),
            Some("DENY_UNLESS_MATCHING_CONSENT_VALID_AT_DECISION_TIME")
        );
    }

    #[test]
    fn household_data_access_before_revocation_remains_time_scoped() {
        let mut manifest = base();
        manifest["consent_records"] = json!([{
            "consent_id":"consent-a1",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"water-service-planning",
            "service_id":"water-access",
            "consent_state":"REVOKED",
            "consent_revision":"consent-r2",
            "granted_at_offset_s":0,
            "revoked_at_offset_s":200,
            "consent_record_evidence_id":"synthetic-consent-evidence"
        }]);
        manifest["household_data_access_requests"] = json!([{
            "request_id":"request-before-revoke",
            "household_id":"synthetic-household-a",
            "data_category":"occupancy_schedule",
            "purpose":"water-service-planning",
            "service_id":"water-access",
            "decision_as_of_offset_s":100,
            "evidence_id":"synthetic-data-request-5"
        }]);
        let report = audit_manifest(&manifest);
        assert_eq!(report["status"].as_str(), Some("PASS"));
        assert_eq!(
            report["authorized_household_data_access_requests"].as_u64(),
            Some(1)
        );
    }

    #[test]
    fn synthetic_household_outcomes_are_reported_without_aggregate_compensation() {
        let report = audit_manifest(&base());
        println!("synthetic audit errors: {}", report["errors"]);
        assert_eq!(report["status"].as_str(), Some("PASS"));
        assert_eq!(
            report["household_service_outcomes_checked"].as_u64(),
            Some(3)
        );
        assert_eq!(report["household_floor_breach_count"].as_u64(), Some(1));
        assert_eq!(
            report["neighborhood_aggregate_compensation"].as_str(),
            Some("NOT_COMPUTED")
        );
    }

    #[test]
    fn essential_household_outcome_cannot_be_omitted() {
        let mut manifest = base();
        manifest["household_service_outcomes"]
            .as_array_mut()
            .unwrap()
            .remove(2);
        assert!(has_code(
            &audit_manifest(&manifest),
            "ESSENTIAL_FLOOR_OUTCOME_MISSING"
        ));
    }

    #[test]
    fn partial_delivery_cannot_be_marked_floor_satisfied() {
        let mut manifest = base();
        manifest["household_service_outcomes"][0]["delivery_state"] = json!("PARTIAL");
        manifest["household_service_outcomes"][0]["delivered_quantity"] = json!(800);
        assert!(has_code(
            &audit_manifest(&manifest),
            "HOUSEHOLD_FLOOR_FALSE_SATISFIED"
        ));
    }

    #[test]
    fn per_household_outcome_units_must_match_service_contract() {
        let mut manifest = base();
        manifest["household_service_outcomes"][0]["unit"] = json!("m3/day");
        assert!(has_code(
            &audit_manifest(&manifest),
            "OUTCOME_UNIT_MISMATCH"
        ));
    }

    #[test]
    fn stress_matrix_is_versioned_unique_and_owner_routed() {
        let matrix: Value = serde_json::from_str(STRESS_MATRIX).expect("valid stress matrix JSON");
        assert_eq!(
            matrix["benchmark_id"].as_str(),
            Some("HIGHVELD-NEIGHBORHOOD-COMMONS-STRESS-MATRIX")
        );
        assert_eq!(matrix["maturity"].as_str(), Some("PROPOSED_NOT_EVALUATED"));
        let cases = matrix["scenario_families"]
            .as_array()
            .expect("scenario array");
        assert!(cases.len() >= 15);
        let mut ids = BTreeSet::new();
        for case in cases {
            let id = case["scenario_id"].as_str().expect("scenario id");
            assert!(ids.insert(id.to_owned()), "duplicate scenario id: {id}");
            assert!(matches!(
                case["owner_system"].as_str(),
                Some("SOL_ATLAS" | "SYMTROPY" | "MYCELIX" | "SPECIALIST_SOLVER")
            ));
            assert!(
                case["required_observations"]
                    .as_array()
                    .is_some_and(|items| !items.is_empty())
            );
        }
    }

    #[test]
    fn synthetic_reference_manifest_passes_without_claiming_evaluation() {
        let report = audit_manifest(&base());
        assert_eq!(report["status"].as_str(), Some("PASS"));
        assert_eq!(report["maturity"].as_str(), Some("PROPOSED_NOT_EVALUATED"));
    }

    #[test]
    fn essential_service_cannot_be_hub_only() {
        let mut manifest = base();
        manifest["households"][0]["essential_service_floors"][0]["local_fallback"]["depends_on_commons"] =
            json!(true);
        assert!(has_code(
            &audit_manifest(&manifest),
            "LOCAL_FALLBACK_DEPENDS_ON_COMMONS"
        ));
    }

    #[test]
    fn commons_asset_cannot_be_declared_as_household_fallback() {
        let mut manifest = base();
        manifest["households"][0]["essential_service_floors"][0]["local_fallback"]["asset_id"] =
            json!("commons-water-hub");
        assert!(has_code(
            &audit_manifest(&manifest),
            "FALLBACK_IS_COMMONS_ASSET"
        ));
    }

    #[test]
    fn fallback_must_be_in_a_distinct_failure_domain() {
        let mut manifest = base();
        manifest["households"][0]["essential_service_floors"][0]["local_fallback"]["failure_domain_id"] =
            json!("fd-commons-water");
        manifest["assets"][1]["failure_domain_id"] = json!("fd-commons-water");
        assert!(has_code(
            &audit_manifest(&manifest),
            "LOCAL_FALLBACK_COMMON_CAUSE"
        ));
    }

    #[test]
    fn local_fallback_must_survive_network_isolation() {
        let mut manifest = base();
        manifest["households"][0]["essential_service_floors"][0]["local_fallback"]["depends_on_network"] =
            json!(true);
        assert!(has_code(
            &audit_manifest(&manifest),
            "LOCAL_FALLBACK_DEPENDS_ON_NETWORK"
        ));
    }

    #[test]
    fn local_fallback_must_have_manual_degraded_mode() {
        let mut manifest = base();
        manifest["households"][0]["essential_service_floors"][0]["local_fallback"]["manual_mode_available"] =
            json!(false);
        assert!(has_code(
            &audit_manifest(&manifest),
            "LOCAL_FALLBACK_NO_MANUAL_MODE"
        ));
    }

    #[test]
    fn allocated_commons_service_requires_maintenance_contract_reference() {
        let mut manifest = base();
        manifest["service_contracts"][0]["maintenance_contract_id"] =
            json!("missing-maintenance-contract");
        assert!(has_code(
            &audit_manifest(&manifest),
            "MAINTENANCE_CONTRACT_NOT_QUALIFIED"
        ));
    }

    #[test]
    fn allocated_commons_service_requires_spares_and_repair_contract() {
        let mut manifest = base();
        manifest["maintenance_contracts"][0]["spares_state"] = json!("UNKNOWN");
        assert!(has_code(
            &audit_manifest(&manifest),
            "MAINTENANCE_CONTRACT_NOT_QUALIFIED"
        ));
    }

    #[test]
    fn maintenance_contract_must_bind_the_actual_provider_asset() {
        let mut manifest = base();
        manifest["maintenance_contracts"][0]["asset_id"] = json!("commons-energy-hub");
        assert!(has_code(
            &audit_manifest(&manifest),
            "MAINTENANCE_CONTRACT_NOT_QUALIFIED"
        ));
    }

    #[test]
    fn shared_allocation_requires_approved_human_governance_and_appeal() {
        let mut manifest = base();
        manifest["allocation_policies"][0]["final_authority"] = json!("AUTOMATED");
        assert!(has_code(
            &audit_manifest(&manifest),
            "ALLOCATION_GOVERNANCE_NOT_QUALIFIED"
        ));
    }

    #[test]
    fn shared_access_must_be_qualified_before_allocation() {
        let mut manifest = base();
        manifest["service_contracts"][0]["access_path"]["qualification_state"] = json!("UNKNOWN");
        assert!(has_code(
            &audit_manifest(&manifest),
            "UNQUALIFIED_ACCESS_ALLOCATED"
        ));
    }

    #[test]
    fn queue_wait_cannot_exceed_household_service_floor() {
        let mut manifest = base();
        manifest["service_contracts"][0]["queue_policy"]["maximum_wait_s"] = json!(900);
        manifest["households"][0]["essential_service_floors"][0]["maximum_wait_s"] = json!(300);
        assert!(has_code(
            &audit_manifest(&manifest),
            "QUEUE_EXCEEDS_RESIDENT_FLOOR"
        ));
    }

    #[test]
    fn unmodelled_capacity_overcommit_is_rejected() {
        let mut manifest = base();
        manifest["service_contracts"][0]["capacity_per_window"] = json!(10);
        manifest["service_contracts"][0]["peak_demand_per_window"] = json!(100);
        manifest["service_contracts"][0]["queue_policy"]["enabled"] = json!(false);
        assert!(has_code(
            &audit_manifest(&manifest),
            "CAPACITY_OVERCOMMIT_UNMODELED"
        ));
    }

    #[test]
    fn resource_reserve_cannot_be_spent_by_the_commons() {
        let mut manifest = base();
        manifest["resource_accounts"][0]["committed_outflow_quantity"] = json!(14000);
        assert!(has_code(
            &audit_manifest(&manifest),
            "RESOURCE_RESERVE_BREACH"
        ));
    }

    #[test]
    fn duplicate_meter_or_notice_event_cannot_be_counted_twice() {
        let mut manifest = base();
        let transfer = manifest["resource_transfers"][0].clone();
        manifest["resource_transfers"]
            .as_array_mut()
            .unwrap()
            .push(transfer);
        assert!(has_code(
            &audit_manifest(&manifest),
            "DUPLICATE_RESOURCE_SOURCE_EVENT"
        ));
    }

    #[test]
    fn unit_mismatch_is_never_silently_converted() {
        let mut manifest = base();
        manifest["resource_transfers"][0]["unit"] = json!("kWh");
        assert!(has_code(
            &audit_manifest(&manifest),
            "RESOURCE_TRANSFER_UNIT_MISMATCH"
        ));
    }

    #[test]
    fn access_time_cannot_exceed_resident_declared_limit() {
        let mut manifest = base();
        manifest["service_contracts"][0]["access_path"]["maximum_access_time_s"] = json!(900);
        manifest["households"][0]["essential_service_floors"][0]["maximum_access_time_s"] =
            json!(120);
        assert!(has_code(
            &audit_manifest(&manifest),
            "ACCESS_EXCEEDS_RESIDENT_FLOOR"
        ));
    }

    #[test]
    fn pending_allocation_policy_cannot_authorize_distribution() {
        let mut manifest = base();
        manifest["allocation_policies"][0]["approval_state"] = json!("PENDING");
        assert!(has_code(
            &audit_manifest(&manifest),
            "ALLOCATION_GOVERNANCE_NOT_QUALIFIED"
        ));
    }

    #[test]
    fn maturity_cannot_be_promoted_by_editing_the_manifest() {
        let mut manifest = base();
        manifest["maturity"] = json!("EVALUATED");
        assert!(has_code(
            &audit_manifest(&manifest),
            "MATURITY_CLAIM_ESCALATION"
        ));
    }
}
