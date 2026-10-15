# Black Sea Grain Initiative: UN Comtrade query plan

- **State:** `query_plan_only`; API requests have **not** been run.
- **Machine-readable plan:** [black-sea-grain-trade-query-plan.json](black-sea-grain-trade-query-plan.json).
- **Case readiness:** remains `design_only`; this plan does not qualify the pilot for analysis.

## Purpose and limits

The query is intended to establish whether a reproducible monthly, country-level trade series exists for Ukraine's wheat (HS 1001) and maize (HS 1005) exports. It is a separate all-route trade measure, not a reconstruction of shipments carried under the Black Sea Grain Initiative. UN Comtrade reports merchandise-trade aggregates; it does not identify named ships, firms, consignments or physical routes. See the [UN Comtrade API guide](https://uncomtrade.org/docs/un-comtrade-api/).

The initial dimensions are goods (`C`), monthly (`M`), classification discovery code `HS`, reporter Ukraine (M49 `804`), partner world aggregate (`0`), exports (`X`), and product codes `1001,1005`. The first pass is metadata and coverage discovery, **not an assumption that results are comparable**. Preserve returned classification/version fields and do not pool across incompatible classifications or meanings.

## Bounded requests

Each calendar year from 2017 through 2023 is a separate 12-month request chunk. This deliberately bounded client policy makes retrieval, retries and completeness accounting auditable. It is not presented as a server-enforced maximum. The Comtrade preview guide documents the preview endpoint; verify the current access and output constraints at retrieval time.

An example request shape is:

```text
GET https://comtradeapi.un.org/public/v1/preview/C/M/HS
    ?reporterCode=804
    &partnerCode=0
    &cmdCode=1001,1005
    &flowCode=X
    &period=202201,202202,202203,202204,202205,202206,202207,202208,202209,202210,202211,202212
```

Use the exact request parameters in the JSON plan; do not treat the formatted line breaks as a literal URL. A response that reports more rows than it returns, reaches the 500-row preview ceiling, or omits needed dimensions is incomplete/ambiguous and must not advance. Preserve each raw response **before** parsing, plus exact request URL/parameters, retrieval timestamp, HTTP metadata available, byte length, SHA-256, response count, parser/tool version and source terms. Never save a subscription key.

## Measurement and completeness rules

- Preserve all raw fields. Inspect the actual source schema and reference metadata before interpreting `primaryValue`, `netWgt`, `qty`, quantity units, estimated/aggregate flags or report status.
- Do not convert a missing quantity to zero; do not silently mix trade value, net weight and reported quantity; do not deduplicate without a documented row key.
- Inspect reporting coverage, estimated records, revisions, classification vintage, partner semantics, currency/value basis and quantity units by month. If the data cannot support a comparable monthly series, narrow the claim or abandon that comparison.
- Keep Comtrade monthly aggregates separate from the JCC Initiative-only monthly observations and from FAO price indices. Alignment is descriptive, not causal evidence.

## Control policy

No control group is selected by this query plan. After inspecting metadata and missingness—but before examining the resulting outcome patterns—write a dated preregistration that justifies comparison exporters/commodities, seasonality, treatment of the COVID period, concurrent shocks, exclusions and sensitivity checks. Do not pick or swap controls to obtain a preferred result. The proposed 2017–2021 baseline includes pandemic-era disruption and is not, by itself, a causal counterfactual.

## Local exact-byte capture helper

For a network-enabled workstation, run the bounded, read-only acquisition helper from the repository root using a **new output directory**:

```sh
python3 scripts/earth-systems/capture_black_sea_sources.py \
  --snapshot-dir /tmp/black-sea-sources-2026-10-15
```

The command reads this plan, fetches the official JCC page, the linked FAO monthly CSV, and each of the seven Comtrade chunks, then records exact response byte counts, SHA-256 digests, timestamps, request URLs/parameters, selected response headers and per-source status in `capture-manifest.json`. Payloads remain only in the chosen local snapshot directory; the command does not commit or upload them. It uses no credentials. Each response is bounded, redirects must remain on the allowlisted HTTPS source paths, and existing snapshot paths/files are never overwritten. Preview-limit, malformed-response, HTTP, content-type and storage failures remain explicit and make the command exit non-zero; successful individual artifacts are retained with their manifest state for diagnosis.

This helper does **not** mean sources have been captured in this current review. No hash is filled into the research plan, and no terms, coverage, measurements, classification, independent review or causal claim is pre-approved. Before storing snapshots anywhere shared, review the source terms and retention policy. A response hash is a byte-integrity claim only.

## Advancement gates

The plan's validation flags stay false until the real chunks are acquired and hashed, completeness and schema are independently checked, units/classification are reviewed, a control design is preregistered, and independent review is documented. Until then, no trade-data result, causal estimate, operational warning or supply-chain prediction is claimed.
