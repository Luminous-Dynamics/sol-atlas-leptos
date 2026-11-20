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

**Implemented on the research branch:** the Rust reference evaluator and independent Node oracle recompute this commitment. Rust uses `serde_jcs` 0.2.0 and `sha2` 0.10.9; Node uses an independent JCS serializer and `node:crypto`. The four shared golden vectors are frozen in `benchmarks/territorial-integrity/egress/query-predicate-commitments-v0.1.json`. A local Node 22.16.0 check matched the four canonical byte strings and SHA-256 digests, including key-order equivalence, decomposed Unicode and the `1e-7` number form. This was an inline local check, not execution of the committed workflow oracle or Rust tests.

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

## 8. Canonical commitment and strict raw-input boundary

The query predicate commitment is now:

\`SHA-256("luminous.cter.query-predicate.v0.1\\0" || JCS({profile_ref, predicate}))\`

The canonical preimage includes the semantics profile, not only the AST. This prevents the same expression from being silently reinterpreted under a different predicate language. Rust computes it with pinned \`serde_jcs = 0.2.0\` and \`sha2 = 0.10.9\`; the independent Node oracle uses ECMAScript JSON number/string formatting, UTF-16 key ordering and Node's SHA-256 implementation.

The shared four-vector corpus covers:
1. the baseline all/equality query;
2. reordered JSON member names producing the same canonical bytes and digest;
3. decomposed Unicode preserved without normalization;
4. IEEE-754 exponent rendering for \`1e-7\`.

Rust tests assert both canonical bytes and digests. The independent Node script does the same, checks query/receipt/public-projection digest agreement and rejects predicate substitution without commitment refresh. A local Node 22.16.0 inline run matched 4/4 vectors, demonstrated key-order invariance and predicate mutation sensitivity, and rejected an unsafe integer. The committed Node 22 workflow and Rust tests remain pending exact-head execution.

Rust also exposes strict raw-input entry points for private coverage, egress manifests and composed releases. They parse recursively with a duplicate-member-detecting Serde visitor and reject integers outside the exact IEEE-754 safe-integer range before JCS commitment evaluation. These production-facing adapters should use the raw \`*_json\` entry points rather than first deserializing into \`serde_json::Value\`, because ordinary JSON value parsing can discard duplicate-member evidence.

This closes the *predicate-integrity* gap, not the *query-input provenance* gap. A committed predicate establishes the rule, but the per-item \`query_input\` attributes remain a separate input set that must be bound to exact frontier source records and an owner/resolver attestation.

## 9. Next proof and source-input integrity increment

The next requirement is tracked in [CTER-QUERY-INPUT-ATTESTATION-001 #973](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/973). The predicate commitment must be paired with a separate, domain-separated commitment to the sorted per-item query-input snapshot, binding:
- exact frontier reference, version and commitment;
- exact query and predicate profile/commitment;
- versioned attribute projection schema;
- each item reference, availability state, private attribute projection and source-object/content-digest bindings;
- canonicalization profile and snapshot commitment.

The custodian snapshot hash proves only what was frozen, not that its attributes are true. Mycelix/source-owner resolution must authenticate the snapshot under a distinct typed receipt before any result can be described as source-owner attested. Predicate result, source-origin binding, owner attestation and authorization remain separate facts.

The VDS/private proof qualification in #967 must then cover the relation between the owner-attested input snapshot, the predicate result, the complete frontier partition and the package's terminal source leaves. An inclusion receipt over an opaque query hash alone is not enough.

## 10. Claim ceiling

The current evaluator recomputes the exact v0.1 predicate commitment and checks TRUE/FALSE/UNKNOWN results against the private query-input snapshots supplied to it. The raw Rust entry points reject duplicate JSON object members and unsafe integers. This establishes integrity of the declared rule and strict parsing of the consumed record; it does not establish truth or completeness of the per-item attributes.

It does not prove that the source owner discovered every real-world fact, that the frontier covers the real world, that the query is legally appropriate, or that disclosure is authorized. The per-item source-input snapshot and its owner/resolver attestation are the remaining qualification layer tracked by #973.

Symthaea may propose alternate predicates and counterexamples read-only. It cannot choose the authoritative query or grant authority. Mycelix remains the identity, consent, policy, signed receipt and currentness owner. CTER remains the consumer-side query/package/partition binding and qualification layer.


## 10. Machine-readable workload profile

The benchmark matrix is committed at \`benchmarks/territorial-integrity/egress/predicate-workloads-v0.1.json\`. It freezes:
- 1k/10k/100k/1M record frontiers;
- 1/10/100 predicates per batch and expression depths 1/4/8/16/32;
- missing-attribute rates of 0/1/10/50 percent and several TRUE/FALSE/UNKNOWN distributions;
- six operator workloads;
- Rust, independent Node, RFC 9535 JSONPath-subset and OPA/Rego adapter candidates;
- 22 adversarial semantics, canonicalization and cross-implementation faults.

The workload separates predicate evaluation, parse/compile, canonicalization, commitment generation and commitment verification cost. No performance result or candidate winner is selected.


## 11. Current executable suite counts

- Private coverage: 38 synthetic cases (1 valid, 5 incomplete, 5 unknown, 27 deny).
- Composed release: 21 cases (2 valid structural controls, 19 denials).
- Egress binding: 35 cases (1 valid baseline and 34 adverse/indeterminate/review controls).
- Predicate commitment golden vectors: 4 cross-language canonical-byte/digest vectors.

These are fixture specifications and source-level conformance tests. The local Node 22.16.0 golden-vector check is a separate result. No Rust build/test, exact-head Node workflow pass, owner-attested input snapshot, COSE/VDS proof or production release is claimed.


## Query input provenance composition (#973)

The predicate commitment is now complemented by a separate query-input snapshot commitment. The first reference slice freezes the exact per-item attribute projection, source references, source content-digest commitments, availability state, and the frontier/query/profile identity. Scope is evaluated against that snapshot, not mutable partition attributes.

See [CTER_QUERY_INPUT_ATTESTATION_PROFILE_V0_1.md](CTER_QUERY_INPUT_ATTESTATION_PROFILE_V0_1.md) and [#973](https://github.com/Luminous-Dynamics/sol-atlas-leptos). The predicate and snapshot hashes are domain-separated and cannot be substituted for one another.

This does not prove source truth. The custodian creates the snapshot, source-content digests are not recomputed over fetched bytes, and the source owner/Mycelix has not yet issued or confirmed an attestation receipt for this new snapshot type. That is a separate interoperability and trust qualification requirement.


## 12. Current suite inventory after #973

The predicate rule is now paired with the separate query-input source snapshot. The current branch's frozen suite sizes are:
- predicate/private coverage: 44 cases;
- composed release: 24 cases;
- egress binding: 39 cases;
- predicate JCS golden vectors: 4;
- source-input snapshot golden fixtures: 2 (private and composed).

Earlier 38-case predicate-suite and 37/37 fixture-mirror statements in this document describe the pre-snapshot revision. They must not be read as execution results for the current 44-case suite. The latest current suite is source-present; exact-head Rust and committed Node 22 workflows remain pending.
