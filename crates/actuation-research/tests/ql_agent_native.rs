//! Real QL CLI through the production Prime faculty; no scripted owner/model.
use actuation_research::{faculty, process::ProcessSpec};
use serde_json::{json, Value};

#[test]
#[ignore = "requires QL_AGENT_TEST_BIN, QL_AGENT_OWNER_REVISION and QL_AGENT_EVENT_FIXTURE from the actual QL build"]
fn prime_faculty_uses_the_actual_event_and_harmonic_owner_without_a_model() {
    let ql = std::env::var("QL_AGENT_TEST_BIN").expect("actual native QL binary");
    let revision = std::env::var("QL_AGENT_OWNER_REVISION").expect("source basis revision");
    let fixture = std::env::var("QL_AGENT_EVENT_FIXTURE").expect("actual QL event fixture");
    let mut event: Value = serde_json::from_slice(&std::fs::read(fixture).unwrap()).unwrap();
    event["observed"] = json!([
        {"field":"lens","value":"L2'","origin":"observed","basis_refs":[event["material"]["ref"]]},
        {"field":"local-position","value":3,"origin":"observed","basis_refs":[event["material"]["ref"]]},
        {"field":"coordinate-face","value":"direct","origin":"observed","basis_refs":[event["material"]["ref"]]},
        {"field":"musical-basis","value":"chromatic","origin":"observed","basis_refs":[event["material"]["ref"]]}
    ]);
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("faculty.json");
    std::fs::write(&config, json!({"schema":"actuation.prime-faculty/v1",
        "owner":{"native_cli":true,"revision":revision,"process":{"program":ql,"args":[],"cwd":dir.path(),"environment":{},"timeout_ms":5000,"output_limit":8*1024*1024}},
        "harmonic_enabled":true
    }).to_string()).unwrap();
    let request = json!({"event":event,"requested_heads":["lens"]});
    let invoke = |operation: &str, request: Value| {
        faculty::invoke(
            &config,
            &json!({"operation":operation,"request":request}),
            None,
            Some("prime:native-ql-test"),
        )
        .unwrap()
    };
    let projected = invoke("ql-project-event", request.clone());
    assert_eq!(projected["success"], true);
    assert_eq!(projected["result"]["schema"], "ql.agent-projection/v1");
    assert_eq!(projected["result"]["decision_head_ids"], json!([]));
    assert!(projected["result"]["determination"]
        .get("provider")
        .is_none());
    let determined = invoke("ql-decide", request.clone());
    assert_eq!(
        determined["result"], projected["result"],
        "zero heads bypass even absent decision configuration"
    );
    assert_eq!(
        invoke("ql-decision-frame", request.clone())["result"],
        projected["result"]["frame"]
    );
    assert_eq!(
        invoke("ql-harmonic-read", request.clone())["result"],
        projected["result"]["harmonic"]
    );
    let mut invalid = request.clone();
    invalid["event"]["observed"][0]["value"] = json!("L99");
    assert_eq!(invoke("ql-project-event", invalid)["success"], false);
    let topology = faculty::invoke(&config, &json!({"operation":"tda-vietoris-rips", "request":{
        "metric":"precomputed","complex":"vietoris-rips","coefficients":2,"max_homology_dimension":1,"max_scale":2,
        "distances":[[0,1,1],[1,0,1],[1,1,0]],"source_basis":{"source_ref":"ql:test:measured-triangle"}
    }}), None, Some("prime:native-ql-test")).unwrap();
    assert_eq!(topology["success"], true);
    let result = &topology["result"]["result"];
    assert_eq!(result["schema"], "ql.tda-persistence/v1");
    let h0: Vec<_> = result["intervals"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["dimension"] == 0)
        .collect();
    assert_eq!(h0.len(), 3);
    assert_eq!(h0.iter().filter(|row| row["death"].is_null()).count(), 1);
    assert_eq!(h0.iter().filter(|row| row["death"] == 1.0).count(), 2);
    // Controlled faulty transport input exercises the real native verifier.
    // The external projected result is deliberately fabricated; it must never
    // become operative. This is refusal evidence, not model accuracy evidence.
    let mut unresolved = request.clone();
    unresolved["event"]["observed"] = json!([]);
    let current = invoke("ql-project-event", unresolved.clone());
    let frame = &current["result"]["frame"];
    let response = json!({"schema":"ql.agent-decision-response/v1",
        "event_basis_digest":frame["event_basis_digest"],
        "frame_digest":current["result"]["determination"]["frame_digest"],
        "kernel_basis":frame["kernel_basis"],"outcome":"answered",
        "provider":{"provider_ref":"test:controlled-fault-input","model_ref":"test:fault-input",
          "model_revision":"test:controlled-fault-input","runtime_revision":"test:fault-input"},
        "proposals":[{"head_id":"semantic-lens","label_ids":["L99"],"confidence":1.0,"spans":[]}]});
    let faulty = dir.path().join("faulty-response.json");
    std::fs::write(&faulty, json!({"schema":"ql.agent-decision-admission/v1",
        "response":response,"projection":{"determination":{"status":"determined","fabricated":true}}}).to_string()).unwrap();
    let mut configured: Value = serde_json::from_slice(&std::fs::read(&config).unwrap()).unwrap();
    configured["decision"] = json!({"program":"/bin/cat","args":[faulty],"cwd":dir.path(),
        "environment":{},"timeout_ms":5000,"output_limit":1024*1024});
    std::fs::write(&config, configured.to_string()).unwrap();
    let refused = invoke("ql-decide", unresolved);
    assert_eq!(refused["success"], true);
    assert!(!refused["result"]["determination"]["refused_candidates"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(
        refused["result"]["determination"]["refused_candidates"][0]["reason"],
        "proposal violates native candidate field/cardinality"
    );
    assert_ne!(refused["result"]["determination"]["status"], "determined");
    assert!(refused["result"]["determination"]
        .get("fabricated")
        .is_none());
    println!(
        "{}",
        json!({"schema":"actuation.ql-native-faculty-proof/v1","checks":7,"model_calls":0,"kernel_basis":projected["result"]["frame"]["kernel_basis"]})
    );
}

#[cfg(unix)]
#[test]
#[ignore = "requires QL_AGENT_ADAPTER_ROOT and QL_AGENT_PYTHON from the installed shared adapter"]
fn prime_decision_timeout_reaps_the_real_separately_grouped_native_client() {
    let root = std::env::var("QL_AGENT_ADAPTER_ROOT").unwrap();
    let python = std::env::var("QL_AGENT_PYTHON").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("inner.pid");
    let nested = format!(
        "import os,time; open({:?},'w').write(str(os.getpid())); time.sleep(60)",
        marker
    );
    let outer = format!("import sys; sys.path.insert(0,{:?}); import ql_agent_aikit as a; a.run_owned([sys.executable,'-c',{:?}],60)",root,nested);
    let process = ProcessSpec {
        program: python.into(),
        args: vec!["-c".into(), outer],
        cwd: dir.path().to_owned(),
        environment: Default::default(),
        timeout_ms: 1000,
        output_limit: 1024 * 1024,
    };
    let error = process
        .run_with_termination_grace(b"", std::time::Duration::from_millis(2500))
        .unwrap_err();
    assert!(error.to_string().contains("timed out"));
    let pid = std::fs::read_to_string(marker).expect("actual inner native client launched");
    let alive = std::process::Command::new("/bin/kill")
        .args(["-0", pid.trim()])
        .status()
        .unwrap();
    assert!(
        !alive.success(),
        "timed-out nested client must be reaped before the faculty returns"
    );
}
