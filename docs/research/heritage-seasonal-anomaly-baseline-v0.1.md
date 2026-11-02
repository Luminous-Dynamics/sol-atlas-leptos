# Seasonal Robust Deviation (SRD) baseline v0.1

**Purpose:** provide a fixed-reference, interpretable change score to compare with learned temporal embedding distance and self-supervised change detection.
**Status:** synthetic implementation and protocol only; not yet evaluated on archaeological data.

## Method

For each monthly feature x_t, select valid reference observations from the same calendar month in an explicitly fixed reference period ending before the evaluation period.

- m_t = median(R_t)
- MAD_t = median(|R_t - m_t|)
- s_t = max(1.4826 * MAD_t, scale_floor)
- z_t = (x_t - m_t) / s_t

The report keeps signed z, absolute z, and the monotonic anomaly-strength index 1 - exp(-|z|) separate. The last is **not a calibrated probability**. The alert threshold must be preregistered or selected on validation data, never tuned on the blind test.

The v0.1 method has no learned parameters, no future leakage, and no imputation. It uses a fixed reference period; it does not continuously update its normal model during evaluation. A reference needs at least two distinct years of valid observations for the same month, otherwise the system abstains.

## Why it matters

Archaeological satellite monitoring is confounded by seasonal vegetation, agriculture, soil moisture, illumination, drought, floods, sensor calibration and compositing. The 2026 literature now treats phenology-aware pseudo-change suppression as an explicit part of heritage-change pipelines. A 2026 cropmark study reports directional transfer between landscapes based on spectral-geometric similarity rather than simple geographic proximity. These findings motivate separate seasonal-normalisation and cross-landscape-transfer controls; they do not show that a simple median/MAD score is sufficient for archaeological interpretation.

References:
- UAV heritage condition workflow with phenology-aware pseudo-change suppression, Scientific Reports (2026): https://www.nature.com/articles/s41598-026-63132-8
- Beyond site-specific archaeological prospection: Spectral-geometry reveals cross-site cropmark signals in satellite data, PNAS Nexus (published 2026-10-10): https://academic.oup.com/pnasnexus/advance-article/doi/10.1093/pnasnexus/pgag336/8907313
- WATCH temporal archaeological change detection: https://github.com/microsoft/WATCH

## Input contract

Each series identifies one physical/spectral/embedding feature by stable ID, units, exact source reference, SHA-256 digest of the feature definition, and strictly ordered Gregorian month ordinals with optional values.

Do not use a feature called NDVI unless actual near-infrared and red inputs are present and the source channel contract authorises that calculation. A validity mask is not a spectral band. The heritage_baseline module enforces channel semantic compatibility separately.

## Missing data and abstention

- An absent month is reported by exact month ordinal.
- An explicit null is present but invalid.
- A valid observation without enough reference years is marked insufficient_reference.
- A scored month is distinct from a month merely present in the archive.
- observation_coverage is non-null observations / expected months.
- score_coverage is months actually scored / expected months.
- Evaluation windows are bounded to 240 months per call.

No missing month is silently treated as zero change.

## Synthetic tests

The authored Rust tests cover deterministic repeatable seasonality, a persistent injected shift, missing vs absent month accounting, insufficient reference years, exclusion of future data, duplicate months, overlapping windows, non-finite inputs, bounded evaluation windows, and extreme values that would overflow robust statistics. These are test designs, not executed Rust evidence until the exact PR head compiles and tests pass.

## Failure modes and limits

SRD does not distinguish archaeological threat from non-archaeological disturbance. A road, quarry, field, erosion event, vegetation burn or sensor gain change may generate a strong anomaly. The score is a detection baseline, not a causal label.

It assumes same-month historical observations are an adequate seasonal counterfactual. In climates with variable planting/harvest timing or strong multi-year drought, the same calendar month can represent different phenological states. Later work should compare circular month windows, robust trend/seasonal decomposition, SAR/optical fusion, and local climate/phenology covariates without contaminating the fixed blind evaluation.

It also assumes feature units are already harmonised. A sensor transition can create an anomaly; that should be reported as sensor/domain shift, not a heritage threat.

## Benchmark protocol

Compare SRD, WATCH TED, WATCH SSCD, and a simple temporal-difference baseline on the same sites, months, masks, event labels and fixed splits. Report exact-month and ±1/±3/±6-month event recall, 1–3 and 4–6 month early-warning recall, false-alert episodes per 100 observed site-months, observation and score coverage, performance per region/sensor/biome/change cause, and analyst-review yield at fixed monthly review budgets.

Do not claim superiority from anomaly magnitude or synthetic tests.