import assert from "node:assert/strict";
import fs from "node:fs";

const corpusUrl = new URL("../../egress/cases-v0.1.json", import.meta.url);
const corpus = JSON.parse(fs.readFileSync(corpusUrl, "utf8"));

const LEVEL = { none: 0, incomplete: 1, unknown: 2, review: 3, deny: 4 };

function nonblank(value, key) {
  const candidate = value?.[key];
  return typeof candidate === "string" && candidate.trim().length > 0
    ? candidate
    : null;
}

function isHex64(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function isSha256Digest(value) {
  return nonblank(value, "algorithm") === "sha-256"
    && isHex64(nonblank(value, "value"));
}

function stringSet(value) {
  if (!Array.isArray(value) || value.length === 0) return null;
  const values = value.map((item) => typeof item === "string" ? item.trim() : "");
  if (values.some((item) => !item) || new Set(values).size !== values.length) return null;
  return new Set(values);
}

function dataClassSet(value) {
  const values = stringSet(value);
  const allowed = new Set(["public", "community", "restricted", "culturally_protected"]);
  if (!values || [...values].some((value) => !allowed.has(value))) return null;
  return values;
}

function detectCycle(graph) {
  // Kahn's algorithm is iterative, so long lineage chains do not consume
  // the JavaScript call stack in scale-oriented corpus runs.
  const indegree = new Map();
  for (const [node, children] of graph) {
    if (!indegree.has(node)) indegree.set(node, 0);
    for (const child of children) indegree.set(child, (indegree.get(child) ?? 0) + 1);
  }
  const ready = [...indegree.entries()]
    .filter(([, degree]) => degree === 0)
    .map(([node]) => node);
  let processed = 0;
  while (ready.length) {
    const node = ready.pop();
    processed += 1;
    for (const child of graph.get(node) ?? []) {
      const degree = indegree.get(child) - 1;
      indegree.set(child, degree);
      if (degree === 0) ready.push(child);
    }
  }
  return processed !== indegree.size;
}

export function evaluateDisclosureManifest(input) {
  let severity = LEVEL.none;
  const reasons = new Set();
  const raise = (level, code) => {
    severity = Math.max(severity, LEVEL[level]);
    reasons.add(code);
  };

  if (nonblank(input, "schema") !== "luminous.cter.disclosure-manifest.v0.1") {
    raise("incomplete", "manifest_schema_missing_or_unsupported");
  }
  if (nonblank(input, "canonicalization_profile") !== "rfc8785-jcs-v1") {
    raise("incomplete", "canonicalization_profile_missing_or_unsupported");
  }
  for (const key of ["manifest_id", "coverage_record_ref", "prepared_at"]) {
    if (!nonblank(input, key)) raise("incomplete", "manifest_" + key + "_missing");
  }
  if (!isSha256Digest(input?.manifest_commitment)) {
    raise("incomplete", "manifest_commitment_malformed");
  }

  const binding = input?.authorization_binding;
  if (!binding || typeof binding !== "object") {
    raise("incomplete", "authorization_binding_missing");
    return result();
  }
  for (const key of [
    "request_ref", "authorization_receipt_ref", "principal_ref", "subject_ref",
    "operation", "resource_ref", "evidence_frontier_ref", "coverage_query_ref", "coverage_query_profile_ref", "purpose_ref",
    "recipient_ref", "jurisdiction_profile_ref",
  ]) {
    if (!nonblank(binding, key)) raise("incomplete", "authorization_" + key + "_missing");
  }
  if (!isSha256Digest(binding.request_commitment)) {
    raise("incomplete", "authorization_request_commitment_malformed");
  }
  if (!isSha256Digest(binding.coverage_query_commitment)) {
    raise("incomplete", "authorization_coverage_query_commitment_malformed");
  }
  if (nonblank(binding, "coverage_query_profile_ref") !== "urn:luminous:cter:predicate-json-v0.1") {
    raise("incomplete", "authorization_coverage_query_profile_missing_or_unsupported");
  }
  if (!isHex64(nonblank(binding, "project_snapshot_sha256"))) {
    raise("incomplete", "authorization_project_snapshot_malformed");
  }
  if (!["collect_measurement", "publish_observation", "export_evidence"].includes(binding.operation)) {
    raise("deny", "operation_outside_cter_disclosure_boundary");
  }
  const authorizedClasses = dataClassSet(binding.data_classes);
  if (!authorizedClasses) raise("incomplete", "authorization_data_classes_missing_or_invalid");

  const destination = input?.destination_binding;
  if (!destination || typeof destination !== "object") {
    raise("incomplete", "destination_binding_missing");
    return result();
  }
  for (const key of ["recipient_ref", "destination_ref", "destination_key_epoch_ref", "channel_ref", "transport_profile_ref"]) {
    if (!nonblank(destination, key)) raise("incomplete", "destination_" + key + "_missing");
  }
  if (nonblank(destination, "recipient_ref") !== nonblank(binding, "recipient_ref")) {
    raise("deny", "recipient_binding_mismatch");
  }

  const packageValue = input?.package;
  if (!packageValue || typeof packageValue !== "object") {
    raise("incomplete", "package_missing");
    return result();
  }
  if (!nonblank(packageValue, "package_ref") || !nonblank(packageValue, "media_type")) {
    raise("incomplete", "package_identity_or_media_type_missing");
  }
  if (!isSha256Digest(packageValue.package_payload_digest)) {
    raise("incomplete", "package_payload_digest_malformed");
  }
  const outputs = packageValue.outputs;
  const transformations = packageValue.transformations;
  if (!Array.isArray(outputs)) {
    raise("incomplete", "package_outputs_missing");
    return result();
  }
  if (!Array.isArray(transformations)) {
    raise("incomplete", "package_transformations_missing");
    return result();
  }
  if (outputs.length === 0 || transformations.length === 0) {
    raise("incomplete", "package_outputs_or_transformations_empty");
    return result();
  }

  const outputIds = new Set();
  const outputClasses = new Map();
  const outputDigests = new Map();
  const sourceDigests = new Map();
  const sourceClasses = new Map();
  const outputTransformRefs = new Map();
  const outputSourceRefs = new Map();

  for (const output of outputs) {
    const objectRef = nonblank(output, "object_ref");
    if (!objectRef) {
      raise("incomplete", "output_object_ref_missing");
      continue;
    }
    if (outputIds.has(objectRef)) raise("deny", "output_object_identity_duplicated");
    outputIds.add(objectRef);
    if (!isSha256Digest(output.content_digest)) raise("incomplete", "output_content_digest_malformed");
    outputDigests.set(objectRef, nonblank(output.content_digest, "value") ?? "");
    if (!Number.isSafeInteger(output.byte_length) || output.byte_length < 0 || !nonblank(output, "media_type")) {
      raise("incomplete", "output_length_or_media_type_invalid");
    }

    const classes = dataClassSet(output.data_classes);
    if (!classes) raise("incomplete", "output_data_classes_missing_or_invalid");
    if (classes && authorizedClasses && [...classes].some((value) => !authorizedClasses.has(value))) {
      raise("deny", "output_data_class_outside_authorized_scope");
    }
    outputClasses.set(objectRef, classes ?? new Set());

    if (!Array.isArray(output.source_refs)) {
      raise("incomplete", "output_source_lineage_missing");
      continue;
    }
    if (output.source_refs.length === 0) raise("incomplete", "output_source_lineage_empty");
    const sourceIds = new Set();
    for (const source of output.source_refs) {
      const sourceRef = nonblank(source, "object_ref");
      if (!sourceRef) {
        raise("incomplete", "source_object_ref_missing");
        continue;
      }
      if (!isSha256Digest(source.content_digest)) raise("incomplete", "source_content_digest_malformed");
      const digest = nonblank(source.content_digest, "value") ?? "";
      if (sourceDigests.has(sourceRef) && sourceDigests.get(sourceRef) !== digest) {
        raise("deny", "source_object_identity_conflict");
      }
      sourceDigests.set(sourceRef, digest);
      const sourceClassSet = dataClassSet(source.data_classes);
      if (!sourceClassSet) raise("incomplete", "source_data_classes_missing_or_invalid");
      if (sourceClasses.has(sourceRef) && JSON.stringify([...sourceClasses.get(sourceRef)].sort()) !== JSON.stringify([...(sourceClassSet ?? [])].sort())) {
        raise("deny", "source_classification_identity_conflict");
      }
      sourceClasses.set(sourceRef, sourceClassSet ?? new Set());
      if (sourceIds.has(sourceRef)) raise("deny", "source_reference_duplicated");
      sourceIds.add(sourceRef);
    }
    outputSourceRefs.set(objectRef, sourceIds);

    const transformRefs = stringSet(output.transformation_refs);
    if (!transformRefs) raise("incomplete", "output_transformation_refs_missing_or_invalid");
    outputTransformRefs.set(objectRef, transformRefs ?? new Set());
  }

  const transformByRef = new Map();
  for (const transform of transformations) {
    const transformRef = nonblank(transform, "transform_ref");
    if (!transformRef) {
      raise("incomplete", "transform_ref_missing");
      continue;
    }
    if (transformByRef.has(transformRef)) raise("deny", "transform_identity_duplicated");
    transformByRef.set(transformRef, transform);
    if (!nonblank(transform, "profile_ref")) raise("incomplete", "transform_profile_ref_missing");
    if (!isSha256Digest(transform.version_commitment)) raise("incomplete", "transform_version_commitment_malformed");
    const allowedKinds = [
      "identity_copy", "redaction", "aggregation", "generalization",
      "translation", "format_conversion", "derived_summary", "other",
    ];
    if (!allowedKinds.includes(transform.transform_kind)) raise("incomplete", "transform_kind_missing_or_unknown");

    if (transform.transform_kind === "identity_copy") {
      const inputs = stringSet(transform.input_object_refs);
      const produced = stringSet(transform.output_object_refs);
      if (!inputs || !produced || inputs.size !== 1 || produced.size !== 1) {
        raise("incomplete", "identity_copy_cardinality_invalid");
      } else {
        const inputRef = [...inputs][0];
        const outputRef = [...produced][0];
        const inputDigest = sourceDigests.get(inputRef) ?? outputDigests.get(inputRef);
        const outputDigest = outputDigests.get(outputRef);
        if (inputDigest === undefined || outputDigest === undefined) {
          raise("incomplete", "identity_copy_digest_unresolved");
        } else if (inputDigest !== outputDigest) {
          raise("deny", "identity_copy_digest_mismatch");
        }
      }
    }
  }

  // An intermediate output can be a source for a downstream transform;
  // the shared object reference must still bind identical content.
  for (const [sourceRef, sourceDigest] of sourceDigests) {
    if (outputDigests.has(sourceRef) && outputDigests.get(sourceRef) !== sourceDigest) {
      raise("deny", "source_output_content_conflict");
    }
  }

  const knownObjectIds = new Set([...sourceDigests.keys(), ...outputIds]);
  const graph = new Map();
  const referencedTransforms = new Set();
  const transformInputs = new Map();
  for (const [outputRef, refs] of outputTransformRefs) {
    for (const transformRef of refs) {
      referencedTransforms.add(transformRef);
      const transform = transformByRef.get(transformRef);
      if (!transform) {
        raise("incomplete", "transformation_lineage_incomplete");
        continue;
      }
      const inputs = stringSet(transform.input_object_refs);
      const produced = stringSet(transform.output_object_refs);
      if (!inputs || !produced) {
        raise("incomplete", "transform_input_or_output_refs_invalid");
        continue;
      }
      if (!produced.has(outputRef)) raise("deny", "transform_does_not_produce_referenced_output");
      if ([...inputs].some((ref) => !knownObjectIds.has(ref))) raise("incomplete", "transform_input_object_unresolved");
      for (const inputRef of inputs) {
        if (outputIds.has(inputRef)) {
          if (!graph.has(inputRef)) graph.set(inputRef, new Set());
          graph.get(inputRef).add(outputRef);
        }
      }
      if (!transformInputs.has(outputRef)) transformInputs.set(outputRef, new Set());
      for (const inputRef of inputs) transformInputs.get(outputRef).add(inputRef);
    }
  }

  for (const [transformRef, transform] of transformByRef) {
    if (!referencedTransforms.has(transformRef)) raise("incomplete", "orphan_transformation_unreferenced");
    const produced = stringSet(transform.output_object_refs);
    if (!produced) continue;
    for (const outputRef of produced) {
      if (!outputIds.has(outputRef)) {
        raise("incomplete", "transform_output_object_unresolved");
      } else if (!outputTransformRefs.get(outputRef)?.has(transformRef)) {
        raise("incomplete", "output_omits_transform_reference");
      }
    }
  }

  for (const [outputRef, sourceRefs] of outputSourceRefs) {
    const inputs = transformInputs.get(outputRef);
    if (!inputs || inputs.size !== sourceRefs.size
        || [...inputs].some((sourceRef) => !sourceRefs.has(sourceRef))) {
      raise("incomplete", "transformation_input_source_set_mismatch");
    }
    for (const sourceRef of sourceRefs) {
      if (!sourceDigests.has(sourceRef) || !inputs?.has(sourceRef)) {
        raise("incomplete", "declared_source_not_connected_to_output_transform");
      }
    }
    const sourceClassUnion = new Set();
    for (const sourceRef of sourceRefs) {
      for (const cls of sourceClasses.get(sourceRef) ?? []) sourceClassUnion.add(cls);
    }
    const classes = outputClasses.get(outputRef);
    if (sourceClassUnion.size && classes
        && JSON.stringify([...sourceClassUnion].sort()) !== JSON.stringify([...classes].sort())) {
      const hasReference = [...(outputTransformRefs.get(outputRef) ?? [])].some((ref) =>
        nonblank(transformByRef.get(ref), "classification_change_authority_ref"));
      if (hasReference) {
        raise("unknown", "classification_change_authority_resolution_unavailable");
      } else {
        raise("review", "classification_change_authority_reference_missing");
      }
    }
  }

  if (detectCycle(graph)) raise("deny", "transformation_graph_cycle");
  return result();

  function result() {
    const disposition = Object.keys(LEVEL).find((key) => LEVEL[key] === severity);
    const labels = {
      none: "valid_structure",
      incomplete: "incomplete",
      unknown: "unknown",
      review: "review",
      deny: "deny",
    };
    return { disposition: labels[disposition], reason_codes: [...reasons].sort() };
  }
}

function test(name, mutate, expected, reason) {
  const input = structuredClone(corpus.baseline_manifest);
  mutate(input);
  const actual = evaluateDisclosureManifest(input);
  assert.equal(actual.disposition, expected, name + ": " + actual.reason_codes.join(","));
  assert.ok(actual.reason_codes.includes(reason), name + ": missing reason " + reason);
  return name;
}

const passed = [];
assert.equal(corpus.cases.length, 35, "frozen egress corpus size");
assert.equal(corpus.cases.length, 34, "frozen egress corpus size");
const baselineResult = evaluateDisclosureManifest(corpus.baseline_manifest);
assert.equal(baselineResult.disposition, "valid_structure", baselineResult.reason_codes.join(","));
passed.push("baseline");

passed.push(test(
  "recipient-binding",
  (input) => { input.destination_binding.recipient_ref = "urn:fixture:recipient:other"; },
  "deny",
  "recipient_binding_mismatch",
));
passed.push(test(
  "data-class-overreach",
  (input) => { input.package.outputs[0].data_classes = ["community", "restricted"]; },
  "deny",
  "output_data_class_outside_authorized_scope",
));
passed.push(test(
  "malformed-digest-algorithm",
  (input) => { input.package.outputs[0].content_digest.algorithm = "blake3"; },
  "incomplete",
  "output_content_digest_malformed",
));
passed.push(test(
  "missing-source",
  (input) => { input.package.outputs[0].source_refs = []; },
  "incomplete",
  "output_source_lineage_empty",
));
passed.push(test(
  "missing-transform",
  (input) => { input.package.outputs[0].transformation_refs = ["urn:fixture:transform:missing"]; },
  "incomplete",
  "transformation_lineage_incomplete",
));
passed.push(test(
  "cycle",
  (input) => { input.package.transformations[0].input_object_refs = [input.package.outputs[0].object_ref]; },
  "deny",
  "transformation_graph_cycle",
));
passed.push(test(
  "duplicate-output",
  (input) => { input.package.outputs.push(structuredClone(input.package.outputs[0])); },
  "deny",
  "output_object_identity_duplicated",
));
passed.push(test(
  "classification-change-needs-review",
  (input) => {
    input.authorization_binding.data_classes = ["community", "public"];
    input.package.outputs[0].data_classes = ["public"];
    input.package.transformations[0].transform_kind = "generalization";
  },
  "review",
  "classification_change_authority_reference_missing",
));
passed.push(test(
  "classification-reference-not-self-authenticating",
  (input) => {
    input.authorization_binding.data_classes = ["community", "public"];
    input.package.outputs[0].data_classes = ["public"];
    input.package.transformations[0].transform_kind = "generalization";
    input.package.transformations[0].classification_change_authority_ref = "urn:fixture:authority:classification-change";
  },
  "unknown",
  "classification_change_authority_resolution_unavailable",
));
passed.push(test(
  "identity-copy-digest",
  (input) => { input.package.outputs[0].content_digest.value = "b".repeat(64); },
  "deny",
  "identity_copy_digest_mismatch",
));

passed.push(test(
  "missing-coverage-query-reference",
  (input) => { delete input.authorization_binding.coverage_query_ref; },
  "incomplete",
  "authorization_coverage_query_ref_missing",
));
passed.push(test(
  "malformed-coverage-query-commitment",
  (input) => { input.authorization_binding.coverage_query_commitment.algorithm = "blake3"; },
  "incomplete",
  "authorization_coverage_query_commitment_malformed",
));
passed.push(test(
  "unsupported-coverage-query-profile",
  (input) => { input.authorization_binding.coverage_query_profile_ref = "urn:luminous:cter:predicate-json-v0.2"; },
  "incomplete",
  "authorization_coverage_query_profile_missing_or_unsupported",
));

passed.push(test(
  "duplicate-source-reference",
  (input) => { input.package.outputs[0].source_refs.push(structuredClone(input.package.outputs[0].source_refs[0])); },
  "deny",
  "source_reference_duplicated",
));

const chain = structuredClone(corpus.baseline_manifest);
const firstOutput = structuredClone(chain.package.outputs[0]);
const secondRef = "urn:fixture:output:derived-002";
chain.package.outputs.push({
  object_ref: secondRef,
  content_digest: {
    algorithm: "sha-256",
    value: "2222222222222222222222222222222222222222222222222222222222222222",
  },
  byte_length: 96,
  media_type: "application/octet-stream",
  data_classes: ["community"],
  source_refs: [{
    object_ref: firstOutput.object_ref,
    content_digest: firstOutput.content_digest,
    data_classes: ["community"],
  }],
  transformation_refs: ["urn:fixture:transform:derived-002"],
});
chain.package.transformations.push({
  transform_ref: "urn:fixture:transform:derived-002",
  transform_kind: "format_conversion",
  profile_ref: "urn:fixture:transform-profile:format-conversion-v0",
  version_commitment: {
    algorithm: "sha-256",
    value: "3333333333333333333333333333333333333333333333333333333333333333",
  },
  input_object_refs: [firstOutput.object_ref],
  output_object_refs: [secondRef],
});
const chainResult = evaluateDisclosureManifest(chain);
assert.equal(chainResult.disposition, "valid_structure", chainResult.reason_codes.join(","));
passed.push("intermediate-provenance-dag");

const hiddenSource = structuredClone(corpus.baseline_manifest);
const sourceTwo = "urn:fixture:source:synthetic-002";
const outputTwo = "urn:fixture:output:synthetic-002";
const transformTwo = "urn:fixture:transform:identity-002";
hiddenSource.package.outputs.push({
  object_ref: outputTwo,
  content_digest: {
    algorithm: "sha-256",
    value: "4444444444444444444444444444444444444444444444444444444444444444",
  },
  byte_length: 64,
  media_type: "application/octet-stream",
  data_classes: ["community"],
  source_refs: [{
    object_ref: sourceTwo,
    content_digest: {
      algorithm: "sha-256",
      value: "4444444444444444444444444444444444444444444444444444444444444444",
    },
    data_classes: ["community"],
  }],
  transformation_refs: [transformTwo],
});
hiddenSource.package.transformations.push({
  transform_ref: transformTwo,
  transform_kind: "identity_copy",
  profile_ref: "urn:fixture:transform-profile:identity-copy-v0",
  version_commitment: {
    algorithm: "sha-256",
    value: "5555555555555555555555555555555555555555555555555555555555555555",
  },
  input_object_refs: [sourceTwo],
  output_object_refs: [outputTwo],
});
hiddenSource.package.transformations[0].input_object_refs = [
  "urn:fixture:source:synthetic-001",
  outputTwo,
];
const hiddenSourceResult = evaluateDisclosureManifest(hiddenSource);
assert.equal(hiddenSourceResult.disposition, "incomplete");
assert.ok(hiddenSourceResult.reason_codes.includes("transformation_input_source_set_mismatch"));
passed.push("undeclared-transform-input");

console.log("CTER egress oracle: " + passed.length + "/" + passed.length + " local structural cases matched.");
console.log("Frozen specification corpus: " + corpus.cases.length + " entries (" + (corpus.cases.length - 1) + " adversarial specifications plus one baseline); no production authorization is asserted.");
