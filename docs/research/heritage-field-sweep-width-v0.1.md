# HDB-15 — Calibrated Field Survey Detectability v0.1

## Research basis

Banning et al. (2017), *Quality Assurance in Archaeological Survey*, DOI 10.1007/s10816-016-9274-2, recommends calibration transects over test fields seeded with artifacts at known locations. Surveyors should walk at realistic speeds; calibration should record artifact class and lateral range, repeat over the course of a project, and account for visibility, crew and target obtrusiveness. It models lateral detection as:

`p(r)=b exp(-k r²)`

where `p(r)` is the probability of detection at perpendicular distance `r` from the transect, `b` is detection probability on the transect and `k` controls lateral decay. Effective sweep width is the full integral under this curve:

`W = integral[-∞,+∞] p(r) dr = b sqrt(pi/k)`.

The width is an equivalent perfect-detection band, not a claim that detection is certain inside the band. It is method- and context-specific; the same fit cannot be transferred from fieldwalking to magnetometry, augering, UAV, satellite or buried-target detection.

Edwards et al. (2026), *Bayesian optimal allocation of effort in archaeological survey: A case study in Wadi Quṣaybah, Jordan*, DOI 10.1016/j.jas.2026.106568, combines geomorphological modelling of surviving ancient land surfaces, calibration using simulated artifact scatters, sweep-width-based coverage and daily Bayesian reallocation of field survey effort. Several late-prehistoric sites were found; subsequent excavation confirmed at least two as Late Neolithic. This is a field-project comparator, not validation of Sol-Atlas.

## v0.1 implementation

The Rust evaluator consumes a frozen, externally fitted profile. It does not fit the nonlinear detection function from raw calibration observations. This avoids pretending to reproduce the paper's external R/SPSS fit without raw calibration data and fit diagnostics.

The profile binds:
- target material/feature class and obtrusiveness;
- fieldwalking method;
- crew cohort and declared search speed;
- visibility, ground cover, moisture, illumination, season and terrain strata;
- fitted `b`, `k`, parameter standard errors and their covariance;
- fit software/method, fit receipt, calibration data digest and split digest;
- disjoint fit and validation transect IDs, seeded target counts and held-out Brier score;
- rights status and receipt.

A v0.1 engineering floor requires at least 3 fit transects, 2 independent validation transects, 20 fit targets and 10 validation targets. These are initial guards, not a universal statistical sample-size recommendation.

A first-order delta-method standard error is propagated from the supplied 2×2 fitted-parameter covariance:
`Var(W) ≈ (dW/db)² Var(b) + (dW/dk)² Var(k) + 2(dW/db)(dW/dk)Cov(b,k)`.
The report includes a two-sided normal-approximation 95% interval. This approximation is limited, especially near the boundary `b=0/1` or with small calibration samples. Bootstrap/profile-likelihood intervals should be preferred in the full calibration pipeline.

The coverage receipt must supply a GIS-side overlap-adjusted transect length, CRS, geometry digests and spatial-union receipt. The core computes `nominal swept area = W × overlap-adjusted length` and divides by survey-unit area. It does not compute geometric unions; that is the responsibility of a CRS-aware geospatial verifier. If the supplied ratio exceeds one, evaluation fails closed.

## Claim boundaries

- This is fieldwalking only. It does not qualify other instruments or methods.
- The report does not estimate whether an ancient land surface survived; that is a separate geomorphological model.
- It does not estimate whether archaeological materials occur in the survey unit.
- A non-finding does not establish absence.
- Repeated searches are not assumed independent.
- Structural receipts/digests are not authenticated by the module.
- The 95% interval is conditional on the externally supplied fit and covariance estimate.
- No survey plan authorizes access or fieldwork.

## Benchmark next steps

- Reproduce Wadi Quṣaybah calibration and coverage with its original data/code/fit semantics where available and rights-compatible.
- Compare the parametric exponential fit with empirical/binned detection functions and bootstrap/profile-likelihood intervals.
- Validate on independent field/season/crew/target-class profiles.
- Feed calibrated coverage/detectability receipts into HDB-11 and PR #446's sequential survey planner.
- Evaluate uniform-grid, random, expert-designed, static geomorphological and sequential Bayesian allocation at matched crew-hour budgets.
