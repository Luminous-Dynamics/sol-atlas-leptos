# Exact-Rational MATPOWER Two-Bus DC-OPF Proof v0.1

**Status:** narrowly scoped exact proof checker for one frozen rational source fixture. This is not a general OPF solver or a physical-grid validation tool.

## What is proved

For the byte-exact MATPOWER case `two_bus_binding_line.m` (Git blob SHA-1 `6f835a443a6b33a11fade361a437c7cfab9d0182`), the crate reconstructs the supported convex single-period DC-OPF LP with checked rational arithmetic and verifies:

- Exact bus active-power balance at both buses.
- Exact reference-angle equality.
- Exact generator and symmetric branch-flow bounds.
- Non-negative inequality multipliers.
- Exact stationarity in both bus angles and both generator dispatch variables.
- Exact complementary slackness for both line limits and all generator bounds.
- Exact equality of primal objective and dual lower bound: **2200 $/h**.

The binding branch flow is `2/5 pu`; dispatch is `2/5 pu` and `3/5 pu`; the upper branch-limit multiplier is `2000` in the declared per-unit Lagrangian convention.

## Mathematical convention

With (b=1/x=10), base (S_b=100) MVA, load (d=1) pu and line limit (ar f=2/5) pu:

`h_1=b(\theta_1-\theta_2)-p_1=0`

`h_2=b(\theta_2-\theta_1)-p_2+d=0`

`f=b(\theta_1-\theta_2), \, -\bar f\le f\le\bar f`

`0\le p_1,p_2\le 2`

The objective in per-unit dispatch variables is `1000 p_1 + 3000 p_2` $/h. The Lagrangian convention is:
`L = c'p + lambda_1 h_1 + lambda_2 h_2 + mu_U(f-fbar) + mu_L(-f-fbar) + mu_Lo(lo-p) + mu_Up(p-hi)`.

Certificate:
- `theta_1=0`, `theta_2=-1/25`, `p_1=2/5`, `p_2=3/5`.
- `lambda_1=1000`, `lambda_2=3000`, `mu_U=2000`.
- All other inequality multipliers are zero.

The exact dual objective is `lambda_2*d - mu_U*fbar = 3000 - 800 = 2200 $/h`.

## Fail-closed design

- The exact Git blob SHA-1 is recomputed from source bytes.
- The source must match the one explicitly admitted MATPOWER fixture and supported data rows.
- Every rational operation uses checked `i128` arithmetic and normalized numerator/denominator form.
- Overflow, unsupported source bytes, failed stationarity, nonzero complementary products, or a nonzero primal-dual gap are hard errors.
- No f64 conversion, solver-provided success flag or external boolean is used.
- The known-answer values are computed by the checker; the receipt contains exact fraction strings only.

## Run

```sh
cargo test --manifest-path tools/exact-dc-opf-proof/Cargo.toml --locked --offline
cargo run --manifest-path tools/exact-dc-opf-proof/Cargo.toml --locked --offline
```

## Claim ceiling

This is an exact optimality proof for one source-bound, rational, convex two-bus DC-OPF LP. It does not certify arbitrary MATPOWER cases, floating-point solver output, AC-OPF, losses, multi-period commitment, uncertainty, dynamics, protection, grid-code compliance, physical equipment or field operation. Expanding this checker requires a separately reviewed exact parser/model compiler and a general certificate format; no silent broadening is allowed.
