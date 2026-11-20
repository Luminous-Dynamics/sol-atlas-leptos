import assert from "node:assert/strict";
import fs from "node:fs";

const corpusUrl = new URL("../../privacy/cases-v0.1.json", import.meta.url);
const corpus = JSON.parse(fs.readFileSync(corpusUrl, "utf8"));

function canonical(value) {
  if (Array.isArray(value)) return "[" + value.map(canonical).join(",") + "]";
  if (value && typeof value === "object") {
    return "{" + Object.keys(value).sort().map(
      (key) => JSON.stringify(key) + ":" + canonical(value[key]),
    ).join(",") + "}";
  }
  return JSON.stringify(value);
}

function findCanary(value, canaries, path = "$", hits = []) {
  if (typeof value === "string") {
    for (const canary of canaries) {
      if (value.includes(canary)) hits.push({ path, canary });
    }
  } else if (Array.isArray(value)) {
    value.forEach((entry, index) => findCanary(entry, canaries, path + "[" + index + "]", hits));
  } else if (value && typeof value === "object") {
    for (const [key, entry] of Object.entries(value)) {
      findCanary(entry, canaries, path + "." + key, hits);
    }
  }
  return hits;
}

export function classifyPairedWorldCase(testCase, declaredChannels) {
  const a = testCase.world_a?.observables ?? {};
  const b = testCase.world_b?.observables ?? {};
  const observed = new Set([...Object.keys(a), ...Object.keys(b)]);
  const undeclared = [...observed].filter((channel) => !declaredChannels.has(channel)).sort();
  if (undeclared.length) {
    return { disposition: "inventory_incomplete", undeclared_channels: undeclared, canary_hits: [] };
  }
  const canaries = testCase.canaries ?? [];
  const canaryHits = [
    ...findCanary(a, canaries, "world_a"),
    ...findCanary(b, canaries, "world_b"),
  ];
  const differentChannels = [...observed]
    .filter((channel) => canonical(a[channel]) !== canonical(b[channel]))
    .sort();
  const disposition = canaryHits.length || differentChannels.length
    ? "leak_detected"
    : "non_disclosure";
  return {
    disposition,
    different_channels: differentChannels,
    canary_hits: canaryHits,
    undeclared_channels: [],
  };
}

const declared = new Set(corpus.declared_channels);
const ids = new Set();
const counts = { leak_detected: 0, non_disclosure: 0, inventory_incomplete: 0 };
const results = [];
for (const testCase of corpus.cases) {
  assert.ok(!ids.has(testCase.case_id), "duplicate case id: " + testCase.case_id);
  ids.add(testCase.case_id);
  const actual = classifyPairedWorldCase(testCase, declared);
  assert.equal(
    actual.disposition,
    testCase.expected,
    testCase.case_id + ": " + JSON.stringify(actual),
  );
  counts[actual.disposition] += 1;
  results.push({ case_id: testCase.case_id, disposition: actual.disposition });
}

assert.equal(corpus.synthetic_only, true);
assert.equal(corpus.cases.length, 24);
assert.deepEqual(counts, {
  leak_detected: 20,
  non_disclosure: 3,
  inventory_incomplete: 1,
});
console.log("CTER privacy oracle: " + results.length + "/" + results.length + " paired-world fixture dispositions matched.");
console.log("Counts: " + JSON.stringify(counts) + ".");
console.log("Fixture semantics only; no production privacy guarantee is asserted.");
