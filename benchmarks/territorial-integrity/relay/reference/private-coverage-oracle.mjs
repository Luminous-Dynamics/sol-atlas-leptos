import assert from "node:assert/strict";
import fs from "node:fs";

const corpusUrl = new URL("../../egress/private-coverage-cases-v0.1.json", import.meta.url);
const corpus = JSON.parse(fs.readFileSync(corpusUrl, "utf8"));

const LEVEL = { none: 0, incomplete: 1, unknown: 2, deny: 3 };

function nonblank(value, key) {
  const item = value?.[key];
  return typeof item === "string" && item.trim().length > 0 ? item : null;
}

function isHex64(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function isSha256(value) {
  return nonblank(value, "algorithm") === "sha-256"
    && isHex64(nonblank(value, "value"));
}

function referenceSet(value) {
  if (!Array.isArray(value)) return { error: "reference_list_missing_or_invalid" };
  const refs = new Set();
  for (const item of value) {
    if (typeof item !== "string" || !item.trim()) {
      return { error: "reference_list_contains_empty_item" };
    }
    if (refs.has(item)) return { error: "duplicate_reference" };
    refs.add(item);
  }
  return { refs };
}

function queryScalar(value) {
  return value === null || ["string", "boolean", "number"].includes(typeof value);
}

function queryMember(input, path) {
  if (typeof path !== "string" || !/^\/[A-Za-z][A-Za-z0-9_-]*$/.test(path)) {
    throw new Error("predicate_path_unsupported");
  }
  if (!input || typeof input !== "object" || Array.isArray(input)) return undefined;
  return Object.hasOwn(input, path.slice(1)) ? input[path.slice(1)] : undefined;
}

function evaluatePredicate(node, input, depth = 0) {
  if (depth > 32) throw new Error("predicate_depth_exceeded");
  if (!node || typeof node !== "object" || Array.isArray(node)) {
    throw new Error("predicate_node_missing_or_not_object");
  }
  switch (node.op) {
    case "all":
    case "any": {
      if (!Array.isArray(node.args) || node.args.length < 1 || node.args.length > 32) {
        throw new Error("predicate_args_count_out_of_range");
      }
      const results = node.args.map((arg) => evaluatePredicate(arg, input, depth + 1));
      if (node.op === "all") {
        if (results.includes("false")) return "false";
        return results.includes("unknown") ? "unknown" : "true";
      }
      if (results.includes("true")) return "true";
      return results.includes("unknown") ? "unknown" : "false";
    }
    case "not": {
      const result = evaluatePredicate(node.arg, input, depth + 1);
      return result === "unknown" ? "unknown" : (result === "true" ? "false" : "true");
    }
    case "exists": {
      if (!input || typeof input !== "object" || Array.isArray(input)) return "unknown";
      return queryMember(input, node.path) === undefined ? "false" : "true";
    }
    case "eq": {
      const actual = queryMember(input, node.path);
      if (actual === undefined) return "unknown";
      if (!Object.hasOwn(node, "value") || !queryScalar(node.value)) {
        throw new Error("predicate_eq_value_not_scalar");
      }
      return JSON.stringify(actual) === JSON.stringify(node.value) ? "true" : "false";
    }
    case "in": {
      const actual = queryMember(input, node.path);
      if (actual === undefined) return "unknown";
      if (!Array.isArray(node.values) || node.values.length < 1 || node.values.length > 64
          || !node.values.every(queryScalar)) {
        throw new Error("predicate_in_values_out_of_range_or_not_scalar");
      }
      return node.values.some((value) => JSON.stringify(value) === JSON.stringify(actual))
        ? "true" : "false";
    }
    default:
      throw new Error("predicate_operator_unsupported");
  }
}

export function evaluatePrivateCoverage(input) {
  let severity = LEVEL.none;
  const reasons = new Set();
  const raise = (level, code) => {
    severity = Math.max(severity, LEVEL[level]);
    reasons.add(code);
  };
  const finish = () => {
    const labels = {
      0: "valid_partition_structure",
      1: "incomplete",
      2: "unknown",
      3: "deny",
    };
    return { disposition: labels[severity], reason_codes: [...reasons].sort() };
  };

  if (nonblank(input, "schema") !== "luminous.cter.private-coverage.v0.1") {
    raise("incomplete", "coverage_schema_missing_or_unsupported");
  }
  if (!nonblank(input, "coverage_ref")) raise("incomplete", "coverage_reference_missing");

  const frontier = input.frontier;
  if (!frontier || typeof frontier !== "object") {
    raise("incomplete", "frontier_missing");
    return finish();
  }
  for (const field of ["frontier_ref", "version", "source_owner_ref", "receipt_ref"]) {
    if (!nonblank(frontier, field)) raise("incomplete", "frontier_" + field + "_missing");
  }
  if (!isSha256(frontier.commitment)) raise("incomplete", "frontier_commitment_malformed");
  const frontierSetResult = referenceSet(frontier.object_refs);
  let frontierRefs = new Set();
  if (frontierSetResult.error) {
    raise("deny", frontierSetResult.error);
  } else if (frontierSetResult.refs.size === 0) {
    raise("incomplete", "frontier_object_set_empty");
  } else {
    frontierRefs = frontierSetResult.refs;
  }

  const query = input.query_binding;
  if (!query || typeof query !== "object") {
    raise("incomplete", "query_binding_missing");
    return finish();
  }
  for (const field of [
    "query_ref", "query_profile_ref", "principal_ref", "subject_ref", "operation", "purpose_ref",
    "recipient_ref", "resource_ref", "jurisdiction_profile_ref",
  ]) {
    if (!nonblank(query, field)) raise("incomplete", "query_" + field + "_missing");
  }
  if (!["export_evidence", "publish_observation"].includes(query.operation)) {
    raise("deny", "coverage_operation_outside_supported_scope");
  }
  for (const field of ["query_commitment", "request_commitment"]) {
    if (!isSha256(query[field])) raise("incomplete", "query_" + field + "_malformed");
  }
  if (!isHex64(nonblank(query, "project_snapshot_sha256"))) {
    raise("incomplete", "query_project_snapshot_malformed");
  }

  const validClasses = new Set(["public", "community", "restricted", "culturally_protected"]);
  const queryClasses = referenceSet(query.data_classes);
  if (queryClasses.error || queryClasses.refs.size === 0
      || [...queryClasses.refs].some((value) => !validClasses.has(value))) {
    raise("incomplete", "query_data_classes_invalid");
  }

  if (nonblank(query, "query_profile_ref") !== "urn:luminous:cter:predicate-json-v0.1") {
    raise("incomplete", "query_profile_missing_or_unsupported");
  }
  let predicateValid = true;
  try {
    evaluatePredicate(query.predicate, {});
  } catch (error) {
    predicateValid = false;
    raise("incomplete", "query_predicate_invalid:" + error.message);
  }

  if (!Array.isArray(input.private_partition)) {
    raise("incomplete", "private_partition_missing");
    return finish();
  }
  const partitionRefs = new Set();
  const includedRefs = new Set();
  for (const item of input.private_partition) {
    const objectRef = nonblank(item, "object_ref");
    if (!objectRef) {
      raise("incomplete", "partition_object_reference_missing");
      continue;
    }
    if (partitionRefs.has(objectRef)) {
      raise("deny", "partition_object_disposition_duplicated");
    }
    partitionRefs.add(objectRef);
    const scope = nonblank(item, "scope_state");
    const disposition = nonblank(item, "disposition");
    if (predicateValid) {
      let match;
      try {
        match = evaluatePredicate(query.predicate, item?.query_input);
      } catch (error) {
        predicateValid = false;
        raise("incomplete", "query_predicate_invalid:" + error.message);
      }
      if (predicateValid) {
        const expectedScope = match === "true" ? "in_scope"
          : (match === "false" ? "out_of_scope" : "unknown");
        if (scope !== expectedScope) {
          raise("deny", "partition_scope_does_not_match_query_predicate");
        }
      }
    }
    if (scope === "in_scope" && disposition === "included") includedRefs.add(objectRef);
    else if (scope === "in_scope" && ["withheld", "unavailable", "unresolved"].includes(disposition)) {}
    else if (scope === "out_of_scope" && disposition === "excluded") {}
    else if (scope === "unknown" && disposition === "unresolved") {}
    else if (disposition === "absent") raise("deny", "partition_absence_state_forbidden");
    else if (!scope || !disposition) raise("incomplete", "partition_scope_or_disposition_missing");
    else raise("deny", "partition_scope_disposition_conflict");
  }
  if (partitionRefs.size !== frontierRefs.size
      || [...partitionRefs].some((ref) => !frontierRefs.has(ref))) {
    raise("incomplete", "partition_frontier_object_set_mismatch");
  }

  const pkg = input.package_binding;
  if (!pkg || typeof pkg !== "object") {
    raise("incomplete", "package_binding_missing");
    return finish();
  }
  if (!isSha256(pkg.package_commitment)) raise("incomplete", "package_commitment_malformed");
  const leavesResult = referenceSet(pkg.terminal_source_leaf_refs);
  let packageLeaves = new Set();
  if (leavesResult.error) raise("deny", leavesResult.error);
  else packageLeaves = leavesResult.refs;
  if (packageLeaves.size !== includedRefs.size
      || [...packageLeaves].some((ref) => !includedRefs.has(ref))) {
    raise("deny", "package_included_source_set_mismatch");
  }
  if ([...packageLeaves].some((ref) => !frontierRefs.has(ref))) {
    raise("deny", "package_contains_phantom_frontier_source");
  }

  const receipt = input.receipt_binding;
  if (!receipt || typeof receipt !== "object") {
    raise("incomplete", "receipt_binding_missing");
  } else {
    if (nonblank(receipt, "receipt_ref") !== nonblank(frontier, "receipt_ref")) {
      raise("unknown", "receipt_reference_binding_unresolved");
    }
    if (nonblank(receipt, "frontier_ref") !== nonblank(frontier, "frontier_ref")
        || nonblank(receipt, "frontier_version") !== nonblank(frontier, "version")
        || JSON.stringify(receipt.frontier_commitment) !== JSON.stringify(frontier.commitment)) {
      raise("unknown", "receipt_frontier_version_or_commitment_mismatch");
    }
    if (JSON.stringify(receipt.package_commitment) !== JSON.stringify(pkg.package_commitment)) {
      raise("deny", "receipt_package_commitment_mismatch");
    }
    for (const field of [
      "query_commitment", "request_commitment", "query_profile_ref", "principal_ref", "subject_ref", "operation",
      "purpose_ref", "recipient_ref", "resource_ref", "jurisdiction_profile_ref",
      "project_snapshot_sha256",
    ]) {
      if (JSON.stringify(receipt[field]) !== JSON.stringify(query[field])) {
        raise("deny", "receipt_query_binding_mismatch:" + field);
      }
    }
    if (!isSha256(receipt.query_commitment) || !isSha256(receipt.request_commitment)) {
      raise("incomplete", "receipt_query_or_request_commitment_malformed");
    }

    const receiptClasses = referenceSet(receipt.data_classes);
    const queryClassSet = referenceSet(query.data_classes);
    if (receiptClasses.error || queryClassSet.error
        || JSON.stringify([...receiptClasses.refs].sort()) !== JSON.stringify([...queryClassSet.refs].sort())) {
      raise("deny", "receipt_query_binding_mismatch:data_classes");
    }
  }

  const assessment = input.receipt_assessment;
  const status = nonblank(assessment, "resolution_status");
  if (status === "current") {}
  else if (status === "stale") raise("unknown", "vds_receipt_stale");
  else if (status === "unavailable") raise("unknown", "vds_receipt_unavailable");
  else if (status === "unknown" || !status) raise("unknown", "vds_receipt_currentness_unknown");
  else if (status === "revoked") raise("deny", "vds_receipt_revoked");
  else raise("incomplete", "vds_receipt_assessment_invalid");
  if (!nonblank(assessment, "assessment_ref") || !nonblank(assessment, "status_snapshot_ref")) {
    raise("incomplete", "vds_receipt_assessment_lineage_missing");
  }

  const view = input.public_verifier_view;
  if (!view || typeof view !== "object" || Array.isArray(view)) {
    raise("incomplete", "public_verifier_view_missing");
    return finish();
  }
  const allowed = new Set([
    "frontier_commitment", "query_commitment", "request_commitment",
    "package_commitment", "proof_ref", "non_inclusion_scope",
  ]);
  if (Object.keys(view).some((key) => !allowed.has(key))) {
    raise("deny", "public_verifier_view_over_discloses");
  }
  if (!nonblank(view, "proof_ref")) raise("incomplete", "public_proof_reference_missing");
  if (nonblank(view, "non_inclusion_scope") !== "frontier_version_only") {
    raise("deny", "non_inclusion_scope_exceeds_committed_frontier");
  }
  const expectedBindings = {
    frontier_commitment: frontier.commitment,
    query_commitment: query.query_commitment,
    request_commitment: query.request_commitment,
    package_commitment: pkg.package_commitment,
  };
  for (const [field, expected] of Object.entries(expectedBindings)) {
    if (JSON.stringify(view[field]) !== JSON.stringify(expected)) {
      raise("deny", "public_commitment_binding_mismatch:" + field);
    }
    if (!isSha256(view[field])) raise("incomplete", "public_commitment_malformed:" + field);
  }
  return finish();
}

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function applyMutation(caseId, input) {
  const partition = input.private_partition;
  switch (caseId) {
    case "PCOV-BASELINE-001": break;
    case "PCOV-DUPLICATE-DISPOSITION-001":
    case "PCOV-ALIAS-DUPLICATION-001":
      partition.push(clone(partition[0])); break;
    case "PCOV-OMITTED-FRONTIER-ITEM-001":
      partition.splice(1, 1); break;
    case "PCOV-WITHHELD-AS-ABSENT-001":
      partition.find(x => x.object_ref.endsWith("item-b")).disposition = "absent"; break;
    case "PCOV-UNAVAILABLE-AS-ABSENT-001":
      partition.find(x => x.object_ref.endsWith("item-c")).disposition = "absent"; break;
    case "PCOV-INCLUDED-SET-MISMATCH-001":
      partition.find(x => x.object_ref.endsWith("item-b")).disposition = "included"; break;
    case "PCOV-PHANTOM-PACKAGE-SOURCE-001":
      input.package_binding.terminal_source_leaf_refs.push("urn:fixture:evidence:phantom"); break;
    case "PCOV-FRONTIER-VERSION-DRIFT-001":
    case "PCOV-RECEIPT-FRONTIER-VERSION-DRIFT-001":
      input.frontier.version = "v2"; break;
    case "PCOV-QUERY-SUBSTITUTION-001":
      input.query_binding.query_commitment.value = "9".repeat(64); break;
    case "PCOV-RECIPIENT-SUBSTITUTION-001":
      input.query_binding.recipient_ref = "urn:fixture:recipient:other"; break;
    case "PCOV-STALE-VDS-RECEIPT-001":
      input.receipt_assessment.resolution_status = "stale"; break;
    case "PCOV-RECEIPT-UNAVAILABLE-001":
      input.receipt_assessment.resolution_status = "unavailable"; break;
    case "PCOV-RECEIPT-REVOKED-001":
      input.receipt_assessment.resolution_status = "revoked"; break;
    case "PCOV-NONINCLUSION-AS-GLOBAL-ABSENCE-001":
      input.public_verifier_view.non_inclusion_scope = "global"; break;
    case "PCOV-PUBLIC-COUNT-LEAK-001":
    case "PCOV-PUBLIC-WITHHELD-COUNT-LEAK-001":
      input.public_verifier_view.withheld_count = 1; break;
    case "PCOV-LOW-ENTROPY-HASH-LEAK-001":
      input.public_verifier_view.low_entropy_hash = "0000"; break;
    case "PCOV-LATE-SOURCE-EVENT-001":
      input.receipt_assessment.resolution_status = "unknown"; break;
    case "PCOV-RESTORE-SEMANTICS-LOSS-001":
      delete input.query_binding; break;
    case "PCOV-EXTRA-PARTITION-OBJECT-001":
      partition.push({
        object_ref: "urn:fixture:evidence:extra",
        scope_state: "in_scope",
        disposition: "withheld",
      }); break;
    case "PCOV-PUBLIC-INVENTORY-REF-LEAK-001":
      input.public_verifier_view.object_ref = "urn:fixture:evidence:item-b"; break;
    case "PCOV-OPERATION-SUBSTITUTION-001":
      input.query_binding.operation = "assert_legal_title"; break;
    case "PCOV-PURPOSE-SUBSTITUTION-001":
      input.query_binding.purpose_ref = "urn:fixture:purpose:other"; break;
    case "PCOV-JURISDICTION-SUBSTITUTION-001":
      input.query_binding.jurisdiction_profile_ref = "urn:fixture:jurisdiction:other"; break;
    case "PCOV-PUBLIC-PACKAGE-COMMITMENT-DRIFT-001":
      input.public_verifier_view.package_commitment.value = "8".repeat(64); break;
    case "PCOV-FRONTIER-ITEM-DUPLICATE-001":
      input.frontier.object_refs.push(input.frontier.object_refs[0]); break;
    case "PCOV-RECEIPT-PROJECTION-DRIFT-001":
      input.receipt_binding.recipient_ref = "urn:fixture:recipient:other"; break;
    case "PCOV-PROOF-VIEW-EXTRA-REFERENCE-001":
      input.public_verifier_view.object_refs = ["urn:fixture:evidence:item-b"]; break;
    case "PCOV-RECEIPT-SUBJECT-DRIFT-001":
      input.receipt_binding.subject_ref = "urn:fixture:subject:other"; break;
    case "PCOV-RECEIPT-DATA-CLASS-DRIFT-001":
      input.receipt_binding.data_classes = ["restricted"]; break;
    case "PCOV-PREDICATE-SCOPE-MISMATCH-001":
      input.private_partition[3].scope_state = "in_scope";
      input.private_partition[3].disposition = "withheld"; break;
    case "PCOV-UNKNOWN-PREDICATE-AS-IN-SCOPE-001":
      input.private_partition[4].scope_state = "in_scope";
      input.private_partition[4].disposition = "withheld"; break;
    case "PCOV-TRUE-PREDICATE-AS-OUT-OF-SCOPE-001":
      input.private_partition[1].scope_state = "out_of_scope";
      input.private_partition[1].disposition = "excluded"; break;
    case "PCOV-PREDICATE-UNKNOWN-OPERATOR-001":
      input.query_binding.predicate.op = "matches"; break;
    case "PCOV-PREDICATE-MALFORMED-PATH-001":
      input.query_binding.predicate.args[0].path = "dataset"; break;
    default:
      throw new Error("unknown private coverage case: " + caseId);
  }
}

const counts = { valid_partition_structure: 0, incomplete: 0, unknown: 0, deny: 0 };
const seen = new Set();
for (const testCase of corpus.cases) {
  assert.ok(!seen.has(testCase.case_id), "duplicate case id " + testCase.case_id);
  seen.add(testCase.case_id);
  const input = clone(corpus.baseline);
  applyMutation(testCase.case_id, input);
  const actual = evaluatePrivateCoverage(input);
  assert.equal(
    actual.disposition,
    testCase.expected,
    testCase.case_id + ": " + actual.disposition + " " + actual.reason_codes.join(","),
  );
  counts[actual.disposition] += 1;
}
assert.equal(corpus.cases.length, 37);
console.log("CTER private coverage oracle: " + corpus.cases.length + "/" + corpus.cases.length + " fixture dispositions matched.");
console.log("Counts: " + JSON.stringify(counts) + ".");
console.log("Partition structure only; no VDS/COSE proof, authority or global-completeness claim.");
