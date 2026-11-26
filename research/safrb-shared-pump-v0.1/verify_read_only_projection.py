#!/usr/bin/env python3
"""Independent test-only verifier for the SAFRB Sol-Atlas projection fixture."""
import copy, json, sys
from pathlib import Path

SINGLE="0a1c507d3bff9b3062e24ca8718e102244473d839c2a31c5ead31812ee249be2"
TEMPORAL="edfd1c2a5a3eb52a0026a0e63abb38ab2ca8109c4de3d96fbff1215ad78f0308"

def require(condition, message):
    if not condition:
        raise AssertionError(message)

def verify(doc):
    require(doc["projection_id"]=="SOL-ATLAS-SAFRB-SHARED-PUMP-001","projection identity")
    require(doc["maturity"]=="synthetic_test_fixture","maturity promotion")
    require(doc["coordinate_frame"]=="schematic_only_no_geographic_coordinates","geographic coordinate claim introduced")
    require(doc["read_only"] is True,"projection acquired operational write authority")
    scenarios={s["case_id"]:s["scenario_sha256"] for s in doc["source_scenarios"]}
    require(scenarios.get("SAFRB-SHARED-PUMP-2ROUTE-001")==SINGLE,"single-window source digest")
    require(scenarios.get("SAFRB-TWO-WINDOW-SHARED-PUMP-001")==TEMPORAL,"two-window source digest")
    physical=doc["layers"]["physical_logistics"]
    nodes=physical["nodes"]; by_id={n["node_id"]:n for n in nodes}
    require(len(by_id)==len(nodes),"duplicate physical/logical node identity")
    pumps=[n for n in nodes if n["kind"]=="physical_resource"]
    require(len(pumps)==1 and pumps[0]["resource_id"]=="physical:pump-01","projection must show exactly one physical pump")
    routes={r["route_id"]:r for r in doc["layers"]["route_references"]}
    require(set(routes)=={"route-a","route-b"},"two logical routes missing")
    for route in routes.values():
        require(route["uses_resource_ids"]==["physical:pump-01"],"logical route no longer references the same pump")
    for edge in physical["edges"]:
        require(edge["from"] in by_id and edge["to"] in by_id,"dangling projection edge")
    pump_edges=[e for e in physical["edges"] if e.get("resource_id")=="physical:pump-01"]
    require(len(pump_edges)==2 and {e["to"] for e in pump_edges}=={"route-a","route-b"},"shared pump projection edges changed")
    evidence=doc["layers"]["evidence"]
    by_evidence={e["evidence_id"]:e for e in evidence}
    require(len(by_evidence)==len(evidence),"duplicate evidence identity")
    for item in evidence:
        require(item["verdict"]=="indeterminate_not_guaranteed","evidence verdict promoted by projection")
        require(item["claim_scope"]=="FUEL-DELIVERABILITY","evidence claim scope promoted by projection")
        require(item["unit"]=="t","evidence unit mismatch")
    require(by_evidence["shared-pump-certificate"]["lower_bound_max_flow"]==8,"single-window lower flow")
    require(by_evidence["shared-pump-certificate"]["lower_bound_min_cut"]==8,"single-window cut")
    require(by_evidence["shared-pump-certificate"]["verdict"]=="indeterminate_not_guaranteed","single-window verdict promotion")
    temporal=by_evidence["two-window-shared-pump-certificate"]
    require(temporal["scenario_sha256"]==TEMPORAL,"two-window certificate digest")
    require(temporal["lower_bound_max_flow"]==temporal["lower_bound_min_cut"]==9,"time-expanded lower certificate")
    require(temporal["upper_bound_max_flow"]==temporal["upper_bound_min_cut"]==12,"time-expanded upper certificate")
    require(temporal["shortfall"]==3 and temporal["verdict"]=="indeterminate_not_guaranteed","time-expanded verdict/shortfall")
    windows=doc["layers"]["time_windows"]
    require(len(windows)==2 and [w["window"] for w in windows]==["[0,1)","[1,2)"],"time windows merged or missing")
    require(windows[0]["pump_capacity_lower"]==6 and windows[0]["lower_bound_delivery"]==6,"window 1 capacity/delivery")
    require(windows[1]["pump_capacity_lower"]==3 and windows[1]["lower_bound_delivery"]==3 and windows[1]["upper_bound_delivery"]==6,"window 2 capacity/delivery")
    require(windows[1]["disposition"]=="indeterminate_not_guaranteed","window 2 scope promotion")
    dependencies=doc["layers"]["dependencies"]
    require(any(d["dependency_id"]=="failure-domain:pump-01" and set(d["shared_by"])=={"route-a","route-b"} for d in dependencies),"shared failure-domain dependency lost")
    require(all(x in doc["visual_invariants"] for x in [
      "render-one-physical-pump-node-only","both-route-references-point-to-same-pump-id",
      "display-service-by-window-not-only-horizon-total","never-infer-geographic-coordinates",
      "never-upgrade-physical-delivery-or-service-from-visualization"
    ]),"required visual invariant missing")
    require(set(["dispatch","reserve","inventory-mutation","equipment-actuation","legal-or-authority-decision"]).issubset(set(doc["prohibited_effects"])),"prohibited effect omitted")
    return True

def main():
    path=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).with_name("read-only-projection-known-answer.json")
    doc=json.loads(path.read_text(encoding="utf-8"))
    verify(doc)
    attacks=[]
    a=copy.deepcopy(doc); a["layers"]["physical_logistics"]["nodes"].append(copy.deepcopy(a["layers"]["physical_logistics"]["nodes"][2])); attacks.append(a)
    a=copy.deepcopy(doc); a["layers"]["route_references"][1]["uses_resource_ids"]=["physical:pump-02"]; attacks.append(a)
    a=copy.deepcopy(doc); a["read_only"]=False; attacks.append(a)
    a=copy.deepcopy(doc); a["coordinate_frame"]="EPSG:4326"; attacks.append(a)
    a=copy.deepcopy(doc); a["layers"]["time_windows"][1]["lower_bound_delivery"]=6; attacks.append(a)
    a=copy.deepcopy(doc); a["layers"]["time_windows"].pop(); attacks.append(a)
    a=copy.deepcopy(doc); a["layers"]["evidence"][1]["verdict"]="guaranteed"; attacks.append(a)
    a=copy.deepcopy(doc); a["prohibited_effects"].remove("dispatch"); attacks.append(a)
    rejected=0
    for attack in attacks:
        try: verify(attack)
        except (AssertionError,KeyError,TypeError): rejected+=1
    require(rejected==len(attacks),"a projection/claim-boundary mutation escaped verifier")
    print("PASS: two source digests; one physical pump; two logical routes; two separate service windows; read-only boundary; 8 hostile mutations rejected")

if __name__=="__main__":
    main()
