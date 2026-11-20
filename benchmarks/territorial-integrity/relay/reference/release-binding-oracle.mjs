import assert from "node:assert/strict";
import fs from "node:fs";
import { evaluateDisclosureManifest } from "./egress-oracle.mjs";
import { evaluatePrivateCoverage } from "./private-coverage-oracle.mjs";

const corpusUrl = new URL("../../egress/composed-release-cases-v0.1.json", import.meta.url);
const corpus = JSON.parse(fs.readFileSync(corpusUrl, "utf8"));

const LEVEL = {
  valid_composed_structure: 0,
  incomplete: 1,
  unknown: 2,
  review: 3,
  deny: 4,
};

function canonicalSet(value) {
  if (!Array.isArray(value)) return null;
  const values = value.map((item) => typeof item === "string" ? item.trim() : "");
  if (values.some((item) => !item) || new Set(values).size !== values.length) return null;
  return [...values].sort();
}

function deriveTerminalLeaves(manifest) {
  const outputs = manifest?.package?.outputs;
  if (!Array.isArray(outputs) || outputs.length === 0) return null;
  const sourceByOutput = new Map();
  for (const output of outputs) {
    const id = output?.object_ref;
    const refs = canonicalSet(output?.source_refs?.map((item) => item?.object_ref));
    if (!id || !refs || sourceByOutput.has(id)) return null;
    sourceByOutput.set(id, refs);
  }
  const pending = [...sourceByOutput.values()].flat();
  const visitedOutputs = new Set();
  const leaves = new Set();
  while (pending.length) {
    const ref = pending.pop();
    if (sourceByOutput.has(ref)) {
      if (!visitedOutputs.has(ref)) {
        visitedOutputs.add(ref);
        pending.push(...sourceByOutput.get(ref));
      }
    } else {
      leaves.add(ref);
    }
  }
  return leaves.size ? [...leaves].sort() : null;
}

export function evaluateComposedReleaseBinding(manifest, coverage) {
  const egress = evaluateDisclosureManifest(manifest);
  const partition = evaluatePrivateCoverage(coverage);
  let severity = Math.max(
    LEVEL[egress.disposition] ?? LEVEL.incomplete,
    LEVEL[partition.disposition === "valid_partition_structure" ? "valid_composed_structure" : partition.disposition] ?? LEVEL.incomplete,
  );
  const reasons = new Set([
    ...egress.reason_codes.map((reason) => "egress:" + reason),
    ...partition.reason_codes.map((reason) => "coverage:" + reason),
  ]);
  const finish = (leaves = []) => ({
    disposition: Object.keys(LEVEL).find((key) => LEVEL[key] === severity),
    reason_codes: [...reasons].sort(),
    derived_terminal_source_leaf_refs: leaves,
  });
  if (severity !== 0) return finish();

  const raise = (level, reason) => {
    severity = Math.max(severity, LEVEL[level]);
    reasons.add(reason);
  };
  if (manifest?.coverage_record_ref !== coverage?.coverage_ref) {
    raise("deny", "coverage_record_reference_mismatch");
  }
  const egressDigest = manifest?.package?.package_payload_digest;
  const coverageDigest = coverage?.package_binding?.package_commitment;
  if (JSON.stringify(egressDigest) !== JSON.stringify(coverageDigest)) {
    raise("deny", "package_payload_commitment_mismatch");
  }

  const auth = manifest?.authorization_binding;
  const query = coverage?.query_binding;
  const frontier = coverage?.frontier;
  if (!auth || !query || !frontier) {
    raise("incomplete", "cross_binding_projection_missing");
    return finish();
  }
  const fields = [
    ["request_commitment", "request_commitment"],
    ["principal_ref", "principal_ref"],
    ["subject_ref", "subject_ref"],
    ["operation", "operation"],
    ["purpose_ref", "purpose_ref"],
    ["recipient_ref", "recipient_ref"],
    ["resource_ref", "resource_ref"],
    ["jurisdiction_profile_ref", "jurisdiction_profile_ref"],
    ["project_snapshot_sha256", "project_snapshot_sha256"],
  ];
  for (const [left, right] of fields) {
    if (JSON.stringify(auth[left]) !== JSON.stringify(query[right])) {
      raise("deny", "authorization_query_binding_mismatch:" + left);
    }
  }
  if (auth.evidence_frontier_ref !== frontier.frontier_ref) {
    raise("deny", "authorization_frontier_reference_mismatch");
  }
  const leftClasses = canonicalSet(auth.data_classes);
  const rightClasses = canonicalSet(query.data_classes);
  if (!leftClasses || !rightClasses) {
    raise("incomplete", "authorization_or_coverage_data_classes_invalid");
  } else if (JSON.stringify(leftClasses) !== JSON.stringify(rightClasses)) {
    raise("deny", "authorization_data_class_set_mismatch");
  }

  const leaves = deriveTerminalLeaves(manifest);
  if (!leaves) {
    raise("incomplete", "terminal_source_leaf_derivation_failed");
  } else {
    const asserted = canonicalSet(coverage?.package_binding?.terminal_source_leaf_refs);
    if (!asserted || JSON.stringify(asserted) !== JSON.stringify(leaves)) {
      raise("deny", "derived_terminal_source_leaf_set_mismatch");
    }
  }
  return finish(leaves ?? []);
}

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function applyMutation(caseId, manifest, coverage) {
  switch (caseId) {
    case "COMPOSED-BASELINE-001": break;
    case "COMPOSED-COVERAGE-REFERENCE-DRIFT-001":
      coverage.coverage_ref = "urn:fixture:coverage:other"; break;
    case "COMPOSED-PACKAGE-DIGEST-DRIFT-001":
      coverage.package_binding.package_commitment.value = "9".repeat(64); break;
    case "COMPOSED-REQUEST-DRIFT-001":
      coverage.query_binding.request_commitment.value = "9".repeat(64); break;
    case "COMPOSED-PRINCIPAL-DRIFT-001":
      coverage.query_binding.principal_ref = "urn:fixture:principal:other"; break;
    case "COMPOSED-SUBJECT-DRIFT-001":
      coverage.query_binding.subject_ref = "urn:fixture:subject:other"; break;
    case "COMPOSED-OPERATION-DRIFT-001":
      coverage.query_binding.operation = "assert_legal_title"; break;
    case "COMPOSED-PURPOSE-DRIFT-001":
      coverage.query_binding.purpose_ref = "urn:fixture:purpose:other"; break;
    case "COMPOSED-RECIPIENT-DRIFT-001":
      coverage.query_binding.recipient_ref = "urn:fixture:recipient:other"; break;
    case "COMPOSED-RESOURCE-DRIFT-001":
      coverage.query_binding.resource_ref = "urn:fixture:resource:other"; break;
    case "COMPOSED-JURISDICTION-DRIFT-001":
      coverage.query_binding.jurisdiction_profile_ref = "urn:fixture:jurisdiction:other"; break;
    case "COMPOSED-SNAPSHOT-DRIFT-001":
      coverage.query_binding.project_snapshot_sha256 = "9".repeat(64); break;
    case "COMPOSED-FRONTIER-DRIFT-001":
      manifest.authorization_binding.evidence_frontier_ref = "urn:fixture:frontier:other"; break;
    case "COMPOSED-DATA-CLASS-DRIFT-001":
      coverage.query_binding.data_classes = ["restricted"]; break;
    case "COMPOSED-ASSERTED-LEAF-DRIFT-001":
      coverage.package_binding.terminal_source_leaf_refs = ["urn:fixture:source:other"]; break;
    case "COMPOSED-DERIVED-LEAF-DRIFT-001": {
      const replacement = "urn:fixture:source:other";
      coverage.frontier.object_refs[0] = replacement;
      coverage.private_partition[0].object_ref = replacement;
      coverage.package_binding.terminal_source_leaf_refs = [replacement];
      break;
    }
    case "COMPOSED-MULTI-STAGE-PROVENANCE-001": {
      const first = clone(manifest.package.outputs[0]);
      const firstRef = first.object_ref;
      const secondRef = "urn:fixture:output:derived-002";
      manifest.package.outputs.push({
        object_ref: secondRef,
        content_digest: {
          algorithm: "sha-256",
          value: "2222222222222222222222222222222222222222222222222222222222222222",
        },
        byte_length: 96,
        media_type: "application/octet-stream",
        data_classes: ["community"],
        source_refs: [{
          object_ref: firstRef,
          content_digest: first.content_digest,
          data_classes: ["community"],
        }],
        transformation_refs: ["urn:fixture:transform:derived-002"],
      });
      manifest.package.transformations.push({
        transform_ref: "urn:fixture:transform:derived-002",
        transform_kind: "format_conversion",
        profile_ref: "urn:fixture:transform-profile:format-conversion-v0",
        version_commitment: {
          algorithm: "sha-256",
          value: "3333333333333333333333333333333333333333333333333333333333333333",
        },
        input_object_refs: [firstRef],
        output_object_refs: [secondRef],
      });
      break;
    }
    default:
      throw new Error("unknown composed release case: " + caseId);
  }
}

const counts = { valid_composed_structure: 0, incomplete: 0, unknown: 0, review: 0, deny: 0 };
const seen = new Set();
for (const testCase of corpus.cases) {
  assert.ok(!seen.has(testCase.case_id), "duplicate case: " + testCase.case_id);
  seen.add(testCase.case_id);
  const manifest = clone(corpus.baseline.disclosure_manifest);
  const coverage = clone(corpus.baseline.private_coverage);
  applyMutation(testCase.case_id, manifest, coverage);
  const actual = evaluateComposedReleaseBinding(manifest, coverage);
  assert.equal(
    actual.disposition,
    testCase.expected,
    testCase.case_id + ": " + actual.disposition + " " + actual.reason_codes.join(","),
  );
  counts[actual.disposition] += 1;
}
assert.equal(corpus.cases.length, 17);
assert.equal(counts.valid_composed_structure, 2);
console.log("CTER composed release oracle: 17/17 synthetic fixture dispositions matched.");
console.log("Counts: " + JSON.stringify(counts) + ".");
console.log("Structural composition only; no production proof or release authorization is asserted.");
