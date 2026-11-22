# City product-chain corpus v0.1

This 24-case synthetic corpus validates receipt-chain completeness from a service/product-family requirement through a declared lifecycle stage sequence. It is not a productive-capability solver, inventory authority, qualification service or commissioning authority.

## Stages

The owner-declared profile may require: design, process, capability, qualified_material, available_supply, delivery_receiving, installation, commissioning, maintenance and recovery. Stage definitions bind their expected owner, contract and source revision.

## Evidence rule

A receipt is satisfied only when requirement ID, product family, stage, owner, contract, source revision, evaluation scope, design/BOM revision and subject digest match the exact requirement. Evidence must be current and result must be `satisfied`. A known blocked/rejected/unavailable result blocks the chain; a missing or unverifiable result leaves it Unknown.

## No local-capability inference

The declared sourcing class (local, regional, imported, recovered, retained) is carried through for comparison. The oracle never declares local productive capability: that remains with owner-native capability and supply receipts and must be independently verified.

## Output

Each service/district/product-family requirement remains separate with each stage's state and reason. Overall outcomes are `ReadyForDeclaredLifecycle`, `Blocked`, `Unknown` or `Unsupported`. No citywide weighted coverage score is emitted.

## Claim ceiling

Synthetic receipt binding and lifecycle-chain semantics only. No real factory, supplier, material, workforce, facility, legal compliance, installed asset or operational city service is qualified.
