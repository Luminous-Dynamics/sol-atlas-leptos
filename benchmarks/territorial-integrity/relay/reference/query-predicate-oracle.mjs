import assert from "node:assert/strict";
import fs from "node:fs";
import {
  canonicalJcs,
  queryPredicateCommitment,
} from "./private-coverage-oracle.mjs";

const vectorsUrl = new URL("../../egress/query-predicate-commitments-v0.1.json", import.meta.url);
const vectors = JSON.parse(fs.readFileSync(vectorsUrl, "utf8"));
const coverageUrl = new URL("../../egress/private-coverage-cases-v0.1.json", import.meta.url);
const coverage = JSON.parse(fs.readFileSync(coverageUrl, "utf8"));

assert.equal(vectors.cases.length, 4);
const seen = new Set();
for (const testCase of vectors.cases) {
  assert.ok(!seen.has(testCase.case_id), "duplicate query vector " + testCase.case_id);
  seen.add(testCase.case_id);
  const preimage = {
    profile_ref: testCase.profile_ref ?? vectors.profile_ref,
    predicate: testCase.predicate,
  };
  const actualCanonical = canonicalJcs(preimage);
  assert.equal(
    actualCanonical,
    testCase.canonical_preimage,
    testCase.case_id + ": JCS canonical bytes differ",
  );
  const actualDigest = queryPredicateCommitment(preimage.profile_ref, testCase.predicate);
  assert.equal(
    actualDigest,
    testCase.expected_digest,
    testCase.case_id + ": domain-separated SHA-256 differs",
  );
}

const baseline = coverage.baseline;
const expected = queryPredicateCommitment(
  baseline.query_binding.query_profile_ref,
  baseline.query_binding.predicate,
);
assert.equal(baseline.query_binding.query_commitment.value, expected);
assert.equal(baseline.receipt_binding.query_commitment.value, expected);
assert.equal(baseline.public_verifier_view.query_commitment.value, expected);

assert.throws(
  () => queryPredicateCommitment(
    "urn:luminous:cter:predicate-json-v0.2",
    baseline.query_binding.predicate,
  ),
  /query_profile_unsupported_for_commitment/,
);
assert.throws(
  () => canonicalJcs({ value: 9007199254740992 }),
  /jcs_unsafe_integer/,
);

const changed = structuredClone(baseline.query_binding.predicate);
changed.args[1].value = "US";
assert.notEqual(
  queryPredicateCommitment(baseline.query_binding.query_profile_ref, changed),
  expected,
  "predicate substitution must change the canonical commitment",
);

console.log("CTER query predicate commitment oracle: 4/4 JCS golden vectors matched.");
console.log("Profile binding, private query/receipt/public projection binding and substitution negative control matched.");
console.log("No upstream receipt authentication or production authorization is asserted.");
