#!/usr/bin/env node
// Independent validation-only CTER authority consumer oracle; no npm dependencies.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
const dir=path.dirname(fileURLToPath(import.meta.url));
const corpus=JSON.parse(fs.readFileSync(path.join(dir,"..","authority-cases-v0.1.json"),"utf8"));
const classes=new Set(["public","community","restricted","culturally_protected"]);
const domains=new Set(["participant_consent","site_access","community_mandate","publication_consent","community_disclosure","archive_export"]);
const utc=s=>typeof s==="string"&&/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/.test(s);
const nonblank=(o,k)=>typeof o?.[k]==="string"&&o[k].trim()?o[k]:null;
function setOf(o,k){const a=o?.[k];if(!Array.isArray(a)||!a.length)return null;const s=new Set(a);if(s.size!==a.length||a.some(x=>typeof x!=="string"||!x.trim()))return null;return s;}
const sensitive=s=>s.has("community")||s.has("culturally_protected");
function requiredDomains(op,isSensitive){const d=new Set();if(op==="collect_measurement"){d.add("participant_consent");d.add("site_access");if(isSensitive)d.add("community_mandate");}else if(op==="publish_observation"){d.add("publication_consent");if(isSensitive)d.add("community_disclosure");}else if(op==="export_evidence"){d.add("archive_export");if(isSensitive)d.add("community_disclosure");}else return null;return d;}
function scopeMatches(r,q){for(const k of ["principal_ref","subject_ref","operation","resource_ref","project_snapshot_sha256","purpose_ref","recipient_ref","jurisdiction_profile_ref"])if(r?.[k]!==q?.[k])return false;const a=setOf(r,"authorized_data_classes"),b=setOf(q,"data_classes");return !!a&&!!b&&a.size===b.size&&[...a].every(x=>b.has(x));}
function evaluate(input){
 let severity=0;const reasons=new Set();
 const raise=(level,code)=>{severity=Math.max(severity,level);reasons.add(code);};
 const q=input?.request;
 if(!q)return {disposition:"incomplete",reason_codes:["request_missing"]};
 for(const k of ["request_id","principal_ref","subject_ref","operation","resource_ref","project_snapshot_sha256","purpose_ref","recipient_ref","jurisdiction_profile_ref"])if(!nonblank(q,k))raise(1,"request_"+k+"_missing");
 const at=nonblank(q,"evaluated_at");if(!at)raise(1,"request_evaluated_at_missing");else if(!utc(at))raise(1,"request_evaluated_at_invalid");
 if(!/^[0-9a-f]{64}$/.test(q.project_snapshot_sha256||""))raise(1,"request_project_snapshot_sha256_invalid");
 const data=setOf(q,"data_classes");if(!data)raise(1,"request_data_classes_missing_or_invalid");else if([...data].some(x=>!classes.has(x)))raise(1,"request_data_class_unknown");
 if(typeof q.offline_mode!=="boolean")raise(1,"request_offline_mode_missing");
 if(q.operation==="assert_legal_title"){raise(4,"legal_effect_outside_cter");return finish();}
 const required=data?requiredDomains(q.operation,sensitive(data)):null;
 if(!required){raise(4,"operation_outside_cter_authority_boundary");return finish();}
 const decisions=input?.decisions;if(!Array.isArray(decisions)){raise(2,"upstream_decision_bundle_unavailable");return finish();}
 const byDomain=new Map();const receiptContents=new Map();
 for(const r of decisions){
  const ref=nonblank(r,"receipt_ref");if(!ref){raise(1,"decision_receipt_malformed");continue;}
  const content=JSON.stringify(r);
  if(receiptContents.has(ref)){if(receiptContents.get(ref)!==content)raise(4,"authorization_receipt_identity_conflict");continue;}
  receiptContents.set(ref,content);
  if(r.receipt_type!=="AuthorizationReceipt")raise(4,"receipt_type_laundering");
  if(!scopeMatches(r,q))raise(4,"decision_scope_mismatch");
  if(!nonblank(r,"policy_ref")||!nonblank(r,"status_snapshot_ref")||!setOf(r,"input_receipt_refs"))raise(1,"decision_lineage_incomplete");
  if(!nonblank(r,"decision_maker_ref")||!nonblank(r,"decision_authority_ref"))raise(1,"decision_authority_lineage_missing");
  const domain=nonblank(r,"decision_domain");if(!domain){raise(1,"decision_receipt_malformed");continue;}
  if(!domains.has(domain))raise(1,"decision_domain_unknown");
  switch(r.disposition){case "allow":break;case "deny":raise(4,"upstream_decision_denied");break;case "review":raise(3,"upstream_decision_requires_review");break;case "unknown":raise(2,"upstream_decision_unknown");break;default:raise(1,"decision_disposition_invalid");}
  switch(r.signature_status){case "verified":break;case "invalid":raise(4,"receipt_signature_invalid");break;case "unavailable":raise(2,"receipt_signature_unavailable");break;default:raise(1,"receipt_signature_status_missing");}
  switch(r.authority_status){case "resolved":break;case "revoked":raise(4,"issuer_authority_revoked");break;case "unresolved":raise(3,"issuer_authority_unresolved");break;default:raise(1,"issuer_authority_status_missing");}
  switch(r.revocation_status){case "clear":break;case "revoked":raise(4,"decision_revoked");break;case "unknown":raise(2,"revocation_state_unknown");break;default:raise(1,"revocation_status_missing");}
  switch(r.currentness_status){case "current":break;case "stale":raise(2,"decision_currentness_stale");break;case "unknown":raise(2,"decision_currentness_unknown");break;default:raise(1,"currentness_status_missing");}
  if(utc(r.valid_from)&&utc(r.valid_until)&&r.valid_from<r.valid_until){if(at<r.valid_from)raise(4,"decision_not_yet_valid");else if(at>=r.valid_until)raise(4,"decision_expired");}else raise(1,"decision_validity_window_invalid");
  if(utc(r.offline_lease_expires_at)){if(q.offline_mode===true&&at>=r.offline_lease_expires_at)raise(2,"offline_authority_lease_expired");}else if(!(r.offline_lease_expires_at===null&&q.offline_mode===false))raise(1,"offline_lease_reference_invalid");
  const list=byDomain.get(domain)||[];list.push(ref);byDomain.set(domain,list);
 }
 for(const domain of required){const refs=byDomain.get(domain);if(!refs)raise(2,"required_decision_missing:"+domain);else if(new Set(refs).size>1)raise(3,"multiple_decisions_for_domain:"+domain);}
 for(const domain of byDomain.keys())if(!required.has(domain))raise(3,"unexpected_decision_domain:"+domain);
 if(required.has("community_mandate")||required.has("community_disclosure")){
  const g=input.governance_context;
  if(!g)raise(2,"community_governance_context_missing");
  else{
   for(const k of ["governance_profile_ref","decision_procedure_ref","representation_assessment_ref","appeal_route_ref"])if(!nonblank(g,k))raise(1,"governance_"+k+"_missing");
   switch(g.representation_status){case "resolved":break;case "contested":raise(3,"community_representation_contested");break;case "unknown":raise(2,"community_representation_unknown");break;default:raise(1,"community_representation_status_invalid");}
   switch(g.coverage_status){case "complete_within_declared_scope":break;case "known_gaps":raise(3,"community_representation_has_gaps");break;case "unknown":raise(2,"community_coverage_unknown");break;default:raise(1,"community_coverage_status_invalid");}
   switch(g.dissent_status){case "none":break;case "recorded":if(!nonblank(g,"dissent_register_ref"))raise(1,"recorded_dissent_reference_missing");break;case "material_unresolved":raise(3,"material_dissent_unresolved");break;case "unknown":raise(2,"dissent_state_unknown");break;default:raise(1,"dissent_status_invalid");}
   switch(g.challenge_status){case "none":case "resolved":break;case "open":raise(3,"material_challenge_open");break;case "unknown":raise(2,"challenge_state_unknown");break;default:raise(1,"challenge_status_invalid");}
   if(q.offline_mode===true){if(utc(g.governance_offline_lease_expires_at)&&at<g.governance_offline_lease_expires_at){}else if(utc(g.governance_offline_lease_expires_at))raise(2,"governance_offline_lease_expired");else raise(2,"governance_offline_lease_missing_or_invalid");}
  }
 }
 return finish();
 function finish(){return {disposition:["eligible","incomplete","unknown","review","deny"][severity]||"incomplete",reason_codes:[...reasons].sort()};}
}
const failures=[];
for(const c of corpus.cases){const result=evaluate(c.input);if(result.disposition!==c.expected)failures.push({case_id:c.case_id,expected:c.expected,actual:result});}
const report={suite_version:corpus.suite_version,cases:corpus.cases.length,matched:corpus.cases.length-failures.length,result:failures.length?"FAIL":"PASS",failures};
console.log(JSON.stringify(report,null,2));if(failures.length)process.exitCode=1;
