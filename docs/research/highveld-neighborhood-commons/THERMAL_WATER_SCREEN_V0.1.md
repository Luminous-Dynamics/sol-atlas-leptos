# Highveld first-principles thermal-water screening v0.1

**Study:** `HIGHVELD-PASSIVE-COOLING-WATER-001`  
**Purpose:** deterministic unit/ceiling checks before detailed thermal simulation  
**Maturity:** `ANALYTICAL_SCREENING_ONLY`

## What this screen computes

- Sensible thermal storage capacity: `m · cp · ΔT`.
- Latent heat absorbed at the evaporation interface: `m_evaporated · h_fg`.
- Pump electricity: `P_electric · duration · duty_fraction`.
- Evaporative water draw and remaining declared discretionary water budget.

It does **not** estimate the fraction of heat that reaches an occupied zone, predict comfort, estimate evaporation from weather, calculate wet-bulb conditions, or qualify water source, rights, water quality, instruments or restrictions. Its output must leave useful occupied-zone cooling as `UNKNOWN`.

## Input and units

All fields are SI or explicitly named:
- water volume: m³;
- water temperature swing: K;
- evaporation rate: L/h;
- exposure duration: h;
- pump input power: kW;
- pump duty: dimensionless [0,1];
- discretionary water budget: L;
- density: kg/L;
- specific heat: kJ/(kg·K);
- latent heat: kJ/kg.

Constants are supplied in the case file rather than hidden in code so their provenance and assumptions remain visible.

## Known-answer reference example

The committed example is synthetic: 10 m³, 1 K swing, evaporation 0.5 L/h for 8 hours, pump 0.12 kW at 50% duty, 5 L discretionary budget, 1 kg/L density, 4.186 kJ/(kg·K) specific heat and 2,450 kJ/kg latent heat.

Expected calculations (rounded for display):
- sensible storage capacity: 11.6278 kWh;
- evaporated water: 4.0 L;
- latent heat at the evaporation interface: 2.7222 kWh;
- pump electricity: 0.48 kWh;
- remaining declared water budget: 1.0 L.

These quantities are separate. They must not be netted into a "cooling efficiency", because the useful building-side heat transfer is unknown. The example water source, water quality, weather and instruments are intentionally unqualified, so the operational gate must remain blocked.

## Claim rules

- Sensible capacity is not delivered cooling.
- Latent heat at the water surface is not delivered cooling.
- Pump electricity must be reported separately.
- No source class can bypass source rights or quality checks.
- No missing water term is treated as zero.
- No calculation from this screen is a field measurement, EnergyPlus output or model validation.
