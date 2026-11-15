# Financial Observatory — Synthetic Supplier/Energy Dossier v1

**Status:** public synthetic development artifact  
**Programme:** [FIN-OBS-001 / Sol-Atlas #666](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/666)  
**Source task:** [Symthaea Financial Research Agent PR #11233](https://github.com/Luminous-Dynamics/symthaea/pull/11233)  
**Task ID:** dev.synthetic.supplier-energy-shock.0001  
**Decision frontier:** 2026-10-01T12:00:00Z  
**Claim ceiling:** synthetic evidence interpretation and conditional scenario design only

> This dossier is fictional and uses only the public development fixture. It is not an analysis of a real company, a financial recommendation, an estimate of a real loss, or a causal finding.

## 1. Decision question

As of 1 October 2026 at 12:00 UTC, what is known about the fictional buyer's supplier and energy context? What operational consequences remain plausible, which facts are missing, and what evidence would most change the assessment?

The available bundle contains two observations, not a complete buyer portfolio, procurement ledger, production plan, energy contract or financial statement.

## 2. Evidence frontier and source register

The task manifest declares:
- Evidence manifest SHA-256: 5e19a871eb17620ba1622de9ed26a64fd8f488b59c1389ae3d2ba0a849bbce8d
- Decision frontier: 2026-10-01T12:00:00Z
- Rights profile: synthetic-public-development
- Cohort: development_public

The digest is copied from the task manifest. This dossier does not claim to have independently reconstructed the manifest digest or verified retained artifact bytes. That remains an execution requirement for the evidence-manifest/byte-verification pipeline.

### E-SUP-01 — supplier schedule observation

| Field | Value |
|---|---|
| Evidence ID | ev.synthetic.supplier-status.0001 |
| Source ID | synthetic-source.supplier-status.v1 |
| Source record | SYN-REC-SUPPLIER-0001 |
| Subjects | SYN-BUYER-01; SYN-SUPPLIER-01 |
| Observation | shipment_schedule_disrupted |
| Reported / published | 2026-09-30T09:00:00Z |
| Known / retrieved | 2026-09-30T09:15:00Z |
| Valid from | 2026-09-30T09:00:00Z |
| Valid until | Not supplied |
| Content SHA-256 | 00267da342f0e0901070c16ceb5b0daeda51172cc3fd23dd7a60d99179501adc |
| Provenance family ID | synthetic-family.supplier.v1 |
| Buyer purchase share | **Unknown** |
| Buyer inventory buffer | **Unknown** |

**Interpretation:** the fixture supports that a shipment-schedule disruption was reported for the named supplier in the buyer context. It does not establish purchase concentration, inventory coverage, an executed contract, actual missed deliveries, alternative capacity, production interruption or financial loss.

The most recent supplied status was known 26 hours and 45 minutes before the decision frontier. There is no freshness threshold or later update. Therefore the supplier's status at the decision frontier is **not established**. The absent valid-until value must not be interpreted as proof that the disruption persisted indefinitely.

### E-ENERGY-01 — September energy stress index

| Field | Value |
|---|---|
| Evidence ID | ev.synthetic.energy-index.0001 |
| Source ID | synthetic-source.energy-index.v1 |
| Source record | SYN-REC-ENERGY-0001 |
| Subjects | SYN-ENERGY-STRESS-INDEX; SYN-BUYER-01 |
| Observation | 124 index_points |
| Period | 2026-09 |
| Published | 2026-09-30T10:00:00Z |
| Known / retrieved | 2026-09-30T10:10:00Z |
| Valid from | 2026-09-01T00:00:00Z |
| Valid until (exclusive) | 2026-10-01T00:00:00Z |
| Content SHA-256 | d2f2546d4770292721f77a509cf1e3fb6d1b1a12c9ee8c85f9d5c291dec203b6 |
| Provenance family ID | synthetic-family.energy.v1 |
| Buyer energy tariff | **Unknown** |

**Interpretation:** 124 index points is a September synthetic index observation. The fixture provides no reference baseline, rebasing rule or percentage unit. It is incorrect to describe this as a 24% increase.

The observation's validity interval ended 12 hours before the decision frontier. It remains a historical September observation, not a measurement of the buyer's current October tariff, bill or energy cost.

## 3. Evidence status and claim register

| Claim ID | Claim | Type | Disposition | Evidence / missing input |
|---|---|---|---|---|
| C-01 | A supplier shipment-schedule disruption was reported | Observed | Supported | E-SUP-01 |
| C-02 | The report was available before the decision frontier | Derived temporal check | Supported within fixture | E-SUP-01 known_at |
| C-03 | The supplier remained disrupted at the decision frontier | Current-state claim | Not evaluable | No later status refresh or freshness rule |
| C-04 | The buyer obtains 40% of purchases from this supplier | Quantitative claim | Unsupported | Purchase share is unknown |
| C-05 | The buyer has two weeks of inventory | Quantitative claim | Unsupported | Inventory buffer is unknown |
| C-06 | September energy stress index was 124 points | Observed | Supported | E-ENERGY-01 |
| C-07 | The index implies a 24% energy-cost increase | Derived claim | Rejected | No base value/rebasing rule; wrong unit interpretation |
| C-08 | The buyer's tariff rose | Buyer-specific observation | Unsupported | Tariff is unknown |
| C-09 | The supplier disruption caused buyer revenue loss | Causal/financial claim | Not established | No exposure, inventory, production, contract or outcome chain |
| C-10 | Two conditional operational mechanisms merit investigation | Hypotheses | Plausible, unconfirmed | Inputs needed to discriminate remain unknown |

**Important distinction:** unsupported or not evaluable in this evidence bundle does not mean the statement is false in the world. It means the permitted evidence cannot establish it.

## 4. Competing hypotheses

These hypotheses are conditional, not probability estimates.

### H-01 — disruption absorbed by buffers or substitution

If the buyer has adequate inventory, alternative supplier capacity, or sufficient lead time to substitute, the reported schedule disruption may not materially interrupt production.

**Evidence currently available:** the supplier disruption report.

**Required discriminating evidence:** buyer purchase share by input, usable inventory, substitute supplier identity/capacity, qualification status, lead time, open orders and current shipment schedule.

**Invalidation condition:** verified inventory and substitute capacity are insufficient to bridge the disruption period for the affected production inputs.

### H-02 — disruption becomes an operational bottleneck

If the supplier is material to the buyer, the input is difficult to substitute, and inventory is inadequate, the disruption could delay production or constrain throughput.

**Evidence currently available:** the supplier disruption report and buyer/supplier subject references.

**Required discriminating evidence:** actual procurement exposure, purchase orders, shipment confirmations, bill-of-materials/input criticality, inventory by input, production schedule and substitute lead times.

**Invalidation condition:** verified alternative capacity or sufficient inventory demonstrates that the affected supplier cannot constrain the relevant production period.

### Energy interaction

E-ENERGY-01 may motivate inquiry into the buyer's energy exposure. It does not establish that the buyer's tariff changed, that the buyer consumes energy at a particular rate, or that the energy observation caused or amplified any supplier impact.

A joint supplier/energy scenario is permissible only as a separately labelled hypothetical branch with explicit input assumptions.

## 5. Unknowns and evidence-gap priorities

The following is a **qualitative** priority ordering based on whether the evidence could discriminate the competing hypotheses. No numerical expected value of information has been computed because the fixture provides no outcome probabilities, decision utility, acquisition cost or time-to-acquire model.

| Priority | Requested evidence | Why it matters | Expected owner/system |
|---|---|---|---|
| 1 | Buyer purchase share by supplier/input | Establishes concentration and materiality | Mycelix procurement/source projection |
| 2 | Usable inventory and consumption rate by input | Establishes buffer duration | Mycelix operational evidence |
| 3 | Current supplier status, open orders and shipment confirmations | Distinguishes historical report from continuing disruption | Mycelix source/provenance |
| 4 | Substitute suppliers, qualification and available capacity/lead time | Tests whether substitution is feasible | Mycelix + Sol-Atlas relationships |
| 5 | Bill of materials and production dependency | Maps an affected input to actual operations | Mycelix / qualified operational system |
| 6 | Buyer tariff, energy contract, consumption and billing period | Converts generic index into buyer-specific exposure | Mycelix financial evidence |
| 7 | Index methodology, baseline and revision history | Establishes whether index changes can be compared | Source adapter / evidence verifier |

A later implementation may calculate EVSI or decision-value estimates, but only after it has an explicit decision model, outcome distribution and acquisition cost/time data. Entropy reduction alone is not decision value.

## 6. Scenario panel — assumptions are not observations

The base evidence state leaves buyer purchase share, inventory, substitute capacity and buyer tariff **unknown**. A scenario control must never write hypothetical values back into the observed evidence register.

| Scenario | Explicit assumption | Conditional output | What it does not prove |
|---|---|---|---|
| S-01: high buffer | Inventory covers the disruption window | Production may continue while supply recovers | That the buyer actually has this buffer |
| S-02: low buffer | Inventory is shorter than replenishment delay | Production interruption becomes possible if the input is critical | That any interruption occurred |
| S-03: substitute available | Qualified alternate capacity arrives in time | Disruption may be mitigated | That an alternate supplier exists |
| S-04: no substitute | No qualified capacity is available within the window | Exposure may be materially higher if supplier is critical | A quantified loss or causal outcome |
| S-05: energy sensitivity | Buyer tariff/consumption supplied under a declared contract profile | Conditional energy-cost sensitivity can be calculated | That the synthetic index is the tariff |
| S-06: compound stress | Supplier and energy assumptions both worsen | Joint scenario may show correlated operational pressure | That the observations are causally connected |

Scenario output should carry its own assumption IDs, model version, input frontier and uncertainty. It must be visually distinct from observed facts.

## 7. Dependency graph projection

Only these edges are evidenced in the fixture:
- E-SUP-01 identifies SYN-SUPPLIER-01 and SYN-BUYER-01 in its subject/metadata context.
- E-ENERGY-01 identifies SYN-BUYER-01 and SYN-ENERGY-STRESS-INDEX in its subject context.

No direct edge is established for buyer purchase share, supplier contract/order, actual shipment receipt, inventory dependency, bill-of-materials criticality, alternate supplier, buyer tariff/consumption, production, revenue or financial loss.

An implementation must render missing edges as unknown, not draw inferred supplier or causal links merely because two entities appear in the same report.

## 8. Product projection contract for Sol-Atlas

A first UI implementation should expose these panels without inventing another evidence database:

1. **As-of control:** decision frontier, target/valid-time selector and explicit historical/streaming mode.
2. **Evidence register:** source, record, exact content digest, provenance family, rights profile, published/known/retrieved times, valid interval and freshness state.
3. **Claim inspector:** typed claim, evidence links, derivation, assumptions, contradictory evidence, disposition and uncertainty.
4. **Hypothesis comparison:** H-01/H-02 with evidence for, evidence against, missing discriminators and invalidation condition.
5. **Evidence-gap queue:** qualitative priority initially; future measured acquisition cost/time and EVSI only when grounded.
6. **Scenario controls:** explicit hypothetical inputs; no mutation of base observations.
7. **Replay/export:** deterministic dossier and manifest references.

System ownership:
- Mycelix owns source/evidence identity, rights, financial facts and provenance projections.
- Symthaea owns analytical hypotheses, claim structure, calculations and scenario requests.
- Symtropy may supply qualified physical/operational model outputs, clearly labelled as model outputs.
- Sol-Atlas presents and navigates these objects; visualization cannot increase epistemic status.

## 9. Replay and qualification checklist

| Check | Current state |
|---|---|
| Synthetic task exists | Present in Symthaea PR #11233 branch |
| Two evidence records exist | Present in the synthetic fixture |
| Decision frontier supplied | Present |
| Material unknowns preserved | Done in this dossier |
| Two competing hypotheses written | Done in this dossier |
| Historical energy period distinguished from current tariff | Done in this dossier |
| Supplier current-status freshness established | **Not established** |
| Artifact bytes independently verified | **Not established here** |
| Availability proof bound to principal/rights | **Pending schema bridge #11264** |
| Date-only / interval-censored precision | **Pending #11256** |
| Independent scenario model qualified | **Not performed** |
| Deterministic dossier replay receipt | **Not implemented** |
| Real company or financial impact validated | **Not in scope** |

## 10. Adjudication note

The source fixture was also used for a masked AI rubric-calibration pilot in the Symthaea financial benchmark. That pilot is not a human panel, live model tournament, or independent human reliability result. Its scoring artifact is linked from the Financial Research Agent protocol; this dossier does not import its scores as factual evidence.

## 11. Claim ceiling

This dossier demonstrates how two fictional observations can be translated into evidence-bounded claims, competing operational hypotheses, unknowns and conditional scenario requests. It does not show that a real supplier disruption occurred, that the buyer suffered a loss, that energy prices changed for the buyer, that a causal mechanism was established, or that any system beats an industry benchmark.

A product demo should reproduce this dossier from the exact Symthaea fixture bytes and a verified evidence/availability manifest before the example is upgraded beyond a documentation-only synthetic artifact.