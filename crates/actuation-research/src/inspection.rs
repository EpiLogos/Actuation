//! Read-only projection of actually retained native events into DSH's inspection
//! ABI. It is never the candidate's context or evidence of QL closure by itself.
use actuation_runtime::LoopEvent;
use serde_json::{json, Value};
pub const SCHEMA: &str = "ql-series1-dsh-inspection/0.1";
pub fn project(events: &[LoopEvent], run_id: &str, condition: &str) -> Value {
    let mut positions = vec![];
    let mut relations = vec![];
    let mut closure = vec![];
    let mut reentry = vec![];
    let mut operators = vec![];
    let mut difference = vec![];
    let mut portable = vec![];
    for (index, event) in events.iter().enumerate() {
        // This index orders this retained projection, not another owner's store.
        let item = json!({"portable_record_index":index,"portable_channel":event.channel,
            "portable_event_type":event.event_type,"event":event});
        portable.push(item);
        if event.channel != "runtime-semantic" {
            continue;
        }
        if !event.payload["active_position"].is_null() {
            positions.push(event.payload["active_position"].clone());
        }
        let research = &event.payload["research"];
        let row = json!({"record_index":index,"event_id":event.event_id,"event_type":event.event_type,"payload":event.payload});
        if let Some(relation) = research["relation"].as_str() {
            if relation.len() == 3
                && relation.starts_with('R')
                && relation.as_bytes()[1..].iter().all(u8::is_ascii_digit)
            {
                relations.push(json!({"record_index":index,"event_id":event.event_id,"event_type":event.event_type,"relation":relation}));
            }
        }
        let kind = event.event_type.to_lowercase();
        if kind.contains("closure") {
            closure.push(row.clone());
        }
        if ["reentry", "re-entry", "reopen"]
            .iter()
            .any(|s| kind.contains(s))
        {
            reentry.push(row.clone());
        }
        if ["conjugate", "child_", "depth", "square", "modulation"]
            .iter()
            .any(|s| kind.contains(s))
        {
            operators.push(row.clone());
        }
        let text = event.payload.to_string();
        if ["\"pi\"", "\"rho\"", "π", "ρ", "difference"]
            .iter()
            .any(|s| text.contains(s))
        {
            difference.push(row);
        }
    }
    let projection = json!({"schema":SCHEMA,"run_id":run_id,"condition":condition,"read_only":true,
        "candidate_context_authority":false,"model_calls":[],
        "index_standing":"order within this retained native projection; not a host store sequence",
        "ql":{"positions":positions,"relations":relations,"closure":closure,"reentry":reentry,
            "operators":operators,"pi_rho_difference":difference},"portable_events":portable});
    let seed=portable.iter().enumerate().map(|(seq,item)|json!({"type":"series1/portable-event","seq":seq,
        // DSH's seed time is a deterministic structural value, not wall-clock observation.
        "time":0,"ignorable":true,"data":{"schema":SCHEMA,"run_id":run_id,"condition":condition,
            "portable_record_index":item["portable_record_index"],"portable_channel":item["portable_channel"],
            "portable_event_type":item["portable_event_type"],"event":item["event"]}})).collect::<Vec<_>>();
    json!({"projection":projection,"seed":seed})
}

#[cfg(test)]
mod tests {
    use super::*;
    use actuation_core::ExternalRef;

    fn event(channel: &str, event_type: &str, payload: Value, sequence: u64) -> LoopEvent {
        LoopEvent {
            channel: channel.into(),
            event_id: format!("trace:run:{sequence}"),
            event_type: event_type.into(),
            run_id: ExternalRef::new("trace:run").unwrap(),
            sequence,
            runtime: "classic".into(),
            payload,
        }
    }

    #[test]
    fn projection_sorts_retained_events_into_their_sections() {
        let events = vec![
            event(
                "runtime-semantic",
                "model_closure",
                json!({"active_position":"P4","research":{"relation":"R31"}}),
                0,
            ),
            event(
                "runtime-semantic",
                "circuit_reopened",
                json!({"research":{"relation":"R12"},"note":"pi difference observed"}),
                1,
            ),
            event(
                "runtime-semantic",
                "child_started",
                json!({"research":{"relation":"not-R"}}),
                2,
            ),
            event("runtime", "capability_returned", json!({"ok":true}), 3),
        ];
        let out = project(&events, "trace:run", "classic");
        let p = &out["projection"];
        assert_eq!(p["schema"], json!(SCHEMA));
        assert_eq!(p["run_id"], json!("trace:run"));
        assert_eq!(p["condition"], json!("classic"));
        assert_eq!(p["read_only"], json!(true));
        assert_eq!(p["candidate_context_authority"], json!(false));
        assert_eq!(p["model_calls"], json!([]));
        assert_eq!(p["ql"]["positions"], json!(["P4"]));
        assert_eq!(p["ql"]["relations"].as_array().unwrap().len(), 2);
        assert_eq!(p["ql"]["closure"].as_array().unwrap().len(), 1);
        assert_eq!(p["ql"]["reentry"].as_array().unwrap().len(), 1);
        assert_eq!(p["ql"]["operators"].as_array().unwrap().len(), 1);
        assert_eq!(p["ql"]["pi_rho_difference"].as_array().unwrap().len(), 1);
        // Every event is retained portably, including the non-semantic one.
        assert_eq!(p["portable_events"].as_array().unwrap().len(), 4);
        let seed = out["seed"].as_array().unwrap();
        assert_eq!(seed.len(), 4);
        assert_eq!(seed[0]["seq"], json!(0));
        assert_eq!(seed[3]["seq"], json!(3));
        assert_eq!(seed[2]["data"]["portable_record_index"], json!(2));
        // The non-semantic event contributes no QL rows.
        assert!(p["ql"]["operators"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["record_index"].as_u64().unwrap() < 3));
    }

    #[test]
    fn projection_of_an_empty_run_stays_schema_true() {
        let out = project(&[], "trace:empty", "ql-deep");
        assert_eq!(out["projection"]["ql"]["positions"], json!([]));
        assert_eq!(out["seed"], json!([]));
        assert_eq!(out["projection"]["portable_events"], json!([]));
    }
}
