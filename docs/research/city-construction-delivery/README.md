# City construction delivery corpus v0.1

33 synthetic cases test one bounded contract: can a qualified, available product lot be delivered, unloaded, received and accepted for a declared above-ground work package in the requested time window?

## States

- `DeliveryFeasible`: all declared prerequisites are supported, but the full accepted receipt has not occurred.
- `AcceptedDelivery`: full quantity received, custody accepted and independent inspection passed.
- `Partial`, `Late`, `Blocked`, `Unknown`, `Rejected`, `Disputed`, `NotComparable`, `Unsupported` remain distinct.

## Critical semantics

- The adapter consumes owner facts; it does not invent a supplier, inventory stock, grade, legal route, vehicle payload, lift capacity, site access window or receiving workforce.
- All quantity comparisons require exact unit equality. No implicit unit conversion or density assumption exists.
- Lot allocation is summed across all candidate deliveries. If planned demand exceeds declared available quantity, every affected candidate is blocked; no order-dependent allocation is guessed.
- Receiving-bay and laydown reservations are summed by resource and time slot. A shared-resource overbook blocks all contenders.
- Time uses integer ticks and half-open windows. An acceptance that finishes exactly at the endpoint is Late.
- Route and vehicle feasibility are inputs with evidence state and revision, not recomputed path claims.
- A delivery only becomes AcceptedDelivery after the required physical receipt/custody and independent inspection evidence. A plan or in-transit shipment is not accepted inventory.
- Imported supply can be delivered; the adapter does not claim it is local productive capability.
- Design/BOM revision mismatch blocks the candidate. The old material lot is not silently substituted into the new design.

## Owner boundary

This is a Sol-Atlas cross-domain conformance adapter. Mycelix #4814 remains authoritative for supplier, inventory and production dependencies; Mycelix #5149 for lot custody, inspection and accepted receipt; Symtropy route/vehicle/resource and construction accepted-output owners remain authoritative for those facts. Exact production schemas are not guessed; #606 owns the owner-native adapter discovery.

## Claim ceiling

Passing the corpus validates only this synthetic delivery-feasibility logic. It does not prove actual supplier stock, route legality, vehicle certification, lifting safety, site permissions, material suitability, construction programme feasibility or an accepted real delivery.
