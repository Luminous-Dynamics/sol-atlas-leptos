# Pool heat-recovery conservation ledger v0.1

**Study ID:** HIGHVELD-POOL-HEAT-RECOVERY-LEDGER-001  
**Parent:** HIGHVELD-POOL-WATER-THERMAL-001 / ENERGY-PORTS-001  
**Maturity:** SYNTHETIC_CONSERVATION_CONTRACT_NOT_PHYSICALLY_VALIDATED

## 1. Why this is separate from the pool archetype qualifier

Registering a heat-recovery arm and requiring a fallback sink does not prove the proposed heat balance. A heat pump's condenser heat is not identical to building-side cooling: compressor electrical work enters the condenser energy balance, and compressor-shell/ambient losses must be accounted for. The condenser's heat output must then be routed exactly once among pool recovery, a qualified fallback sink and measured distribution losses.

This ledger is an independent conservation check around measured meters or specialist model outputs. It does not simulate a vapor-compression cycle, size a heat exchanger, predict pool temperature, prove displaced heater savings or validate water quality.

## 2. Two distinct first-law boundaries

### 2.1 Refrigeration circuit

For the declared compressor/refrigerant control volume:

\[
Q_{\mathrm{cond}} =
Q_{\mathrm{building,removed}} +
W_{\mathrm{compressor}} -
Q_{\mathrm{compressor,ambient\ loss}}
\]

The signs are explicit: compressor electrical work enters the heat-pump circuit; measured heat lost from the declared boundary to ambient is subtracted. Auxiliary pump/fan/treatment electricity remains a separate ledger unless the selected meter boundary shows it enters this control volume.

The residual must fall within a predeclared expanded uncertainty from the actual instruments/model. A missing term is not zero.

### 2.2 Condenser heat disposition

\[
Q_{\mathrm{cond}} =
Q_{\mathrm{pool,recovered}} +
Q_{\mathrm{fallback,rejected}} +
Q_{\mathrm{distribution,loss}}
\]

Every joule leaving the condenser must have one measured destination or remain explicitly unaccounted and therefore unqualified. Pool recovery is bounded by both actual pool heating demand and remaining thermal headroom below the authority/owner-approved temperature ceiling. If no pool demand or headroom remains, recovery is disabled and excess condenser heat goes to a separately qualified fallback sink.

## 3. Keep three service claims separate

The output reports three distinct quantities:
- **building-side heat removed**: cooling service delivered to the building boundary;
- **pool heat recovered**: useful heat delivered to the pool's heating demand;
- **pool-heater input displaced**: independently measured avoided energy at the baseline pool heater.

Do not sum condenser heat and pool heat and relabel the sum as building cooling. Do not infer displaced heater energy from recovered heat alone; the baseline heater, its efficiency/control, the measurement period and counterfactual must be qualified separately.

## 4. Synthetic known-answer case

The registered synthetic case uses:
- building-side heat removed 10.0 kWh;
- compressor electrical work 3.0 kWh;
- compressor-to-ambient loss 0.2 kWh;
- measured condenser output 12.8 kWh;
- pool heating demand 6.0 kWh and available headroom 5.0 kWh;
- pool recovery 5.0 kWh, qualified fallback rejection 7.5 kWh and distribution loss 0.3 kWh;
- separately measured displaced heater input 4.5 kWh.

The numbers close both boundaries exactly. They are synthetic arithmetic, not a measured system or a Highveld performance forecast.

## 5. Required evidence

Every case binds:
- source bundle and exact model/configuration/BOM revision;
- building-side heat/flow meter and compressor electricity meter;
- compressor ambient-loss method/evidence;
- condenser heat meter;
- pool demand, water temperature, remaining headroom and authority-approved ceiling;
- isolated heat-exchanger admission and transfer meter;
- fallback sink identity, qualification, available capacity and rejection meter;
- distribution-loss method/evidence;
- uncertainty method and independent known-answer oracle;
- separate baseline pool-heater measurement and displacement method.

Evidence references establish lineage, not sensor truth. Instrument admission and uncertainty review remain separate.

## 6. Negative controls and benchmark targets

- condenser first-law residual beyond uncertainty;
- condenser disposition residual beyond uncertainty;
- pool recovery above heat demand;
- pool recovery above remaining thermal headroom;
- pool temperature at/above its admitted ceiling;
- non-isolated/unqualified heat exchanger;
- excess heat with unqualified or undersized fallback sink;
- non-zero recovery when pool demand is zero;
- missing compressor, condenser, pool, fallback or uncertainty evidence;
- displaced pool-heater claim without baseline meter and counterfactual evidence.

Targets: zero accepted balance residuals beyond declared uncertainty; zero heat disposition double-count; zero pool limit/headroom violations; zero unqualified excess-heat discharge; preserve the valid positive control; 100% input/evidence/model traceability.

## 7. Industry comparison and claim ceiling

The UC Davis WCEC hotel case and California pool-heat-sink feasibility analysis remain distinct reference evidence classes in the parent manifest: one is a field demonstration, the other a selected-condition modelled feasibility estimate. Neither is a local performance prediction. The energy ledger makes their lessons testable without copying reported savings into Highveld's expected results.

**Status:** SYNTHETIC_CONSERVATION_CONTRACT_NOT_PHYSICALLY_VALIDATED. No actual heat pump, pool, condenser meter, fallback sink, displaced heater or field data has been admitted.
