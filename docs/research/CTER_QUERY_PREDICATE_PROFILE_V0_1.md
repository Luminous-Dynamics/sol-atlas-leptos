# CTER Deterministic Query Predicate Profile v0.1

**Tracking:** [CTER-QUERY-SEMANTICS-001 #972](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/972)  
**Composes:** #960 egress binding, #962 private coverage, #967 VDS/private proof selection; Mycelix #1215 typed receipt algebra.  
**Status:** bounded reference evaluator and synthetic mutation suite; query commitment recomputation and production adapters remain future work.  
**Research date:** 2026-11-20.

## 1. Why set equality is not query correctness

A complete partition can still be wrong if the custodian assigns scope labels without evaluating the exact requested selection predicate. The partition might contain every committed frontier object and still:
- label a non-matching item \`in_scope\`;
- label a matching item \`out_of_scope\`;
- convert missing attributes into a false non-match;
- use a different selection rule from the one approved in the authorization request.

CTER must distinguish three operations:

1. **Query semantics:** determine whether each frontier item matches the exact selection predicate.
2. **Disposition:** determine whether a matching item was included, withheld, unavailable or unresolved.
3. **Authorization:** determine whether the requested package, recipient, purpose and use are authorized by the owner of that authority.

These stages are related but must not collapse into a global \`verified=true\` flag.

## 2. Paradigm: a query-semantics intermediate representation

The v0.1 reference profile uses the versioned identifier:

\`urn:luminous:cter:predicate-json-v0.1\`

Its AST is a narrow, pure data-selection language, not an authorization language and not an unrestricted query runtime. An upstream query adapter can compile an approved request into this form. The coverage evaluator consumes the same normalized predicate rather than reinterpreting arbitrary SQL, UI filters, search syntax or agent-generated code.

Supported operators:
- \`all\`, \`any\`, \`not\`
- \`eq\`, \`in\`, \`exists\`

The profile restricts field paths to one ASCII JSON object-member segment, such as \`/jurisdiction\`. Operands are JSON scalars. Boolean child arrays contain 1-32 nodes, \`in\` lists contain 1-64 values, and recursive expression depth is capped at 32. Array traversal, nested paths, wildcards, regular expressions, custom functions, user-defined code and external data lookups are outside v0.1.

This bounded language is deliberately less expressive than general query systems. A small, independently implemented predicate profile is easier to review, cross-check and later prove than an opaque expression string with environment-dependent extensions.

## 3. Three-valued match semantics

For each frontier item, predicate evaluation returns \`TRUE\`, \`FALSE\` or \`UNKNOWN\`.

| Operation | Rule |
|---|---|
| \`eq\` | Exact JSON scalar type-and-value equality; missing input/member is UNKNOWN |
| \`in\` | Exact type-and-value match against the committed scalar list; missing input/member is UNKNOWN |
| \`exists\` | Missing query-input object is UNKNOWN; member present is TRUE; available object with member absent is FALSE |
| \`not\` | TRUE/FALSE invert; UNKNOWN remains UNKNOWN |
| \`all\` | Any FALSE makes FALSE; otherwise any UNKNOWN makes UNKNOWN; otherwise TRUE |
| \`any\` | Any TRUE makes TRUE; otherwise any UNKNOWN makes UNKNOWN; otherwise FALSE |

The order of evaluation cannot change the result. These are explicit strong-Kleene-style three-valued rules for this profile, not whatever null/missing coercion a database happens to use.

The resulting scope state is mandatory:

| Predicate result | Required \`scope_state\` |
|---|---|
| TRUE | \`in_scope\` |
| FALSE | \`out_of_scope\` |
| UNKNOWN | \`unknown\` |

Dispositions are checked separately:
- \`in_scope\` may be \`included\`, \`withheld\`, \`unavailable\` or \`unresolved\`;
- \`out_of_scope\` must be \`excluded\`;
- \`unknown\` must remain \`unresolved\`.

An unavailable source or missing required attribute must not be converted into a false predicate result. It remains UNKNOWN until the query input is resolved or the record is requalified.

## 4. Synthetic fixture

The committed baseline query is:

\`\`\`json
{
  "op": "all",
  "args": [
    {"op": "eq", "path": "/dataset", "value": "cadastre"},
    {"op": "eq", "path": "/jurisdiction", "value": "ZA"}
  ]
}
\`\`\`

Private item attributes are provided to the reference evaluator:
- item-a: cadastre / ZA -> TRUE, included;
- item-b: cadastre / ZA -> TRUE, withheld;
- item-c: cadastre / ZA -> TRUE, unavailable;
- item-d: cadastre / US -> FALSE, excluded;
- item-e: cadastre with jurisdiction missing -> UNKNOWN, unresolved.

The public verifier view contains none of these per-item attributes or object references. They remain within the private coverage record and its authorized custodian-side evaluation boundary.

The synthetic fixture currently includes 37 mutation cases: the prior 32-case coverage suite plus five predicate/partition controls. The composed-release corpus now has 21 cases, including query-profile and predicate substitution attempts.

## 5. Exact cross-record binding

The private query has a \`query_profile_ref\` and a predicate AST. The following references are required to agree:
- private query profile and receipt projection profile;
- private query reference and egress authorization's \`coverage_query_ref\`;
- private query commitment and egress authorization's \`coverage_query_commitment\`;
- private query profile and egress authorization's \`coverage_query_profile_ref\`.

This prevents the same package/recipient/request from being reused under a different declared query-semantics profile.

The commitment boundary is not yet complete. The reference evaluators currently validate SHA-256 digest shape and cross-record equality; they do **not** recompute the query commitment from canonical predicate bytes. A later increment must define and test the precise commitment preimage, domain separation, canonicalization and cross-language hashing. Until that exists, a structurally consistent query hash must not be described as a cryptographically verified predicate commitment.

## 6. Standards and industry comparison

### JSONPath (RFC 9535)

[RFC 9535](https://www.rfc-editor.org/rfc/rfc9535.html) standardizes JSONPath query expressions, including filter selectors and logical expressions. Its filter expressions are side-effect-free, and it defines an extension mechanism for additional functions. This is a useful adapter/comparison target, not a reason to implement arbitrary expressions inside the CTER reference evaluator.

A future JSONPath adapter must pin the RFC version, reject unsupported function extensions, and show that the compiled CTER predicate has exactly the same selection set on independent fixtures. Do not assume syntactic translation implies semantic equivalence.

### OPA / Rego

[OPA decision logs](https://www.openpolicyagent.org/docs/management-decision-logs) already capture policy queries, inputs, bundle metadata and decision identifiers for auditing and offline debugging. CTER should consume policy/authority evidence where appropriate, not recreate a policy decision point.

The division is:
- query predicate: which source items satisfy the requested selection;
- OPA/Mycelix authority: whether this operation and disclosure are permitted;
- CTER: whether the resulting exact package and private partition correspond to that approved query and use.

A query match does not grant authority; an authorization decision does not prove that a custodian correctly evaluated the data-selection predicate.

### JSON Canonicalization Scheme (RFC 8785)

[RFC 8785 JCS](https://www.rfc-editor.org/rfc/rfc8785.html) describes deterministic JSON serialization for cryptographic operations. It is an Informational RFC, not an Internet Standards Track specification, and constrains inputs to the I-JSON subset, including IEEE-754 binary64-compatible numbers and no duplicate property names.

The v0.1 predicate uses only simple scalar operands and restricted object-member names, but the commitment profile must still specify:
- the exact object being canonicalized (profile reference plus AST, not a display string);
- duplicate-key rejection before parsing;
- number representation and permitted numeric range;
- Unicode preservation and key ordering;
- domain separation between predicate, query and request commitments;
- cross-language test vectors for Rust and Node implementations.

The reference evaluator does not yet perform this cryptographic recomputation.

## 7. Benchmark plan

Run 1k, 10k, 100k and 1M frontier items with 1, 10 and 100 predicates per query. Vary:
- TRUE/FALSE/UNKNOWN rates and attribute-missing rates;
- \`all\`/\`any\` tree shape, depth and fan-out;
- scalar type confusion (string vs number vs boolean vs null);
- unknown operators, malformed paths and excessive depth;
- late attribute corrections and query-profile upgrades;
- withheld/unavailable rates and proof audience;
- online/offline and independent Rust/Node evaluation.

Report separately:
- predicate evaluation p50/p95/p99 and max;
- parsing/compilation cost;
- peak memory and allocations;
- unknown rate and its causes;
- false-match and false-nonmatch counts on the frozen ground truth;
- profile/version mismatch rejection;
- Rust/Node agreement;
- query commitment generation/verification and canonicalization cost once implemented.

Targets, not achieved results:
- zero FALSE items accepted as \`in_scope\`;
- zero TRUE items accepted as \`out_of_scope\`;
- zero UNKNOWN items silently narrowed to TRUE/FALSE;
- zero unsupported operators accepted;
- 100% independent implementation agreement on the same frozen valid/invalid AST corpus.

Do not use one weighted score. Correctness and fail-closed behavior are gates; performance is reported separately.

## 8. Next proof and commitment increment

Before a VDS or private proof can establish query completeness, the verifier must know what predicate was committed. Required next steps:
1. define canonical predicate bytes and domain-separated commitment preimage;
2. implement hash recomputation in Rust and Node;
3. publish shared test vectors (property order, Unicode, duplicate keys, number edge cases, null and missing);
4. add predicate substitution and canonicalization mismatch controls;
5. compare the reference AST against pinned JSONPath and an OPA/rego adapter on an identical source corpus;
6. include the exact predicate/profile commitment in the host-resolved typed receipt and proof statement.

This is the prerequisite for claiming a proof that a private partition covers all items matching a query. A VDS inclusion proof over an opaque query hash alone cannot show that the custodian actually evaluated the intended predicate correctly.

## 9. Claim ceiling

The current implementation checks the exact v0.1 AST against the private query-input snapshots present in the fixture. It does not prove that the source owner supplied truthful or complete attributes, that the frontier covers all real-world evidence, that the query is legally appropriate, that the predicate commitment has been cryptographically recomputed, or that the disclosure is authorized.

Symthaea may propose alternate predicates and counterexamples read-only. It cannot choose the authoritative query or grant authority. Mycelix remains the identity, consent, policy, signed receipt and currentness owner. CTER remains the consumer-side query/package/partition binding and qualification layer.
