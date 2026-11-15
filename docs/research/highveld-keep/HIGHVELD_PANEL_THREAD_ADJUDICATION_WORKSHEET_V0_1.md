# Highveld Panel Thread — Independent Gold-Label Adjudication Worksheet v0.1

**Status:** template only; no labels have been adjudicated  
**Review packet:** `HIGHVELD_PANEL_THREAD_REVIEW_PACKET_V0_1.json`  
**Source corpus version:** `0.1.0`  
**Source corpus Git blob:** `f4a7819798d0d19b7114a9e41aa7ff1215f69410`  
**Parent issue:** [#878 — HPT cross-owner replay](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/878)

## Independence and limitations

The source corpus `HIGHVELD_KEEP_PANEL_THREAD_CORPUS_V0_1.json` contains provisional expected dispositions and labels. This packet deliberately omits those fields and also omits criticality labels to reduce anchoring during independent review.

The source gold-label corpus is public. Therefore this packet **does not establish a cryptographically hidden or guaranteed-blind evaluation**. Reviewers should not inspect the source corpus or discuss labels during first-pass review, but any published benchmark must disclose that the original labels are publicly retrievable. A true hidden holdout requires access control or a separate trusted evaluation custodian.

No case in this worksheet is a real design, material, robot, installation or occupancy decision. Review only the sufficiency of evidence stated in the synthetic scenario.

## Review protocol

1. Appoint two independent reviewers with documented relevant competence. At minimum, the panel should collectively cover engineering configuration/change, manufacturing quality/material traceability, metrology, and industrial robot/workcell safety. A reviewer must declare conflicts and areas outside their competence.
2. Give each reviewer the JSON packet only. Do not reveal the corpus's provisional disposition, reason codes, requalification domains, or the other reviewer's answers.
3. For every case, each reviewer records a disposition, a concise evidence-based rationale, reason codes, missing proof obligations, affected requalification domains, and the minimum admissible evidence that could change the result.
4. A third reviewer resolves disagreements, records both original judgments, explains the resolution, and may return a case as ambiguous/unlabelable pending a better scenario.
5. Preserve all negative, disputed and dissenting judgments. Do not overwrite the first-pass labels.
6. Only after adjudication is complete should the provisional source labels be compared with the independently adjudicated result. Record every disagreement and whether the source label changes.
7. Freeze an adjudication manifest binding the exact review-packet digest, source-corpus Git blob, protocol version, reviewer identities/competence attestations, timestamps, individual labels, disagreements, resolution and final labels. The evaluator must not accept a status string as proof of adjudication.

## Disposition semantics

| Disposition | Adjudication meaning |
|---|---|
| `REVIEW_ONLY` | Evidence is internally coherent for the narrow synthetic review question; no authorization or physical acceptance follows |
| `REQUALIFICATION_REQUIRED` | A prior evidence chain exists, but changed context makes defined proof obligations require re-evaluation |
| `BLOCKED_UNKNOWN` | A required fact or competent evidence is missing or cannot be established |
| `BLOCKED_CONFLICT` | Material evidence conflicts or the asserted identity/context does not reconcile |
| `BLOCKED_AUTHORITY` | The claimed approval/release lacks the required scoped authority or permission |

A reviewer must not infer that these states are linearly interchangeable. Record the reason for the selected state and any safe, admissible transition.

## First-pass review register

The case text below is copied from the redacted packet. Expected labels and source reason codes are intentionally absent.

| Case | Scenario | Preconditions and mutation |
|---|---|---|
| HPT-001 | Complete synthetic design-to-as-built reference chain | A single synthetic panel has exact design and component revision identities.; The IFC/IDS projection, geometry reference, process plan and material-lot references resolve to those exact revisions.; Tool/workcell capability, execution, independent metrology, installation and as-built references are present and current.; All owner-provided references state their own disposition and schema/profile revision. Mutation: No adverse mutation; evaluate the complete reference trace as submitted. |
| HPT-002 | Panel design revision changes after process and inspection evidence | The original design revision has process, inspection and installation references.; A downstream manufacturing package is bound to the original design digest. Mutation: Substitute a newer design revision while retaining process and inspection references from the prior generation. |
| HPT-003 | IDS information pass with a conflicting as-built geometry observation | Required IFC properties satisfy the declared IDS profile.; An independent survey observation reports a station or connection datum outside the declared geometric tolerance. Mutation: Attempt to treat the IDS information pass as proof that physical geometry conforms. |
| HPT-004 | Lossy IFC or robot projection drops a critical datum or tolerance | The source design includes a datum, tolerance or connection requirement used by a downstream manufacturing or robot task.; The transformation declares a lossy or unsupported semantic projection. Mutation: Remove the critical datum/tolerance from the projected work package while leaving the source model unchanged. |
| HPT-005 | Supplier lot identity differs from the approved material requirement | The work package requires an exact material grade and approved product/lot scope.; The supplier delivery or receiving record carries a different physical lot identity or grade. Mutation: Substitute a same-named or same-family lot without an owner-qualified equivalence and approval reference. |
| HPT-006 | Material is delivered but custody or receiving acceptance is incomplete | A dispatch or delivery assertion exists.; A required custody handoff, receiving inspection or acceptance disposition is missing. Mutation: Treat the delivery event as proof that the exact material lot and quantity are accepted inventory. |
| HPT-007 | Work package has complete bill of materials but no current reservation | The material lot and planned quantity are otherwise correctly identified.; No current owner-native reservation/availability evidence covers the exact site, quantity and execution window. Mutation: Infer executable material readiness from the presence of a BOM and delivery event. |
| HPT-008 | Manufacturing process is named but cell capability is not qualified | A process plan references a named manufacturing process.; The actual machine/tool/workcell capability evidence does not cover a required tolerance, material, size or process condition. Mutation: Treat a process label or generic machine family as proof that the exact operation is executable. |
| HPT-009 | Robot tool-center-point calibration is stale after tool exchange | A stationary manipulator and tool were previously qualified for a bounded task.; The tool was exchanged or repaired and the current calibration evidence is expired or bound to an older tool configuration. Mutation: Reuse the previous capability receipt for the new tool/configuration generation. |
| HPT-010 | Robot reports task success but independent metrology is incomplete | An execution record reports task completion.; The required independent measurement coverage is incomplete or its uncertainty is outside the acceptance decision limit. Mutation: Promote the controller or robot self-report to final product acceptance. |
| HPT-011 | Installed component identity does not match the released article | The released package references one exact component/article and revision.; The installed observation identifies a different article, variant or revision. Mutation: Join the installed record to the released design by family name or nominal geometry rather than exact identity. |
| HPT-012 | Material substitution approval applies only to a superseded design revision | A material substitution was approved for one exact design revision.; The current work package uses a later design revision with a changed requirement or interface. Mutation: Reuse the old substitution approval for the new design without an applicability witness. |
| HPT-013 | Inspection instrument calibration is outside its valid interval | A metrology observation is available.; The instrument calibration validity window does not cover the observation time or measurement context. Mutation: Treat the measurement as current because the report file is present and its numeric value is plausible. |
| HPT-014 | Repair changes a connection but old commissioning evidence is reused | An installed panel has passed a bounded commissioning campaign.; A repair replaces or changes a connection, fixing or service interface. Mutation: Retain the previous as-built configuration identity and reuse all commissioning evidence without change-impact assessment. |
| HPT-015 | Material certification is revoked before installation decision | A material certificate was valid at manufacturing time.; A qualified revocation record is available before the installation decision frontier. Mutation: Use the historical certificate without evaluating revocation state at the decision time. |
| HPT-016 | Approval references correct component but signer lacks scoped authority | An approval or review receipt references the exact panel and revision.; The purported signer lacks the required role, scope, delegation or current authority evidence. Mutation: Treat a valid signature or authenticated identity as sufficient professional or statutory approval. |
| HPT-017 | Manufacturer source data has unknown rights or permitted use | A source CAD, drawing or machine file is available.; Its rights or permitted use status is unknown. Mutation: Redistribute or use the source payload in a public benchmark bundle solely because its digest is known. |
| HPT-018 | Two sensors share a common calibration or power dependency | Two measurements are presented as independent corroboration of a critical condition.; The sources share a calibration chain, sensor controller, clock or power dependency. Mutation: Count both observations as independent evidence without recording their shared dependency. |
| HPT-019 | Site reference frame changes after panel layout approval | The panel layout and robot task are bound to an exact site/workcell coordinate frame.; A later survey changes the accepted reference frame or its uncertainty envelope. Mutation: Continue using the previous layout and robot pose evidence without transforming and requalifying the affected relationships. |
| HPT-020 | Unknown fixture state or safety envelope before robot task | A robot work package is otherwise complete.; The exact fixture state, guarded zone, local safety condition or localization validity is Unknown. Mutation: Allow the robot task to proceed because the planner produced a collision-free trajectory. |
| HPT-021 | As-built model is generated from design instead of observed installation | A design model and installation package exist.; No independent installation survey or direct observation is present. Mutation: Copy the design geometry into an as-built record and represent it as measured reality. |
| HPT-022 | Rollback to prior design with changed calibration and new incident history | An exact prior design generation is restored.; Relevant tooling calibration, site conditions or incident history changed after the original evidence was gathered. Mutation: Assume restoring the old design digest automatically restores all previous evidence applicability. |
| HPT-023 | Defect and near-miss history is deleted during repair | A defect or near-miss observation exists for the installed panel or workcell.; A repair or corrective action is proposed. Mutation: Replace the prior event history with a clean post-repair status and omit the original negative evidence. |
| HPT-024 | Panel disassembly attempted without required temporary support or qualified tool | A panel disassembly or replacement procedure is requested.; The required temporary support, access, tool identity or removal-force limit is missing or invalid. Mutation: Treat geometric access or nominal tool compatibility as sufficient evidence that removal is safe and feasible. |

## Reviewer A — independent first pass

Reviewer ID/role: ____________________  
Relevant competence and scope: ____________________  
Conflict-of-interest declaration: ____________________  
Review timestamp (UTC): ____________________  
Review packet digest (SHA-256): ____________________

| Case | Disposition | Reason codes and concise rationale | Missing proof obligations / decisive evidence | Requalification domains | Confidence / ambiguity |
|---|---|---|---|---|---|
| HPT-001 |  |  |  |  |  |
| HPT-002 |  |  |  |  |  |
| HPT-003 |  |  |  |  |  |
| HPT-004 |  |  |  |  |  |
| HPT-005 |  |  |  |  |  |
| HPT-006 |  |  |  |  |  |
| HPT-007 |  |  |  |  |  |
| HPT-008 |  |  |  |  |  |
| HPT-009 |  |  |  |  |  |
| HPT-010 |  |  |  |  |  |
| HPT-011 |  |  |  |  |  |
| HPT-012 |  |  |  |  |  |
| HPT-013 |  |  |  |  |  |
| HPT-014 |  |  |  |  |  |
| HPT-015 |  |  |  |  |  |
| HPT-016 |  |  |  |  |  |
| HPT-017 |  |  |  |  |  |
| HPT-018 |  |  |  |  |  |
| HPT-019 |  |  |  |  |  |
| HPT-020 |  |  |  |  |  |
| HPT-021 |  |  |  |  |  |
| HPT-022 |  |  |  |  |  |
| HPT-023 |  |  |  |  |  |
| HPT-024 |  |  |  |  |  |

## Reviewer B — independent first pass

Reviewer ID/role: ____________________  
Relevant competence and scope: ____________________  
Conflict-of-interest declaration: ____________________  
Review timestamp (UTC): ____________________  
Review packet digest (SHA-256): ____________________

| Case | Disposition | Reason codes and concise rationale | Missing proof obligations / decisive evidence | Requalification domains | Confidence / ambiguity |
|---|---|---|---|---|---|
| HPT-001 |  |  |  |  |  |
| HPT-002 |  |  |  |  |  |
| HPT-003 |  |  |  |  |  |
| HPT-004 |  |  |  |  |  |
| HPT-005 |  |  |  |  |  |
| HPT-006 |  |  |  |  |  |
| HPT-007 |  |  |  |  |  |
| HPT-008 |  |  |  |  |  |
| HPT-009 |  |  |  |  |  |
| HPT-010 |  |  |  |  |  |
| HPT-011 |  |  |  |  |  |
| HPT-012 |  |  |  |  |  |
| HPT-013 |  |  |  |  |  |
| HPT-014 |  |  |  |  |  |
| HPT-015 |  |  |  |  |  |
| HPT-016 |  |  |  |  |  |
| HPT-017 |  |  |  |  |  |
| HPT-018 |  |  |  |  |  |
| HPT-019 |  |  |  |  |  |
| HPT-020 |  |  |  |  |  |
| HPT-021 |  |  |  |  |  |
| HPT-022 |  |  |  |  |  |
| HPT-023 |  |  |  |  |  |
| HPT-024 |  |  |  |  |  |

## Third reviewer — disagreement resolution

Third reviewer ID/role: ____________________  
Relevant competence and scope: ____________________  
Conflict-of-interest declaration: ____________________  
Resolution timestamp (UTC): ____________________  
Review packet digest (SHA-256): ____________________

| Case | A vs B | Resolution / final label | Evidence rationale | Dissent preserved? | Returned to author for rewrite? |
|---|---|---|---|---|---|
| HPT-001 |  |  |  |  |  |
| HPT-002 |  |  |  |  |  |
| HPT-003 |  |  |  |  |  |
| HPT-004 |  |  |  |  |  |
| HPT-005 |  |  |  |  |  |
| HPT-006 |  |  |  |  |  |
| HPT-007 |  |  |  |  |  |
| HPT-008 |  |  |  |  |  |
| HPT-009 |  |  |  |  |  |
| HPT-010 |  |  |  |  |  |
| HPT-011 |  |  |  |  |  |
| HPT-012 |  |  |  |  |  |
| HPT-013 |  |  |  |  |  |
| HPT-014 |  |  |  |  |  |
| HPT-015 |  |  |  |  |  |
| HPT-016 |  |  |  |  |  |
| HPT-017 |  |  |  |  |  |
| HPT-018 |  |  |  |  |  |
| HPT-019 |  |  |  |  |  |
| HPT-020 |  |  |  |  |  |
| HPT-021 |  |  |  |  |  |
| HPT-022 |  |  |  |  |  |
| HPT-023 |  |  |  |  |  |
| HPT-024 |  |  |  |  |  |

## Freeze and release checklist

- [ ] Each reviewer used the redacted packet and completed a separate first-pass record.
- [ ] Competence and conflict declarations are recorded.
- [ ] The exact review-packet SHA-256 is recorded.
- [ ] The original source-corpus Git blob and version are recorded.
- [ ] All 24 cases have two independent first-pass judgments or an explicit non-adjudicable rationale.
- [ ] Every disagreement has a third-reviewer disposition or remains unresolved.
- [ ] Negative evidence, dissent and changes to provisional labels are preserved.
- [ ] Final labels are bound to the exact packet/corpus digests and protocol version.
- [ ] The evaluator has a trusted verification path for the signed adjudication manifest (not merely a caller-supplied status string).
- [ ] A separate hidden holdout and access-control arrangement exists before any blind benchmark or SOTA claim.

## Required final manifest fields

`protocol_id`, `protocol_version`, `review_packet_sha256`, `source_corpus_git_blob_sha`, `source_corpus_sha256`, `reviewer_a_attestation`, `reviewer_b_attestation`, `third_reviewer_attestation`, `individual_labels`, `disagreement_log`, `final_labels`, `created_at`, `signatures`.

This worksheet is an uncompleted review template. It does not declare that any case has been reviewed, adjudicated, certified or qualified.
