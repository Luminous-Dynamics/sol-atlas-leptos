#!/usr/bin/env node
// Validation-only second-language oracle. No npm dependencies.
// The relay reference intentionally uses repeated-add traversal rather than
// the Rust predecessor queue, making this an independent algorithm check.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const dir = path.dirname(fileURLToPath(import.meta.url));
const read = (name) => JSON.parse(fs.readFileSync(path.join(dir, name), "utf8"));
const stable = (v) => Array.isArray(v) ? "[" + v.map(stable).join(",") + "]"
  : v && typeof v === "object" ? "{" + Object.keys(v).sort().map(k => JSON.stringify(k) + ":" + stable(v[k])).join(",") + "}"
  : JSON.stringify(v);
const digest = (v) => typeof v === "string" && /^[0-9a-f]{64}$/.test(v);

function parseDelivery(d) {
  const e=d?.event, a=d?.assessment, c=d?.context, p=d?.payload;
  if (!e || !a || !c || !p || !e.event_id || !e.stream_id || !Number.isSafeInteger(e.sequence)
      || !digest(e.event_hash) || !e.actor
      || !((e.sequence===0 && e.previous_hash===null) || (e.sequence>0 && digest(e.previous_hash)))
      || !p.blob_ref || !p.media_type || !digest(p.blob_sha256)
      || !a.assessment_id || !a.verifier_id || !a.policy_id || !a.assessed_at
      || !a.key_status_snapshot_ref
      || !["trusted_append_receipt","external_trusted_timestamp","signer_claim_only","unknown"].includes(a.event_time_basis)
      || !["verified","invalid","revoked","unavailable"].includes(a.status)
      || !c.projection_key
      || !["public","community","restricted","culturally_protected"].includes(c.access_class)) return null;
  const trusted = ["trusted_append_receipt","external_trusted_timestamp"].includes(a.event_time_basis) && !!a.time_evidence_ref;
  const status = ["invalid","unavailable"].includes(a.status) ? a.status : (trusted ? a.status : "unavailable");
  return {
    id:e.event_id, stream:e.stream_id, seq:e.sequence, prev:e.previous_hash,
    hash:e.event_hash, event:stable(e), payloadDigest:p.blob_sha256,
    assessment:[a.policy_id,status,a.key_status_snapshot_ref,a.event_time_basis].join("|"),
    context:[c.projection_key,c.access_class].join("|")
  };
}

function relayOracle(deliveries) {
  const r={accepted_events:[],duplicate_replays:0,invalid_signatures:[],revoked_keys:[],
    verification_pending:[],verification_conflict_event_ids:[],malformed_events:[],
    missing_parent_events:[],blocked_parent_events:[],equivocation_event_ids:[],
    sequence_fork_event_ids:[],sequence_conflict_pending_event_ids:[],hash_collision_event_ids:[],
    context_conflict_event_ids:[],payload_digest_conflict_event_ids:[],conflict_groups:[]};
  const variants=new Map(), hashIds=new Map();
  deliveries.forEach((d,i)=>{
    const hint=d?.event?.event_id || "#delivery-"+i, x=parseDelivery(d);
    if(!x){r.malformed_events.push(hint);return;}
    const hs=hashIds.get(x.hash)||new Set();hs.add(x.id);hashIds.set(x.hash,hs);
    const by=variants.get(x.id)||new Map(), old=by.get(x.hash);
    if(old){
      if(old.event!==x.event)old.equivocation=true;
      else {r.duplicate_replays++;old.assessments.add(x.assessment);old.contexts.add(x.context);old.payloadDigests.add(x.payloadDigest);}
    } else by.set(x.hash,{...x,assessments:new Set([x.assessment]),contexts:new Set([x.context]),payloadDigests:new Set([x.payloadDigest]),equivocation:false});
    variants.set(x.id,by);
  });
  const unique=new Map();
  for(const [id,by] of variants){
    if(by.size!==1){r.equivocation_event_ids.push(id);continue;}
    const x=[...by.values()][0];
    if(x.equivocation)r.equivocation_event_ids.push(id);else unique.set(id,x);
  }
  for(const ids of hashIds.values())if(ids.size>1)r.hash_collision_event_ids.push(...ids);
  const status=new Map();
  for(const [id,x] of unique){
    if(x.assessments.size!==1)r.verification_conflict_event_ids.push(id);
    else status.set(id,[...x.assessments][0].split("|")[1]);
    if(x.contexts.size>1)r.context_conflict_event_ids.push(id);
    if(x.payloadDigests.size>1)r.payload_digest_conflict_event_ids.push(id);
  }
  const slots=new Map();
  for(const [id,x] of unique){
    if(r.hash_collision_event_ids.includes(id)||status.get(id)==="invalid")continue;
    const k=x.stream+"#"+x.seq, a=slots.get(k)||[];a.push(id);slots.set(k,a);
  }
  for(const ids of slots.values())if(ids.length>1){
    if(ids.every(id=>status.get(id)==="verified"))r.sequence_fork_event_ids.push(...ids);
    else r.sequence_conflict_pending_event_ids.push(...ids);
  }
  const eligible=new Map();
  for(const [id,x] of unique){
    const s=status.get(id);
    if(s==="invalid")r.invalid_signatures.push(id);
    else if(s==="revoked")r.revoked_keys.push(id);
    else if(s==="unavailable")r.verification_pending.push(id);
    else if(s==="verified"&&!r.sequence_fork_event_ids.includes(id)
      &&!r.sequence_conflict_pending_event_ids.includes(id)&&!r.hash_collision_event_ids.includes(id)
      &&!r.payload_digest_conflict_event_ids.includes(id))eligible.set(id,x);
  }
  // Independent repeated-add evaluator; deliberately not a work queue.
  let changed=true;
  while(changed){
    changed=false;
    for(const [id,x] of eligible){
      if(r.accepted_events.includes(id))continue;
      if(x.prev===null){r.accepted_events.push(id);changed=true;continue;}
      const pids=hashIds.get(x.prev);
      if(!pids||pids.size!==1)continue;
      const pid=[...pids][0],p=eligible.get(pid);
      if(p&&p.stream===x.stream&&p.seq+1===x.seq&&r.accepted_events.includes(pid)){
        r.accepted_events.push(id);changed=true;
      }
    }
  }
  for(const [id,x] of eligible){
    if(r.accepted_events.includes(id))continue;
    const pids=x.prev===null?null:hashIds.get(x.prev);
    if(x.prev!==null&&!pids)r.missing_parent_events.push(id);else r.blocked_parent_events.push(id);
  }
  const projections=new Map();
  for(const id of r.accepted_events){
    const x=eligible.get(id);if(x.contexts.size!==1)continue;
    const key=[...x.contexts][0].split("|")[0], streams=projections.get(key)||new Map();
    const previous=streams.get(x.stream);
    if(!previous||x.seq>previous.sequence)streams.set(x.stream,{sequence:x.seq,event_id:id});
    projections.set(key,streams);
  }
  for(const [key,streams] of projections)if(streams.size>1){
    const members=[...streams.entries()].map(([stream,head])=>({event_id:head.event_id,stream_id:stream}))
      .sort((a,b)=>a.event_id.localeCompare(b.event_id)||a.stream_id.localeCompare(b.stream_id));
    r.conflict_groups.push({projection_key:key,members});
  }
  for(const k of ["accepted_events","invalid_signatures","revoked_keys","verification_pending",
    "verification_conflict_event_ids","malformed_events","missing_parent_events","blocked_parent_events",
    "equivocation_event_ids","sequence_fork_event_ids","sequence_conflict_pending_event_ids",
    "hash_collision_event_ids","context_conflict_event_ids","payload_digest_conflict_event_ids"])r[k].sort();
  r.conflict_groups.sort((a,b)=>a.projection_key.localeCompare(b.projection_key));
  return r;
}

function archiveOracle(m) {
  const classes=new Set(["public","community","restricted","culturally_protected"]), invalid="invalid";
  if(m?.schema!=="luminous.cter.community-archive.v0"
      ||m.claim_ceiling!=="archive-structure-and-restoration-only-no-legal-effect"
      ||!m.archive_id||m.profile_version!=="0.1")return invalid;
  const ec=m.export_context;
  if(!ec||!ec.destination_ref||!ec.authorization_ref||!ec.purpose_ref
      ||!Array.isArray(ec.authorized_access_classes)||!ec.authorized_access_classes.length
      ||new Set(ec.authorized_access_classes).size!==ec.authorized_access_classes.length
      ||ec.authorized_access_classes.some(x=>!classes.has(x)))return invalid;
  const authorized=new Set(ec.authorized_access_classes);
  if(!Array.isArray(m.objects)||!Array.isArray(m.source_frontiers))return invalid;
  if(!m.source_frontiers.length)return "incomplete";
  const refs=new Set(),counts={};let excluded=false,review=false,leak=false;
  for(const o of m.objects){
    if(!o.object_ref||refs.has(o.object_ref)||!o.owner_system||!classes.has(o.access_class)||!o.disposition)return invalid;
    refs.add(o.object_ref);counts[o.owner_system]=(counts[o.owner_system]||0)+1;
    if(o.disposition==="included"){
      if(!authorized.has(o.access_class))leak=true;
      if(!o.digest||!["SHA-256","BLAKE3"].includes(o.digest.algorithm)||!digest(o.digest.value)||!o.payload_ref)return invalid;
      if(["redistributable","reference_only"].includes(o.license_state)){}
      else if(["not_cleared","unknown",undefined].includes(o.license_state))review=true;
      else return invalid;
    }else if(["withheld","unavailable"].includes(o.disposition)){
      excluded=true;if(!o.reason_code)return invalid;if(Object.hasOwn(o,"payload_ref"))leak=true;
    }else return invalid;
  }
  const owners=new Set();let incomplete=false;
  for(const f of m.source_frontiers){
    if(!f.owner_system||owners.has(f.owner_system))return invalid;owners.add(f.owner_system);
    if(!f.snapshot_ref||!f.query_or_scope_ref||!f.snapshot_digest
       ||!["SHA-256","BLAKE3"].includes(f.snapshot_digest.algorithm)||!digest(f.snapshot_digest.value))return invalid;
    if(!Number.isSafeInteger(f.observed_count))return invalid;
    if(!Number.isSafeInteger(f.expected_count)||f.expected_count!==f.observed_count
       ||(counts[f.owner_system]||0)!==f.observed_count)incomplete=true;
    if(f.completeness!=="complete_within_declared_frontier"||!Array.isArray(f.gap_refs)||f.gap_refs.length!==0)incomplete=true;
  }
  if(Object.keys(counts).some(owner=>!owners.has(owner)))incomplete=true;
  const t=m.transport_observation;
  if(t?.carrier_bundle_state==="expired"&&t.durable_evidence_state!=="retained")incomplete=true;
  if(t?.durable_evidence_state==="retained"&&!t.retained_event_ref)return invalid;
  if(leak)return "disclosure_violation";if(incomplete)return "incomplete";
  const r=m.restore;
  if(!r||r.provider_independent_restore!==true
     ||!["community_threshold","participant_controlled","independent_trustee"].includes(r.key_custody)
     ||r.restore_test_status!=="pass"||!r.restore_instructions_ref||!r.restore_run_ref)return "not_independently_restorable";
  if(!Array.isArray(m.preservation))return invalid;if(!m.preservation.length)return "review_required";
  let renewed=false,historical=false;
  for(const p of m.preservation){
    if(p.validation_state==="failed"){historical=true;continue;}
    if(["compromised","deprecated","unknown"].includes(p.algorithm_state)){
      const timely=Number.isSafeInteger(p.renewal_time_unix_seconds)
        &&Number.isSafeInteger(p.algorithm_weakening_time_unix_seconds)
        &&p.renewal_time_unix_seconds<=p.algorithm_weakening_time_unix_seconds;
      if(p.validation_state==="valid"&&p.renewal_event_ref&&p.predecessor_ref&&p.timestamp_evidence_ref&&timely)renewed=true;
      else historical=true;
    }else if(p.validation_state!=="valid"||!p.timestamp_evidence_ref)historical=true;
    else if(p.renewal_event_ref)renewed=true;
  }
  if(historical)return "indeterminate_historical_validation";
  if(review)return "review_required";
  if(excluded)return "eligible_with_declared_exclusions";
  if(renewed)return "eligible_with_preservation_chain";
  return "eligible_for_independent_restore";
}

const relay=read("cases-v0.1.json"), relayManifest=read("manifest-v0.1.json");
const archive=read("archive-cases-v0.1.json"), failures=[];
if(relay.cases.length!==relayManifest.case_count)failures.push("relay manifest case_count mismatch");
for(const c of relay.cases)if(stable(relayOracle(c.deliveries))!==stable(c.expected))failures.push("relay "+c.case_id);
for(const c of archive.cases)if(archiveOracle(c.manifest)!==c.expected)failures.push("archive "+c.case_id);
const relayFailed=failures.filter(x=>x.startsWith("relay ")).length;
const archiveFailed=failures.filter(x=>x.startsWith("archive ")).length;
const result={relay_suite:relay.suite_version,relay_cases:relay.cases.length,relay_matched:relay.cases.length-relayFailed,
  archive_suite:archive.suite_version,archive_cases:archive.cases.length,archive_matched:archive.cases.length-archiveFailed,
  result:failures.length===0?"PASS":"FAIL",failures};
console.log(JSON.stringify(result,null,2));
if(failures.length)process.exitCode=1;
