//! Controlled provider tests exercise the production HTTP client. They are not
//! observations of Jev reasoning and do not require a provider credential.
use actuation_adapters::jev::*;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

fn request() -> SystemOneRequest {
    serde_json::from_value(json!({
        "state": {"message":"The delivery is arriving at the wrong address", "attempts":2},
        "model":"jev-1.13.0",
        "questions":{
            "urgent":{"type":"noul","instructions":{"question":"Does the message require attention?"}},
            "route":{"type":"choice","instructions":"Select the responsible service", "criteria":{"delivery":null,"billing":{"scope":"payment"}}},
            "severity":{"type":"score","instructions":"Rate severity", "criteria":["low","high"]}
        }
    })).unwrap()
}
fn response() -> Value {
    json!({"model":"jev-1.13.0","answers":{
        "urgent":{"type":"noul","noul":0.9},
        "route":{"type":"choice","choice":"delivery","probabilities":{"delivery":0.8,"billing":0.2},"confidence":0.6},
        "severity":{"type":"score","score":0.75,"legend":{"0":"low","1":"high"},"probabilities":{"0":0.25,"1":0.75},"confidence":0.5}
    },"usage":{"input_tokens":300,"output_tokens":40}})
}
fn attribution() -> InvocationAttribution {
    serde_json::from_value(json!({"request_ref":"request:general-routing","agent_ref":"agent:worker",
        "agency_ref":"agency:admitted-worker","agent_session_ref":"agent-session:worker",
        "external_refs":["now:bounded-child","commission:delivery"]})).unwrap()
}
fn limits() -> JevLimits {
    JevLimits { deadline_ms: 2_000, attempt_timeout_ms: 500, max_attempts: 2,
        retry_backoff_ms: 10, max_spend_microusd: 10_000,
        max_request_bytes: 65_536, max_response_bytes: 65_536,
        price: JevPrice { model:"jev-1.13.0".into(), input_microusd_per_million_tokens:42_000,
            source_ref:"https://docs.typesafe.ai/models#jev-1.13".into() } }
}
fn parse_response(value: Value) -> Result<SystemOneResponse, String> {
    let value: SystemOneResponse = serde_json::from_value(value).map_err(|e| e.to_string())?;
    request().validate_response(value).map(|d| d.into_response()).map_err(|e| e.to_string())
}

#[test]
fn full_general_typed_protocol_is_validated_without_document_modes() {
    request().validate().unwrap();
    let returned = parse_response(response()).unwrap();
    assert_eq!(returned.model, "jev-1.13.0");
    assert_eq!(returned.usage.input_tokens, 300);
    assert_eq!(returned.answers.len(), 3);
}

#[test]
fn missing_foreign_and_type_mismatched_answers_refuse() {
    let mut missing = response(); missing["answers"].as_object_mut().unwrap().remove("urgent");
    assert!(parse_response(missing).is_err());
    let mut extra = response(); extra["answers"]["unasked"] = json!({"type":"noul","noul":0.5});
    assert!(parse_response(extra).is_err());
    let mut wrong = response(); wrong["answers"]["urgent"] = response()["answers"]["route"].clone();
    assert!(parse_response(wrong).is_err());
}

#[test]
fn missing_model_usage_and_alias_as_actual_version_refuse() {
    for field in ["model", "usage"] {
        let mut value = response(); value.as_object_mut().unwrap().remove(field);
        assert!(parse_response(value).is_err());
    }
    for model in ["", "jev-latest", "jev-preview"] {
        let mut value = response(); value["model"] = json!(model);
        assert!(parse_response(value).is_err());
    }
    let mut value = response(); value["usage"]["input_tokens"] = json!(-1);
    assert!(parse_response(value).is_err());
}

#[test]
fn distributions_cannot_be_partial_normalized_or_have_a_false_argmax() {
    let mut value = response(); value["answers"]["route"]["probabilities"] = json!({"delivery":1.0});
    assert!(parse_response(value).is_err());
    let mut value = response(); value["answers"]["route"]["probabilities"] = json!({"delivery":0.8,"billing":0.8});
    assert!(parse_response(value).is_err());
    let mut value = response(); value["answers"]["route"]["choice"] = json!("billing");
    assert!(parse_response(value).is_err());
    let mut value = response(); value["answers"]["route"]["probabilities"] = json!({"delivery":0.8,"billing":0.2,"foreign":0.0});
    assert!(parse_response(value).is_err());
}

#[test]
fn score_and_rubric_and_finite_probability_must_agree() {
    let mut value = response(); value["answers"]["severity"]["score"] = json!(0.2);
    assert!(parse_response(value).is_err());
    let mut value = response(); value["answers"]["severity"]["legend"]["0"] = json!("other");
    assert!(parse_response(value).is_err());
    let mut value = response(); value["answers"]["urgent"]["noul"] = json!(1.1);
    assert!(parse_response(value).is_err());
    let mut value: SystemOneResponse = serde_json::from_value(response()).unwrap();
    value.answers.insert("urgent".into(), Answer::Noul { noul:f64::NAN });
    assert!(request().validate_response(value).is_err());
}

#[test]
fn structured_question_entries_and_zero_reported_usage_are_not_lost() {
    let mut req = request();
    req.questions.insert("structured".into(), serde_json::from_value(json!({
        "type":"score","instructions":null,"criteria":[{"outcome":"none"},["both","together"]]
    })).unwrap());
    req.validate().unwrap();
    let mut value = response(); value["answers"]["structured"] = json!({"type":"score","score":0.8,
        "legend":{"0":{"outcome":"none"},"1":["both","together"]},
        "probabilities":{"0":0.2,"1":0.8},"confidence":0.5});
    value["usage"] = json!({"input_tokens":0,"output_tokens":0});
    let result: SystemOneResponse = serde_json::from_value(value).unwrap(); let result=req.validate_response(result).unwrap();
    assert_eq!(result.response().usage.input_tokens,0);
}

#[test]
fn malformed_questions_refuse_before_transport() {
    let mut req = request(); req.state = json!(true); assert!(req.validate().is_err());
    let mut req = request(); req.questions.clear(); assert!(req.validate().is_err());
    let mut req = request(); req.questions.insert("bad".into(), Question::Score { instructions:json!("rate"), criteria:vec![json!("one")] });
    assert!(req.validate().is_err());
    let mut req = request(); req.questions.insert("bad".into(), Question::Noul { instructions:json!("yes?"), criteria:Some([("maybe".into(),Value::Null)].into()) });
    assert!(req.validate().is_err());
}

fn read_request(stream: &mut TcpStream) -> Value {
    stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let mut bytes = Vec::new(); let mut buffer = [0u8; 4096];
    let end = loop {
        let n = stream.read(&mut buffer).unwrap(); assert!(n > 0); bytes.extend_from_slice(&buffer[..n]);
        if let Some(offset) = bytes.windows(4).position(|s| s == b"\r\n\r\n") { break offset + 4; }
        assert!(bytes.len() < 65_536);
    };
    let headers = std::str::from_utf8(&bytes[..end]).unwrap().to_lowercase();
    assert!(headers.starts_with("post /v1/systemone http/1.1"));
    assert!(headers.contains("authorization: bearer controlled-only-secret"));
    let len: usize = headers.lines().find_map(|line| line.strip_prefix("content-length:")).unwrap().trim().parse().unwrap();
    while bytes.len() - end < len { let n=stream.read(&mut buffer).unwrap(); assert!(n>0); bytes.extend_from_slice(&buffer[..n]); }
    serde_json::from_slice(&bytes[end..end+len]).unwrap()
}
fn provider(replies: Vec<(u16, String, Duration)>) -> (JevClient, thread::JoinHandle<Vec<Value>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let endpoint = format!("http://{}/v1/systemone", listener.local_addr().unwrap());
    let worker = thread::spawn(move || {
        let mut requests = Vec::new();
        for (status, body, pause) in replies {
            let start = Instant::now();
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream,_)) => break stream,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && start.elapsed() < Duration::from_secs(3) => thread::sleep(Duration::from_millis(5)),
                    Err(e) => panic!("controlled provider did not receive expected request: {e}"),
                }
            };
            requests.push(read_request(&mut stream));
            thread::sleep(pause);
            let _ = write!(stream,"HTTP/1.1 {status} Controlled\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body);
        }
        requests
    });
    (JevClient::controlled_loopback(&endpoint,JevCredential::resolved("controlled-only-secret".into()).unwrap()).unwrap(), worker)
}

#[test]
fn actual_http_transport_delivers_the_questions_and_returns_attributable_usage() {
    let (client, server) = provider(vec![(200,response().to_string(),Duration::ZERO)]);
    let result = client.evaluate(&request(),attribution(),limits(),&JevCancellation::default()).unwrap();
    assert_eq!(result.receipt.standing,JevStanding::Completed);
    assert!(result.response.is_some());
    assert_eq!(result.receipt.attempts[0].actual_model.as_deref(),Some("jev-1.13.0"));
    assert_eq!(result.receipt.total_estimated_cost_microusd,Some(13));
    let receipt = serde_json::to_string(&result.receipt).unwrap();
    assert!(!receipt.contains("controlled-only-secret"));
    assert!(!receipt.contains("wrong address"));
    assert_eq!(server.join().unwrap()[0],serde_json::to_value(request()).unwrap());
}

#[test]
fn incomplete_http_answer_is_not_a_successful_determination() {
    let mut omitted=response();omitted["answers"].as_object_mut().unwrap().remove("urgent");
    let (client,server)=provider(vec![(200,omitted.to_string(),Duration::ZERO)]);
    let result=client.evaluate(&request(),attribution(),limits(),&JevCancellation::default()).unwrap();
    assert_eq!(result.receipt.standing,JevStanding::Failed);
    assert!(result.response.is_none());
    assert_eq!(result.receipt.failure.unwrap().code,"jev.answer.invalid");
    assert_eq!(result.receipt.attempts[0].usage.as_ref().unwrap().input_tokens,300);
    server.join().unwrap();
}

#[test]
fn rate_limit_retry_is_finite_and_unknown_usage_is_not_zero_cost() {
    let (client,server)=provider(vec![(429,"{}".into(),Duration::ZERO),(200,response().to_string(),Duration::ZERO)]);
    let result=client.evaluate(&request(),attribution(),limits(),&JevCancellation::default()).unwrap();
    assert_eq!(result.receipt.standing,JevStanding::Completed);
    assert_eq!(result.receipt.attempts.len(),2);
    assert!(result.receipt.total_estimated_cost_microusd.is_none());
    assert_eq!(result.receipt.total_reserved_cost_microusd,5376);
    assert_eq!(server.join().unwrap().len(),2);
}

#[test]
fn budget_exhaustion_does_not_send_another_request() {
    let (client,server)=provider(vec![(429,"{}".into(),Duration::ZERO)]);
    let mut bound=limits();bound.max_spend_microusd=bound.reservation().unwrap();
    let result=client.evaluate(&request(),attribution(),bound,&JevCancellation::default()).unwrap();
    assert_eq!(result.receipt.standing,JevStanding::BudgetExhausted);
    assert_eq!(result.receipt.attempts.len(),1);
    assert!(result.response.is_none());server.join().unwrap();
}

#[test]
fn malformed_and_oversized_http_responses_cannot_supply_answers() {
    for body in ["not JSON".to_owned(),"x".repeat(500)] {
        let (client,server)=provider(vec![(200,body,Duration::ZERO)]);
        let mut bound=limits();bound.max_response_bytes=100;
        let result=client.evaluate(&request(),attribution(),bound,&JevCancellation::default()).unwrap();
        assert_ne!(result.receipt.standing,JevStanding::Completed);
        assert!(result.response.is_none());server.join().unwrap();
    }
}

#[test]
fn cancelled_in_flight_response_is_discarded_and_no_retry_follows() {
    let (client,server)=provider(vec![(200,response().to_string(),Duration::from_millis(90))]);
    let token=JevCancellation::default();let cancelling=token.clone();
    let cancel=thread::spawn(move || {thread::sleep(Duration::from_millis(30));cancelling.cancel();});
    let result=client.evaluate(&request(),attribution(),limits(),&token).unwrap();
    assert_eq!(result.receipt.standing,JevStanding::Cancelled);
    assert!(result.response.is_none());assert_eq!(result.receipt.attempts.len(),1);
    cancel.join().unwrap();server.join().unwrap();
}

#[test]
fn timeout_and_changed_model_preserve_failure_without_retry_or_answer() {
    let (client,server)=provider(vec![(200,response().to_string(),Duration::from_millis(200))]);
    let mut bound=limits();bound.attempt_timeout_ms=30;
    let result=client.evaluate(&request(),attribution(),bound,&JevCancellation::default()).unwrap();
    assert!(result.response.is_none());assert_eq!(result.receipt.attempts.len(),1);
    assert!(result.receipt.attempts[0].effect_uncertain);server.join().unwrap();
    let mut changed=response();changed["model"]=json!("jev-2.0.0");
    let (client,server)=provider(vec![(200,changed.to_string(),Duration::ZERO)]);
    let result=client.evaluate(&request(),attribution(),limits(),&JevCancellation::default()).unwrap();
    assert!(result.response.is_none());assert_eq!(result.receipt.failure.unwrap().code,"jev.answer.invalid");
    assert!(result.receipt.total_estimated_cost_microusd.is_none());server.join().unwrap();
}

#[test]
fn credentials_endpoints_and_admission_bounds_cannot_expand_implicitly() {
    assert!(JevCredential::resolved("secret\nheader: injected".into()).is_err());
    for endpoint in ["http://localhost:99/v1/systemone","https://attacker.example/v1/systemone", "http://key@127.0.0.1:99/v1/systemone", "http://127.0.0.1:99/other", "http://127.0.0.1:99/v1/systemone?key=value"] {
        assert!(JevClient::controlled_loopback(endpoint,JevCredential::resolved("controlled-only-secret".into()).unwrap()).is_err());
    }
    let mut bound=limits();bound.max_attempts=0;assert!(bound.validate(&request()).is_err());
    let mut bound=limits();bound.max_spend_microusd=1;assert!(bound.validate(&request()).is_err());
    let mut req=request();req.model="jev-latest".into();assert!(limits().validate(&req).is_err());
}
