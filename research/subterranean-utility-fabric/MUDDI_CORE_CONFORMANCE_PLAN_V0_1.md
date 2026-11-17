# MUDDI Core Conformance Plan — SUF Logical Profile v0.1

**Owner:** Sol-Atlas #755  
**Standard:** OGC 23-024 MUDDI Part 1 v1.0  
**Status:** partial traceability; not conformant

## 1. Normative test structure

MUDDI Core conformance is not a class-name checklist. The logical model must implement all four Core requirements and their abstract tests:

- A.1 `/req/core/classes-concepts`: every Core UML class and referenced concept.
- A.2 `/req/core/classes-associations`: same source, target, direction, roles and maximum multiplicities as the UML model.
- A.3 `/req/core/classes-attributes`: all required attributes with names, definitions, types and maximum multiplicities.
- A.4 `/req/core/concrete-value-types`: replace every AbstractValueType with a concrete implementation type, especially geometry.

The normative UML model takes precedence where prose and model differ. The official OGC 1.0 folder contains the XMI representation; it is the reference input for a full crosswalk.

## 2. Current evidence and status

| Test | Current evidence | Status |
|---|---|---|
| A.1 Concepts | Ten Core concepts mapped in `muddi-core-crosswalk-v0.1.json`; concrete profile type names added to synthetic records; abstract base/concrete subtype distinction recorded | Partial; requires UML-level review |
| A.2 Associations | Local fixture has illustrative network links and observation references; exact UML association roles/directions/multiplicities have not been extracted and matched | Not mapped exhaustively |
| A.3 Attributes | MUDDIObject IDs/geometry, MUDDIAsset owner, MUDDIEvent targetObject/targetProperty/validTime, MUDDISpace extent, Network commodityType and NetworkAsset utilityType mapped | Partial; complete inherited/defined attribute audit remains |
| A.4 Concrete value types | Synthetic geometry has Point/LineString/Polygon coordinate arrays and an abstract local frame | Not complete; choose and test ISO 19107 concrete geometry types and coordinate-reference profile |

No MUDDI Core conformance claim may be made until every test is complete and independently reviewed.

## 3. Mandatory attribute crosswalk

The machine-readable crosswalk records required attributes and local names. The principal obligations include:

- MUDDIObject: objectID, recordID, systemID; sf_geometry is conditional/repeating.
- MUDDIAsset: assetOwnerID.
- MUDDIEvent: targetObject, targetProperty, validTime.
- MUDDISpace: extent.
- Network: commodityType.
- NetworkAsset: utilityType.
- NetworkConveyance, NetworkLink, NetworkNode and NetworkAccessory inherit the required NetworkAsset/MUDDIAsset/MUDDIObject fields as applicable.

The LogicalName table maps renamed/localized profile fields back to the conceptual names and records implementation names/default symbols.

## 4. Concrete subtype rule

MUDDI Core classes in the conceptual model are abstract. The fixture therefore records both:
- `muddi_class`: the nearest MUDDI conceptual base class; and
- `concrete_profile_type`: the synthetic local specialization used in the fixture.

A fixture record is not a claim that an abstract UML class is instantiated directly. The final logical profile must define concrete local types and preserve their inheritance/mapping to MUDDI.

## 5. Geometry work still required

The synthetic Point/LineString/Polygon arrays are only fixture placeholders. They do not yet implement a selected ISO 19107 / OGC Simple Features encoding or a production CRS/datum policy. The target logical model should:

1. declare concrete geometry types for each concrete profile class;
2. distinguish topological link geometry from asset geometry and site extent;
3. specify 2D/3D and vertical datum/epoch behavior;
4. define a permitted encoding (candidate: JSON-FG for extended CRS/3D, plus a documented 2D-compatible exchange profile where appropriate);
5. bind transforms and uncertainty to exact source and output revisions;
6. pass positive/negative A.4 value-type tests.

## 6. Association work still required

Use the official OGC XMI/UML model to extract every Core association and inheritance relation. Build an exact role/direction/multiplicity crosswalk. Do not promote illustrative fixture links such as `from_node`, `to_node`, `member_asset` or `conflicts_with` to MUDDI normative associations until the UML mapping demonstrates equivalence. These local links may remain explicit profile extensions.

## 7. Negative-control families

- missing or colliding objectID / recordID / systemID;
- wrong system/provider namespace;
- missing assetOwnerID, utilityType, commodityType, targetObject, targetProperty, validTime or extent;
- absent/invalid logical-name row;
- missing concrete profile subtype for an abstract MUDDI class;
- invalid geometry type/dimensionality/CRS/datum/epoch/units;
- missing or wrong association endpoint/direction/role/multiplicity;
- stale route geometry promoted to current;
- withheld/unobserved represented as absence or zero;
- unauthorized restricted geometry disclosure.

## 8. References

- OGC MUDDI Part 1 v1.0: https://docs.ogc.org/is/23-024/23-024.html
- Official MUDDI model files: https://schemas.opengis.net/muddi/1.0/
- OGC 26-050 building-block framework (non-normative): https://docs.ogc.org/techpaper/26-050/26-050.html