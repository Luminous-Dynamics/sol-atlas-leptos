# Resident spatial-intent graph v0.1

**Study:** HOME-SPATIAL-INTENT-GRAPH-001  
**Parent:** [Sol-Atlas #824](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/824) / Home Freedom #800  
**Maturity:** `SYNTHETIC_SYMBOLIC_GRAPH_ONLY`  
**Execution:** `NOT_GEOMETRICALLY_VERIFIED`

## Why a graph before a floor plan

A room list and area schedule do not fully describe why a home works. Residents care about relationships: quiet work away from cooking, private sleeping away from guests, a kitchen that opens directly to an outdoor room, step-free movement to key spaces, and the ability to share a room at different times without concurrency conflicts.

The spatial-intent graph separates three layers:
1. **Activity intent:** what residents want to do and the requirements they state.
2. **Space allocation:** which functional spaces support those activities.
3. **Relationship/access constraints:** how the spaces must relate, while verification remains owned by downstream geometry and specialist checks.

This gives the later geometric search a set of typed proof obligations. A renderer should not be able to silently change a non-negotiable requirement or convert an unknown relationship into PASS.

## Graph semantics

- Activities and spaces are separate entities.
- One activity can use a space; several activities can share a space if the temporal/occupancy assumptions support it.
- A dedicated-space activity must not be merged into an incompatible space.
- Adjacency is not the same as direct access. Proximity is not the same as privacy. A symbolic edge is not physical proof.
- Acoustic separation, visual privacy, step-free routes and shared services remain distinct relationship types.
- Every relationship carries its verification state, owner and evidence reference. This v0.1 fixture sets physical verification to UNKNOWN.
- Hard constraints and preferences remain separate; preferences are never used to compensate for a hard-constraint failure.

## First paired topology fixture

The companion fixture maps the same resident brief into:
- a shaded courtyard topology with a private outdoor room;
- a compact row-home topology with a private service yard.

The internal area schedules remain 82 m² and 78 m², respectively. The graphs preserve six activities, four relationship requirements, two symbolic access routes and five hard-constraint bindings per candidate.

The fixture is deliberately **not a floor plan**. It has no x/y coordinates, wall lengths, door/window geometry, room polygon, turning clearance, slope, fire separation, daylight or acoustic performance model. All relationship/access/hard-constraint verification states remain UNKNOWN, and both options remain unranked.

## P0 qualification rules

- Unique stable activity, space, assignment, relationship and route IDs.
- Every brief activity maps to a registered space.
- Every dedicated-space activity has its own space assignment.
- Every source relationship maps to the correct pair of activity and assigned space IDs.
- Every non-negotiable brief constraint remains represented exactly once.
- Every declared access route uses registered spaces.
- Symbolic graphs cannot promote physical relationship/access/hard-constraint states above UNKNOWN.
- Room-program areas reconcile, with indoor and outdoor areas kept separate.
- Candidate order and node/edge order must not alter semantic results.
- Missing, duplicate, orphan or contradictory edges are rejected.

## Next geometric stage

P1 can introduce a bounded grid or rectangle-based layout search with explicit site envelope, room dimensions, wall/door/window/opening constraints, corridor/egress, step-free route widths and orientation. It must return multiple alternatives, rejected constraints and Pareto outcomes. The P1 geometry still needs specialist checks and professional/authority review.

## Industry / interoperability position

Use IFC 4.3 for physical space/element exchange and IDS for required information delivery. Keep Brick / ASHRAE 223 / Haystack operational semantics in Sol-Atlas #704. Compare against architect-led room data sheets and adjacency matrices, graph-based space planning, Finch3D rule-bound plan iteration, and Autodesk Forma's supported layout exploration. A symbolic graph qualifier is not a competitor to structural, acoustic or geometric solvers.

## Claim ceiling

No room adjacency is physically confirmed. No accessibility, fire, acoustic, daylight, circulation, code, structural, thermal or resident outcome claim is established. The output only confirms internal symbolic graph structure.
