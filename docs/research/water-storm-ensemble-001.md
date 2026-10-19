# WATER-STORM-ENSEMBLE-001 — paired stormwater stress envelope v0.1

## Goal

Extend the successful S0 solver-adapter smoke into a paired 54-case synthetic event matrix. Keep EPA SWMM as the hydraulic engine and make event identity, geometry, outputs and uncertainty explicit. The benchmark is not a new hydraulic solver and does not use learned surrogates in place of SWMM.

## Frozen matrix

Six six-interval rainfall shapes × three depth multipliers (0.5×, 1.0×, 1.5×) × three circular conduit diameters (0.60 m, 0.40 m, 0.25 m) = **54 runs**.

Every base profile has the same nominal event depth, approximately 14.166667 mm, before applying the depth multiplier. Shapes cover the current reference pulse, a concentrated burst, early peak, delayed peak, two separated peaks and sustained moderate rain. All are hand-authored at a five-minute interval. These shapes are not observed storms, IDF-derived design storms or return-period events.

Every restricted diameter case is paired with the 0.60 m diameter result for the exact same rainfall profile and depth multiplier. This enables event-matched deltas rather than comparing interventions under different rainfall.

## Why ensembles instead of a single headline run?

SWMM supports single-event and continuous rainfall-runoff, dynamic-wave flow routing and (when enabled) water-quality simulation. Its manual supports comparing modeled node depth, flooding, link velocity and flow with calibration data: https://nepis.epa.gov/Exe/ZyPURL.cgi?Dockey=P10145M6.TXT. A 2025 SWMManywhere study found important interaction among urban drainage-model parameters and recommended an uncertainty-driven ensemble approach: https://doi.org/10.1016/j.envsoft.2025.106358. Another 2025 study found that incomplete network data affects simulated drainage results differently across rainfall intensity, event timing and spatial connectivity: https://www.sciencedirect.com/science/article/pii/S0022169425011266.

For South Africa, the SANRAL Drainage Manual 6th Edition discusses regional rainfall temporal distributions. This toy matrix must not be substituted for locally adopted design rainfall or current project standards: https://www.dws.gov.za/projects/nfsp/docs/sanral/Drainage%20Manual%20%28Sixth%20Edition%20%5B2013%5D%29.pdf.

## Required outputs

Each case records exact derived input SHA-256/byte identity, report/output/stdout digests, solver revision and executable digest; runoff and routing continuity; rainfall/runoff volume; peak junction depth and time; peak conduit flow and velocity; ratios to full flow/depth; outfall peak flow/volume; flooding/surcharge state and flooding loss where present.

The receipt also carries matched-baseline deltas for peak node depth, conduit velocity, outfall peak flow and flooding loss. Raw per-case outputs are retained, and summary statistics must be computed only from completed cases with denominators explicit. A composite scalar score is not used.

## Acceptance

- All 54 IDs unique and exactly cover the 6 × 3 × 3 product.
- Every paired baseline resolves to the same event/profile and multiplier at 0.60 m.
- The source input SHA and byte length match before transformation.
- Each derived case differs only in the declared 6-point rainfall vector and C1 diameter.
- A 1% absolute continuity limit is used as a **synthetic smoke guard only**, while exact continuity values are always emitted.
- Solver error/warning signals, missing reports or missing outputs fail closed; failure receipts retain null for unknown metrics.
- The overall receipt is schema-valid for both success and failure; success requires all 54 runs complete.
- No return period, empirical calibration, flooding safety, environmental quality, regulatory compliance or intervention benefit is claimed.

## Next layer

After the scenario plumbing is verified, expand the ensemble to declared ranges for infiltration, imperviousness, conduit roughness, initial storage, outfall boundary and topology. Then validate against authorized observed rainfall/flow/depth data on held-out events and compare to a simple baseline and conventional SWMM-only workflow. Do not tune against the eventual held-out evaluation events.
