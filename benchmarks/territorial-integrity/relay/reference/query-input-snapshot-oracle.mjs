import assert from "node:assert/strict";
import fs from "node:fs";
import { queryInputSnapshotCommitment } from "./private-coverage-oracle.mjs";

const privateUrl = new URL("../../egress/private-coverage-cases-v0.1.json", import.meta.url);
const composedUrl = new URL("../../egress/composed-release-cases-v0.1.json", import.meta.url);
const privateCorpus = JSON.parse(fs.readFileSync(privateUrl, "utf8"));
const composedCorpus = JSON.parse(fs.readFileSync(composedUrl, "utf8"));

const privateCoverage = privateCorpus.baseline;
const privateSnapshot = privateCoverage.query_input_snapshot;
const privateDigest = queryInputSnapshotCommitment(privateSnapshot);
assert.equal(privateDigest, privateSnapshot.snapshot_commitment.value);
assert.equal(privateDigest, privateCoverage.query_binding.input_snapshot_commitment.value);
assert.equal(privateDigest, privateCoverage.receipt_binding.input_snapshot_commitment.value);
assert.equal(privateSnapshot.items.length, 5);
assert.ok(privateSnapshot.items.every((item) => item.query_input
  && item.attribute_source_bindings.length > 0));
assert.ok(privateCoverage.private_partition.every((item) =>
  item.query_input_snapshot_item_ref === item.object_ref));

const composed = composedCorpus.baseline;
const composedCoverage = composed.private_coverage;
const composedSnapshot = composedCoverage.query_input_snapshot;
const composedDigest = queryInputSnapshotCommitment(composedSnapshot);
assert.equal(composedDigest, composedSnapshot.snapshot_commitment.value);
assert.equal(composedDigest, composedCoverage.query_binding.input_snapshot_commitment.value);
assert.equal(composedDigest, composedCoverage.receipt_binding.input_snapshot_commitment.value);
assert.equal(
  composed.baseline_manifest?.authorization_binding?.coverage_input_snapshot_commitment?.value
    ?? composed.disclosure_manifest?.authorization_binding?.coverage_input_snapshot_commitment?.value,
  composedDigest,
);

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

// Reordering items and attribute bindings cannot change the semantic snapshot.
const reordered = clone(privateSnapshot);
reordered.items.reverse();
for (const item of reordered.items) item.attribute_source_bindings.reverse();
assert.equal(queryInputSnapshotCommitment(reordered), privateDigest);

// Attribute changes must change the commitment.
const changed = clone(privateSnapshot);
changed.items[0].query_input.jurisdiction = "US";
assert.notEqual(queryInputSnapshotCommitment(changed), privateDigest);

// Source-reference substitution must change the commitment.
const changedSource = clone(privateSnapshot);
changedSource.items[0].attribute_source_bindings[0].source_object_ref =
  "urn:fixture:evidence:substituted-source";
assert.notEqual(queryInputSnapshotCommitment(changedSource), privateDigest);

// Profile substitution is not interpreted under the v0.1 commitment.
const changedProfile = clone(privateSnapshot);
changedProfile.profile_ref = "urn:luminous:cter:query-input-json-v0.2";
assert.throws(
  () => queryInputSnapshotCommitment(changedProfile),
  /query_input_snapshot_profile_unsupported/,
);

console.log("CTER query-input snapshot oracle: both frozen snapshot digests matched.");
console.log("Ordering invariance, attribute mutation, source substitution and profile mismatch controls matched.");
console.log("Snapshot integrity is not source-owner attestation or proof of source truth.");
