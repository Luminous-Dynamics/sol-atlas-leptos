# IEAC domain outcome-reconciliation envelope v0.1

**Status:** Domain-neutral binding contract and synthetic fixture. It does not estimate causal effects or validate domain-specific measurement truth.  
**Tracks:** [IEAC-OUTCOME-001 / #389](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/389).  
**Composes with:** SCITT decision history #374/#375, event-history audit #376/#382, and domain-owned outcome receipts such as workforce outcome evaluation #276/#277.

## 1. Purpose

IEAC needs to preserve the full chain from decision to real-world outcome without inventing one universal metric system:

`decision → authorized intervention → execution evidence → domain outcome receipt → reconciliation → causal-claim ceiling`

This profile binds domain-owned receipts to the decision and intervention that they purport to follow. It makes the expected-versus-observed window, population, denominator, uncertainty, missingness and interpretation level visible to relying parties.

The envelope deliberately does not recompute a workforce employment estimate, energy reliability metric, water quality metric or manufacturing yield. The owning domain retains the semantics and validator for its measurements. IEAC binds the exact receipt digest and exposes the permitted summary.

## 2. Reuse existing domain owners

The workforce outcome protocol in #276/#277 is the reference integration:
- it separates diagnosis, action, completion, independently verified restoration and paid-work evidence;
- it preserves denominators and missing outcomes;
- it separates gross paid-work events from net additional paid hours;
- it leaves displacement, substitution, deadweight and spillovers as `not_assessed` until evidence justifies more;
- it refuses to call synthetic outcomes field impact.

This envelope references the workforce receipt by exact digest. It does not translate gross paid-work events into jobs created or re-estimate additionality.

The same pattern applies to energy, telecom, water, built environment, port-city and manufacturing receipts. Each domain owns its own metric definitions, uncertainty models, validation and claim limitations.

## 3. Contract

The strict Draft 2020-12 schema is `schemas/industry-outcome-reconciliation-v0.1.schema.json`. The fixture is `schemas/examples/ieac-outcome-reconciliation-synthetic-v0.1.json`.

### 3.1 Decision and intervention binding

The top-level record binds:
- IEAC decision case and case-scoped subject pseudonym;
- exact SCITT decision statement digest and IEAC profile digest;
- intervention/action digest;
- human/organizational authorization receipt digest;
- execution receipt digest, or explicit absence if the action was not executed;
- domain-owned outcome receipt digests and their schema/metric-contract identities.

An outcome receipt that belongs to a different decision, subject, action or profile must not be joined merely because it mentions the same asset. Runtime graph verification must enforce those references.

### 3.2 Domain receipts and metrics

Each domain receipt is referenced by its exact digest, media type, schema version, domain and metric-contract digest. The envelope's `metrics` list identifies which referenced receipt supplies each metric.

For every metric, preserve:
- metric ID and unit;
- expected measurement/range;
- observed measurement/range;
- uncertainty interval where supplied;
- population/denominator and time window;
- missingness/censoring/dispute status;
- reconciliation status and interpretation class.

No generic sum or weighted total across incompatible units is defined. A metric comparison is valid only when the expected and observed values share the same metric contract, units, population scope and comparable measurement window. Runtime must verify this, not infer it from an identical label.

### 3.3 Interpretation and causality are separate

Each metric declares one of:
- `observation_only`: a source reports a measurement or count;
- `before_after_difference`: two values differ over time, without asserting why;
- `descriptive_association`: variables co-vary under the observed design;
- `causal_effect_estimate`: a declared causal estimand supported by a preregistered design and independently reviewed analysis;
- `not_identified`: the available evidence does not identify the effect.

The top-level `claim_ceiling` caps what the combined receipt may claim. The schema enforces a monotonic ceiling across every nested metric: an observation-only receipt cannot contain a metric labelled as a before/after, descriptive-association or causal estimate; a descriptive-association receipt cannot contain a causal metric. It also requires a causal assessment when the top-level ceiling is causal, requires a metric to carry the causal interpretation, and forbids synthetic or rights-unknown/revoked inputs from claiming causal effect.

A before/after difference is **not** a causal estimate. A randomized or quasi-experimental design is not automatically valid merely because its name is present. Domain-specific analysis must evaluate assumptions, diagnostics, interference, selection, missingness, measurement validity and sensitivity to unmeasured confounding.

The causal-assessment object binds:
- design and estimand;
- treatment and comparison definitions;
- preregistration digest;
- analysis-code digest;
- effect estimate and uncertainty interval;
- assumptions/diagnostics;
- independent-review confirmation.

The schema checks shape only. It does not establish that an identification assumption holds, that an interval is calibrated, or that the reviewer is actually independent.

## 4. Missingness, rights and privacy

- Missing, delayed, censored, disputed, partially missing or unavailable outcomes remain explicit. Null expected/observed values and any non-complete missingness state cannot be reconciled as within/outside target; they remain indeterminate, not comparable or not observed. They cannot be converted into zero or successful completion.
- Rights unknown or revoked block causal-effect claims. A rights status of restricted requires a separate permitted-use check; this envelope does not grant disclosure.
- Raw measurements, worker data, PII, confidential source payloads and salt/preimage material remain in domain-controlled stores.
- SCITT records only approved metadata and the digest of this envelope/domain evidence.
- The envelope's `action_authority = none`; outcome evidence cannot retroactively authorize an intervention or override a safety/rights gate.

## 5. Synthetic fixture

The fixture binds two synthetic domain receipts to one intervention:
1. an energy service restoration metric, expected at 45 minutes and observed at 52 minutes with a synthetic uncertainty interval;
2. a workforce gross-paid-work count of four, referenced from the existing workforce outcome contract.

Its claim ceiling remains `observation_only`. The fixture explicitly states that no comparison group or confounder assessment exists and that the workforce event count does not identify net additional employment. The values and artifact digests are synthetic labels, not field data or cryptographic positive vectors.

## 6. Negative controls and runtime invariants

Runtime integration must reject or downgrade:
- domain receipt from wrong case, subject, action, authorization or profile;
- metric pointing to an absent or duplicate domain receipt ID;
- duplicate metric ID, changed metric-contract digest or incompatible units/population/windows;
- expected/observed intervals with inverted bounds or incompatible definitions;
- missing/censored/unavailable outcome represented as zero or as within-target;
- completed execution with no execution receipt;
- causal estimate without preregistration, treatment/comparison definition, analysis digest, interval, diagnostics and independent review;
- before/after change upgraded to causal effect;
- source rights unknown/revoked or the evidence window invalidated by correction/expiry;
- domain receipt digest mismatch or superseded receipt treated as current without an appended supersession event;
- raw sensitive outcome values exposed in public SCITT event payload.

Cross-field checks required beyond JSON Schema:
- unique `domain_receipts.receipt_id` and unique metric IDs;
- every `metrics[].domain_receipt_id` resolves exactly once;
- each metric contract digest matches the referenced receipt's contract;
- `observation_window.start < end`;
- each interval has lower <= upper;
- source measurement date, expected-value basis and observation period are comparable;
- the action and authorization are bound to the same decision event;
- metric claim ceilings do not exceed domain receipt claim ceilings.

## 7. Benchmarks and promotion

The initial corpus should cover:
- one observation-only telemetry case;
- a before/after service change with known concurrent demand/maintenance shift;
- a randomized comparison with a planted known effect;
- stepped-wedge rollout with cluster/time dependence;
- matched comparison with planted unmeasured confounding;
- difference-in-differences with valid and violated parallel-trends assumptions;
- missing-not-at-random, censoring, dropout, displacement and spillover;
- superseded/corrected outcome receipt and delayed source arrival.

Promotion targets:
- zero accepted wrong-case/action/authorization/metric-contract bindings;
- zero causal classifications from observation-only and bare before/after cases;
- 100% explicit not-identified/indeterminate for planted non-identifiable or violated-assumption cases;
- calibrated interval coverage and bias/RMSE/type-I error measured on known synthetic data-generating processes;
- 100% preservation of missingness and denominators;
- no cross-domain unit summation or scalar impact score.

Report estimator quality only for the matching estimand/population. Report p50/p95/p99 reconciliation latency, artifact bytes, CPU/memory and source-data cost separately. Synthetic correctness is conformance, not field impact or SOTA proof.

## 8. Claim ceiling

This profile is a binding envelope, not an impact estimator. It does not prove source truth, physical safety, model validity, service benefit, net employment, causal impact or action authority. Those remain separate domain-specific claims backed by authorized observations, credible comparison designs, uncertainty and independent review.
