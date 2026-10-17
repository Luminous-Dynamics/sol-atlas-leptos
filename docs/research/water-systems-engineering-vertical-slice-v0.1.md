# Water Systems Engineering Vertical Slice v0.1

**Status:** design proposal; not a hydraulic solver, municipal deployment, certified design, or validated Johannesburg model.  
**Research date:** 2026-10-15.  
**Initial home:** Sol-Atlas public repository, as the engineering evidence and spatial-comparison boundary.

## Executive decision

Build a small, solver-backed engineering workflow that compares water-infrastructure alternatives and preserves enough evidence for an independent engineer to reproduce and review the comparison.

Do **not** start by writing a new hydraulic solver or asking Symthaea to produce pipe designs from unconstrained text. Reuse established domain solvers:

- **EPANET** for pressurized drinking-water distribution networks.
- **EPA SWMM** for rainfall-runoff, stormwater conveyance, sanitary/combined sewer hydraulics, storage, pumps, and related quality calculations supported by the selected profile.
- **Qualified building-plumbing calculation tools and applicable South African codes/standards** for building-level work, as a later profile. EPANET/SWMM do not automatically qualify a building plumbing design.

Sol-Atlas should own the portable engineering-scenario manifest and presentation boundary—not the core physics. Symtropy should contribute its conserved water/terrain context where its contracts support the needed transfer, not become a competing utility-network solver. Symthaea can propose and rank alternatives only by invoking declared, repeatable evaluations. Mycelix can preserve source attribution, deliberation, authorization, and outcome evidence without granting authority to a model recommendation.

## Why this is a useful next increment

The existing public work already separates several important concerns:

- Sol-Atlas PR [#104](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/104) proposes a read-only utility-observation contract and a Johannesburg/Vaal Dam evidence pipeline. Its synthetic wire fixture and pure QA helpers are not proof of a validated end-to-end raster pipeline.
- Sol-Atlas PR [#109](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/109) proposes an append-only, evidence-linked civic case ledger and independent resolution verification. Its open PR status and queued checks must not be described as a pass.
- Sol-Atlas PR [#103](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/103) scopes an evidence-bounded Johannesburg essential-service reliability pilot.
- Mycelix [#4496](https://github.com/Luminous-Dynamics/mycelix/issues/4496) defines a water-system evidence/decision/authorization lifecycle; it explicitly stops short of proving physical performance.
- Symtropy [#448](https://github.com/Luminous-Dynamics/symtropy/issues/448), [#450](https://github.com/Luminous-Dynamics/symtropy/issues/450), [#474](https://github.com/Luminous-Dynamics/symtropy/issues/474), and the open [#487](https://github.com/Luminous-Dynamics/symtropy/pull/487) cover conserved water state, solver hierarchy, infrastructure effects, and multiscale hydrology boundaries.

The missing connective tissue is a portable record for an **engineering model run and its alternatives**: which exact model and input snapshot were used, which solver version ran, what convergence/quality checks passed, which metrics were produced, which assumptions were unverified, and what would be required before a result could influence real work.

## Problem selection: do not assume a target neighbourhood

Johannesburg Water's public pipe-replacement programme describes planned renewal of 428 km of water pipelines and 286 km of sewer pipelines over five years. This is a strong motivation to investigate asset prioritisation, but it is not evidence that any particular pipe is defective or that our proposed workflow will improve the programme. Source: [Johannesburg Water — Pipe Replacement Programme](https://www.johannesburgwater.co.za/about-us/campaigns/pipe-replacement-programme/).

South Africa's National Water Action Plan, released on 23 July 2026, targets more reliable delivery of quality water and identifies infrastructure investment, service-delivery reform, and responses to corruption/criminality as part of the wider programme. Software cannot substitute for institutional capacity, funding, authorised access, maintenance crews, or accountable decisions. Source: [South African Government — National Water Action Plan](https://www.gov.za/news/media-statements/president-cyril-ramaphosa-releases-national-water-action-plan-23-jul-2026).

A first local study area should be selected only after a data-readiness and partner review establishes:
1. an accountable asset/network owner and permitted data scope;
2. adequate topology, asset attributes, demand/flow/pressure or rainfall/flow records for the specific model;
3. a named engineering reviewer and documented acceptance criteria;
4. a safe offline/read-only evaluation path; and
5. a baseline period that permits independent comparison.

If no municipal-quality network data is available, begin with a **labelled synthetic network** and a public-data drainage case. Do not invent underground pipe locations from road lines or treat a public incident map as a calibrated hydraulic model.

## Architecture and responsibility boundaries

```text
approved asset records + observations + declared assumptions
                        |
                        v
       typed engineering model package + input snapshot
                        |
             +----------+----------+
             |                     |
             v                     v
     EPANET water model       SWMM sewer / stormwater model
             |                     |
             +----------+----------+
                        v
       typed run result + quality/uncertainty report
                        |
          baseline <-> candidate alternatives
                        |
                        v
       Symthaea analysis / multi-objective ranking
          (advisory; no implicit decision authority)
                        |
                        v
 Sol-Atlas spatial comparison + Mycelix decision/evidence loop
                        |
                        v
      authorised human decision and separately verified outcome

Symtropy: physical water/terrain context and scenario composition
where supported by its own qualified model/authority contracts.
```

Responsibilities:

- **Sol-Atlas:** portable run/alternative manifests, provenance-aware spatial context, comparisons, and evidence-quality display.
- **EPANET/SWMM:** specialised hydraulic calculations within their documented model domains.
- **Symtropy:** environmental hydrology and infrastructure interactions under its one-water and solver-fidelity contracts. Do not infer that a game/world hydrology simulation is automatically an engineering-grade municipal network model.
- **Symthaea:** generate candidate changes, identify trade-offs, find potentially important scenarios, and explain why an alternative deserves review. It cannot replace solver equations, independent validation, or qualified professional judgment.
- **Mycelix:** source-attributed records, append-only deliberation and decision history, explicit authorization envelopes, repair/implementation receipts, and later outcome review.
- **Utility/building owner and qualified engineer:** own authoritative network information, approve engineering assumptions, determine applicable requirements, approve design, and authorise physical work.

The interface between domains should exchange typed observations and scenario results, not mutate another domain's physical state directly.

## Portable engineering artifact contract

Version 0.1 should be a runtime-neutral contract and fixture set before it becomes a database schema or production UI.

### A. Model package manifest

Required fields:

- `schema_version` and stable `model_id`;
- `model_domain`: `pressurized_water`, `stormwater`, `sanitary_sewer`, `combined_sewer`, or a separately defined profile;
- network/area scope, source-owner reference, and access classification;
- exact input snapshot digest and an inventory of source artifacts;
- coordinate reference system and geometry provenance where geometry exists;
- canonical units and explicit unit-conversion policy;
- solver name, version, executable/build digest when available, and relevant settings;
- model assumptions, omitted processes, and known limitations;
- calibration status and the evidence set used for calibration/validation;
- baseline scenario identity and model revision;
- declared validity period or conditions (e.g. demand pattern, rainfall event, operating schedule);
- author/reviewer references and review state.

A digest establishes byte identity only when the bytes and hashing procedure are available. It does not prove that a source record is authentic, measurements are calibrated, or the network is physically correct.

### B. Scenario definition

Each scenario binds to one exact model package and defines only explicit, typed changes, such as:

- demand/diurnal profile or a bounded demand perturbation;
- pump outage or permitted pump operating schedule;
- valve/pipe isolation or a declared pipe roughness/diameter candidate;
- declared leak/emitter or loss assumptions where the solver profile supports them;
- rainfall hyetograph, subcatchment imperviousness/infiltration assumptions, or conduit restriction;
- tank/storage operating rule;
- a proposed repair/replacement or additional asset with fully specified parameters.

Every change needs units, target asset identity, preconditions, source or author, and an explanation. “AI recommends better pipes” is not a valid scenario definition. A missing pipe diameter, elevation, demand, boundary condition, or material assumption must remain explicitly unknown or block the run if required by the selected model.

### C. Run-result record

Bind each result to the exact model-manifest digest, scenario digest, solver version/configuration, run environment, run identity, and result artifacts. Record at least:

- solver exit state, warnings, convergence and any unresolved numerical issues;
- simulated time range, report interval, time-step information, and omitted periods;
- output metrics as **value + unit + scope + interval + derivation/source**;
- mass/continuity or other model-appropriate closure diagnostics and declared acceptance tolerance;
- parameter uncertainty, observation uncertainty, sensitivity results where available;
- calibration/validation profile and the limits of the evidence;
- failed/unavailable metrics instead of silently substituting zero;
- reproducibility status: `NotRun`, `Reproduced`, `Mismatch`, or `Blocked` with reason.

A solver returning exit code zero is not by itself validation. Model convergence is not empirical fit. Empirical fit is not a guarantee that a future event will be predicted correctly.

### D. Alternative comparison

Keep outcome dimensions separate rather than collapsing them into an unreviewable universal score.

For a water-distribution profile, candidate measures include minimum/low-pressure extent, unmet demand where represented, continuity of supply, pump energy, tank operating range, water age/quality indicators the model actually supports, and lifecycle cost estimates from separately sourced unit-cost assumptions.

For a sewer/stormwater profile, candidate measures include overflow/surcharge duration, flooded or ponded volume where represented, peak and time-varying flow/depth, treatment/storage constraints, receiving-water loads when the model/profile supports them, and lifecycle cost.

All metrics need scope, units, intervals, and interpretation. A lower energy estimate cannot silently compensate for a violation of a mandatory service or safety constraint. Mandatory constraints must be reported as pass/fail/unknown before trade-off ranking. Preserve the Pareto set or factor-level comparison for multi-objective analysis; do not silently invent social weights.

## Use each solver only within its scope

### EPANET — pressurised water distribution

The US EPA describes EPANET as modelling extended-period hydraulic and water-quality behaviour in networks of pipes, junctions, pumps, valves, tanks, and reservoirs. It can analyse pressure, pipe flow, tank levels, selected water-quality constituents, age/source tracing, pump energy, and related operational/design questions. Source: [US EPA EPANET](https://www.epa.gov/water-research/epanet) and [official EPANET 2.2 source repository](https://github.com/USEPA/EPANET2.2).

Use EPANET for the first water-distribution adapter. Keep the adapter read-only: it produces a result artifact and does not command real pumps or valves. Record pressure-driven versus demand-driven analysis mode explicitly because that selection can materially change interpretation under low-pressure conditions.

### SWMM — stormwater, drainage, and sanitary/combined sewer profiles

The US EPA documents SWMM as a dynamic rainfall-runoff model for single-event or continuous simulation. It routes flows through pipes, channels, storage/treatment devices, pumps and regulators, supports multiple hydraulic routing methods, and can model surcharge, reverse flow, surface ponding, and selected pollutant processes. Source: [US EPA SWMM](https://www.epa.gov/water-research/storm-water-management-model-swmm) and [SWMM 5.2 User's Manual](https://nepis.epa.gov/Exe/ZyPURL.cgi?Dockey=P10145M6.TXT).

Use SWMM for rainfall-driven drainage and suitable sewer cases. Do not assume every sewer, water-quality, or combined-system phenomenon is captured by the chosen setup. The run manifest must name the routing model, external inflow assumptions, infiltration/inflow treatment, time-step controls, and relevant limits.

### Building plumbing — a later, separately qualified profile

Building plumbing includes fixture demand, pressure and backflow protection, hot-water distribution, storage, greywater/rainwater reuse, drainage/venting, and building-specific fire or health constraints. Its requirements depend on applicable South African laws, standards, building type, and design conditions. A city-scale distribution model alone cannot certify an installation.

The first vertical slice should defer final building-plumbing sizing until the applicable current standards have been identified by a qualified local professional. Keep potable, reclaimed, greywater, rainwater, and wastewater streams typed separately; never infer that captured or treated water is potable merely because it is physically available.

## Safety, privacy, and authority rules

1. **Read-only first.** No SCADA write path, actuation credential, autonomous pump/valve control, or unreviewed PLC/RTU operation.
2. **Segregate sensitive topology.** Exact critical-asset topology, access details, telemetry, and operational weaknesses must not be copied into public DHTs or public map layers by default. Publish only reviewed, appropriately aggregated projections.
3. **Separate evidence classes.** `Observed`, `Derived`, `Simulated`, `Hypothesis`, `Recommendation`, `Decision`, `AuthorizedAction`, and `VerifiedOutcome` are distinct; none may be silently promoted to another.
4. **Recommendation is not authorization.** A model result can create a review candidate, never a work order or physical effect on its own.
5. **Outcomes need evidence.** A repair dispatch, a work-completed checkbox, and a verified improvement to pressure/overflow are different events.
6. **Fail closed on material missing data.** Unknown units, stale source revisions, unmatched asset IDs, invalid boundary conditions, failed quality checks, missing solver version, or unsupported result fields block downstream claims.
7. **No public blame score.** Failure-risk hypotheses must not be turned into public accusations about a resident, crew, contractor, or asset owner without an accountable and fair review process.

## Minimal validation corpus

Start with two deterministic synthetic fixtures. Do not use live Johannesburg topology in the public test corpus.

### Fixture W0 — pressurised network

Create a tiny, fully documented network with reservoir/tank boundary, junctions, pipes, demand pattern, and pump/valve conditions. Store exact input bytes and solver configuration. Include:
- baseline;
- a controlled pump-outage or isolation case with an expected directional service effect;
- a candidate pipe intervention with explicitly stated dimensions and roughness assumptions;
- a deliberately invalid unit or dangling asset reference that must be rejected before solver invocation;
- a non-convergent or incomplete-run fixture if the solver interface can represent it safely.

The expected directional change must be established from the defined fixture using the actual selected solver—not asserted from intuition. Freeze the observed reference result only after its origin, solver version, and run output are captured.

### Fixture S0 — drainage/sewer network

Create a small synthetic catchment with known rainfall forcing, subcatchments, conduits, junctions and an outlet. Include:
- baseline;
- a declared conduit restriction/blockage scenario;
- an alternative storage/infiltration intervention with explicit parameters;
- missing rainfall units and invalid conduit references;
- an overflow/surcharge case whose selected solver profile is expected to represent it.

Check water continuity/closure within a declared tolerance, output units/time intervals, and expected directional changes only where demonstrated by the fixture. This is a software/reference validation corpus, not proof of accuracy for any real catchment.

### Adversarial/negative controls

- a result produced from a different input digest cannot be attached to the requested scenario;
- same visible model ID with a different revision/digest is not treated as the same model;
- a stale or unverified calibration label cannot be displayed as qualified;
- a missing metric does not default to zero;
- mixed units or unknown unit symbols are rejected;
- failure/non-convergence never becomes a recommendation for deployment;
- a recommendation cannot generate an authorization or implementation receipt;
- reproducing the same synthetic input under a different solver version is a new evidence lineage, not automatic equality;
- a failed test or queued CI check is never shown as passed.

## Qualification ladder

- **G0 — contract/fixtures:** schema, sample manifests, explicit failure states, and synthetic-only data; tests validate contract semantics.
- **G1 — local solver adapters:** run the installed/version-pinned EPANET and SWMM binaries/engines against the synthetic fixtures; capture stdout/diagnostics, exact result artifacts, and conservation/closure checks.
- **G2 — independent technical review:** an engineer who did not author the fixture reviews units, boundary conditions, solver configuration, expected results, and quality/acceptance criteria.
- **G3 — empirical validation:** compare selected model outputs with suitable independent observations; publish calibration window, withheld test periods, error distributions and known failure modes.
- **G4 — approved read-only pilot:** only after data-owner permission, protection review, asset matching, and baseline freeze; replay historical records first.
- **G5 — shadow recommendations:** compare predictions/recommendations against later observations while humans retain all operational authority; no autonomous control.

Passing G0 or G1 is not G3, engineering certification, a successful municipal pilot, or evidence of improved service delivery. Queue state is not a pass.

## Starter artifacts in this PR

- `sol-atlas-core/schemas/water-engineering-bundle-v0.1.schema.json` defines a strict Draft 2020-12 envelope for the model package, scenario changes, run state, diagnostics, metrics, result artifacts, and validation claim ceiling.
- `sol-atlas-core/fixtures/water-engineering/w0_network.inp` is a small synthetic EPANET-format input with explicit SI/LPS conventions. It does not represent a real utility network.
- `sol-atlas-core/fixtures/water-engineering/w0-p2-diameter-restriction.synthetic.json` binds a proposed pipe-parameter change to the exact input-file digest. Its run state is `NotRun`, contains no metrics or result artifacts, and explicitly says that no solver was invoked.
- The schema makes the `NotRun` state incompatible with reported run metrics, result artifacts, solver exit code, convergence, or execution timestamps.

The schema and fixture shape were checked against JSON Schema Draft 2020-12 in an earlier local validation step. A Rust semantic validator and CLI have now been added to this draft branch to check cross-record identity, referenced file bytes, SHA-256 and length consistency, asset references, and run-state claim ceilings. The new Rust tests and CLI have been authored but have not yet been executed in this environment. The PR branch also adds an explicit `cargo test -p sol-atlas-core` CI step; its result must be checked against the exact current head before any pass is claimed. EPANET has not been executed and no hydraulic result has been produced.

## Suggested implementation sequence

1. Keep the engineering model/run contract separate from the utility-observation and civic-case contracts to avoid duplicate type ownership.
2. Use the starter schema and synthetic `NotRun` bundle as a contract seed; bind utility observations only after the related interface is reviewed and stabilised.
3. Run and review the Rust semantic validator now added in this draft branch. It checks cross-record model/scenario/input identity, source-inventory consistency, referenced artifact bytes and digests, unique asset IDs, scenario target references, solver-engine identity, and impossible NotRun/Completed claims. Add unit/semantic checks for unit-policy and completed-run manifest completeness before relying on it.
4. Add an offline, deterministic harness invoking explicitly versioned external solvers; do not embed an unreviewed engine fork.
5. Capture actual solver logs/results, verify the exact input/run manifests and conservation/closure diagnostics, then obtain independent technical review.
6. Connect only qualified metrics to Sol-Atlas layers and civic cases; never expose raw critical-infrastructure data through public views.
7. Feed candidate results to Symthaea for constrained, multi-objective comparison. Preserve the full metric vector, assumptions, and uncertainty.
8. Record a human-reviewed decision and explicit authorisation through the Mycelix workflow, then compare post-intervention observations against the frozen baseline.
9. Add a building-plumbing profile only after its local technical and regulatory requirements are separately mapped.

## Solver pinning research for the next executable gate

The next run should be reproducible from immutable source identities, not an unqualified "latest" install:

- **EPANET candidate:** OpenWaterAnalytics EPANET release tag `v2.2` resolves to source commit `4d8d82ddc260fce216af9321fc3d9a4646ac6827`. Its build documentation describes a CMake-built `runepanet` CLI on Linux/macOS. Source: [release ref](https://github.com/OpenWaterAnalytics/EPANET/releases/tag/v2.2), [BUILDING.md](https://github.com/OpenWaterAnalytics/EPANET/blob/4d8d82ddc260fce216af9321fc3d9a4646ac6827/BUILDING.md). The separate EPA archive tag `USEPA/EPANET2.2:2.2.0` resolves to `6d7f5f9c08048ce6b25bb64340547446362ec389`; select one source lineage explicitly rather than mixing them.
- **SWMM candidate for the later fixture:** official EPA repository release `v5.2.4` resolves to `7952ca837988b1c32f791812eccc9fd64547e093`. The EPA repository documents building the `runswmm` command-line executable with CMake. Sources: [release](https://github.com/USEPA/Stormwater-Management-Model/releases/tag/v5.2.4), [EPA SWMM project page](https://www.epa.gov/water-research/storm-water-management-model-swmm).

These are reproducible starting candidates, not a claim that either engine was built or executed here. Before labelling a run reproducible, record the full source commit, build-tool/compiler versions and flags, resulting executable SHA-256, platform, input digest, command arguments, exit status, complete logs/reports, binary output artifact, and the independent checks applied to its diagnostics. Pinning source does not by itself prove the build is reproducible, and a successful solver exit does not prove empirical accuracy.

## Semantic validator implementation (draft-branch increment)

The draft branch now adds:

- `sol-atlas-core/src/water_engineering.rs`: `validate_bundle`, a fail-closed semantic validator separate from JSON Schema.
- `sol-atlas-core/src/bin/water_bundle_validate.rs`: a command-line entry point which returns non-zero when semantic checks fail.
- Unit tests for the clean NotRun fixture, model/scenario mismatch, input digest mismatch, metrics on a NotRun run, unknown scenario asset, well-formed but incorrect SHA-256, path traversal, and duplicate asset IDs.

Run from the repository root:

```sh
cargo test -p sol-atlas-core water_engineering::tests -- --nocapture
cargo run -p sol-atlas-core --bin water-bundle-validate -- \
  sol-atlas-core/fixtures/water-engineering/w0-p2-diameter-restriction.synthetic.json \
  "$PWD"
```

The command prints `SEMANTIC_VALIDATION_PASS` only for the semantic checks it implements; it explicitly does not run a hydraulic solver. The fixture must also pass the separate Draft 2020-12 schema gate. These commands are instructions, not reported test results: local execution was unavailable in this work session and no exact-head CI result is claimed. The added native-core CI step is intended to execute the tests after a runner starts. There is no EPANET/SWMM executable installed in the available shell, so no hydraulic run is claimed either.


## Research references

- US EPA, [EPANET — water-distribution modelling](https://www.epa.gov/water-research/epanet).
- US EPA, [EPANET 2.2 source repository and licence](https://github.com/USEPA/EPANET2.2).
- US EPA, [SWMM — stormwater, sanitary/combined sewer, and drainage modelling](https://www.epa.gov/water-research/storm-water-management-model-swmm).
- US EPA, [SWMM 5.2 User's Manual](https://nepis.epa.gov/Exe/ZyPURL.cgi?Dockey=P10145M6.TXT).
- Johannesburg Water, [Pipe Replacement Programme](https://www.johannesburgwater.co.za/about-us/campaigns/pipe-replacement-programme/).
- South African Government, [National Water Action Plan, 23 July 2026](https://www.gov.za/news/media-statements/president-cyril-ramaphosa-releases-national-water-action-plan-23-jul-2026).
- South African Government, [National norms and standards for domestic water and sanitation services](https://www.gov.za/documents/notices/national-water-act-national-norms-and-standards-domestic-water-and-sanitation) and [Compulsory National Water and Sanitation Services Standards](https://www.gov.za/documents/notices/water-services-act-compulsory-national-water-and-sanitation-services-standards). These are reference entry points, not a substitute for checking current applicability and detailed requirements.

## Real public data intake (aggregate-only first step)

A companion source-attributed observation bundle has been added at
`sol-atlas-core/fixtures/water-engineering/johannesburg-water-public-observations-2024-25.v0.1.json`,
with its independent Draft 2020-12 schema at
`sol-atlas-core/schemas/water-public-observation-bundle-v0.1.schema.json`.
It records 91 observations across six source records: the system-wide 2024/25 water-quality
summary, the published five-year pipe-renewal targets, and provisional 2023/24–2024/25 annual-
report performance/compliance indicators. Annual-report values remain explicitly provisional
pending page-level review of the PDFs. The bundle preserves source IDs, reporting dates, units,
source limitations, and distinctions between published aggregates, planned targets, and values
requiring primary-document verification. See
[`water-data-readiness-johannesburg-v0.1.md`](water-data-readiness-johannesburg-v0.1.md)
for the data-source register and the next acquisition gates.

The source-reference semantic validator and CLI are in
`sol-atlas-core/src/water_observations.rs` and
`sol-atlas-core/src/bin/water_observation_validate.rs`. The CLI command is:

```sh
cargo test -p sol-atlas-core water_observations::tests -- --nocapture
cargo run -p sol-atlas-core --bin water-observation-validate -- \
  sol-atlas-core/fixtures/water-engineering/johannesburg-water-public-observations-2024-25.v0.1.json
```

The bundle now also captures eight figures from Johannesburg Water's [official Active Leak Detection Programme page](https://www.johannesburgwater.co.za/about-us/campaigns/active-leak-detection-programme/): 15 daily teams; in excess of 10,500 km inspected per year (reported as 85% of water infrastructure length); average annual detections of 1,456 bursts, 5,227 leaking meters, 341 valves and 217 hydrants; and approximately 8,100 ML saved annually. These webpage figures are labelled as publisher-reported and not independently reproduced, not verified events or model inputs. The separate Business Plan estimate of 14,355,757 kL/year is retained separately: definitions/time windows are not reconciled, so do not claim one figure confirms or disproves the other.

The FY2025/26 Business Plan adds useful operational context from its page 49 active leak-detection table and page 50 pressure-management discussion: it reports totals of 2,620 burst-pipe activities, 7,212 leaking-meter activities, 476 leaking-valve activities, 283 leaking-hydrant activities, and 10,663.9 km of mains surveyed for FY2023/24. It also reports an estimated 14,355,757 kL/year demand reduction from active leak detection and potential pressure-management savings of 5,918,000 kL/year, and lists a multi-year schedule for 223 advanced leak-detection devices. Source: [Johannesburg Water Business Plan 2025/26, pp. 49–50](https://johannesburgwater.co.za/wp-content/uploads/2025/07/JHB-Water_Business-Plan-2025_For-Upload_FINAL_070725.pdf).

These figures are now in the bundle as 13 source-attributed provisional records. The report's regional activity components do not reconcile arithmetically to several printed totals in the indexed text, so no regional breakdowns are ingested as trusted values. The savings are publisher-reported estimates or potential—not independently verified realised savings. We also preserved two annual-report Table 26 differences (92.67% vs 92.66% sewer response; 67.46% vs 67.42% water-burst response) with the publishing source attached to each value rather than silently choosing one. A machine-readable comparison register now links these two KPI conflicts and the differently scoped savings estimates; the Rust validator checks the referenced observation IDs and recomputes the recorded unit-normalised arithmetic without treating these as reconciled trends.

The real-data fixture is **not** pressure telemetry, site-level chemistry, a pipe inventory,
a calibrated EPANET model, or SWMM forcing. Annual-report excerpts are not treated as verified
until a human checks their source tables. DWS data must be used under its stated restrictions;
where public dashboards do not expose the raw export in this access path, the data is listed
as a candidate request/export rather than invented or silently scraped.

## Nonclaims

This proposal does not establish that an installed implementation exists, any test has run, a hydraulic model is accurate, any particular Johannesburg network is available, any municipal partner has approved access, any intervention will save a specified amount of water or money, or a candidate design complies with applicable law. Those are separate proof obligations.
