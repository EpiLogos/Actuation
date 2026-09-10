use actuation_core::{Error, ExternalRef, Result};
use actuation_runtime::*;
use serde_json::{json, Value};
use std::{
    collections::VecDeque,
    future::Future,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};

pub fn block_on<F: Future>(future: F) -> F::Output {
    struct ThreadWake(std::thread::Thread);
    impl Wake for ThreadWake {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::park(),
        }
    }
}
#[derive(Default)]
pub struct Witness {
    pub events: Vec<LoopEvent>,
    pub refuse: bool,
}
impl RuntimeObserver for Witness {
    fn emit(&mut self, event: &LoopEvent) -> Result<ExternalRef> {
        if self.refuse {
            return Err(Error::new("observer refused evidence"));
        }
        self.events.push(event.clone());
        ExternalRef::new(format!("test-witness:{}", event.event_id))
    }
}
pub struct ScriptedHost {
    models: VecDeque<Value>,
    human: VecDeque<Value>,
    capabilities: Value,
    abort_after_model: bool,
    pub calls: Vec<Value>,
}
impl ScriptedHost {
    pub fn new(input: &Value) -> Self {
        Self {
            models: input["models"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into(),
            human: input["human"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into(),
            capabilities: input["capabilities"].clone(),
            abort_after_model: input["abort_after_model"] == true,
            calls: vec![],
        }
    }
    fn record(&mut self, method: &str, call: &HostCall) {
        let mut payload = call.payload.clone();
        payload.insert("request".into(), call.request.wire().clone());
        self.calls.push(json!({"method":method,"call":payload}));
    }
    fn reply(value: Value) -> std::result::Result<Value, HostError> {
        match value.get("throw").and_then(Value::as_str) {
            Some(message) if value["error_name"] == "AbortError" => {
                Err(HostError::Aborted(message.into()))
            }
            Some(message) => Err(HostError::Failed(message.into())),
            _ => Ok(value),
        }
    }
}
impl RuntimeHost for ScriptedHost {
    fn call_model<'a>(&'a mut self, call: HostCall) -> HostFuture<'a> {
        self.record("model", &call);
        if self.abort_after_model {
            call.cancellation.request();
        }
        let response = Self::reply(self.models.pop_front().unwrap_or(Value::Null));
        Box::pin(async { response })
    }
    fn execute_capability<'a>(&'a mut self, call: HostCall) -> HostFuture<'a> {
        self.record("capability", &call);
        let response =
            Self::reply(self.capabilities[call.payload["name"].as_str().unwrap_or("")].clone());
        Box::pin(async { response })
    }
    fn receive_external_input<'a>(&'a mut self, call: HostCall) -> HostFuture<'a> {
        self.record("human", &call);
        let response = Self::reply(self.human.pop_front().unwrap_or(Value::Null));
        Box::pin(async { response })
    }
    fn read_context<'a>(&'a mut self, call: HostCall) -> HostFuture<'a> {
        self.record("context", &call);
        Box::pin(async move { Ok(json!({"context":call.payload["kind"]})) })
    }
}
pub fn evaluate(row: &Value) -> Result<Value> {
    let input = &row["input"];
    match row["kind"].as_str() {
        Some("loop") => {
            let request = LoopRequest::from_legacy(input["request"].clone())?;
            let mut host = ScriptedHost::new(input);
            let mut witness = Witness::default();
            let cancellation = CancellationToken::default();
            if input["aborted"] == true {
                cancellation.request();
            }
            let execution =
                block_on(ClassicRuntime.run(&request, &mut host, &mut witness, &cancellation))?;
            Ok(json!({"result":execution.report,"events":witness.events,"calls":host.calls}))
        }
        Some("carrier") => {
            let request = LoopRequest::from_legacy(input["request"].clone())?;
            let mut host = ScriptedHost::new(
                &json!({"models":[{"content":"model"}],"human":["human"],"capabilities":{"read":{"ok":true},"write":{"ok":true}}}),
            );
            let carrier = Carrier::from_legacy(&input["carrier"]);
            let result = match carrier {
                Ok(carrier) => match block_on(dispatch_host_carrier(
                    &mut host,
                    &carrier,
                    &request,
                    &CancellationToken::default(),
                    input["payload"].as_object().unwrap().clone(),
                )) {
                    Ok(value) => json!({"ok":true,"value":value}),
                    Err(error) => json!({"ok":false,"error":error.message()}),
                },
                Err(error) => json!({"ok":false,"error":error.to_string()}),
            };
            Ok(json!({"result":result,"calls":host.calls}))
        }
        Some("registry") => {
            let mut registry = RuntimeRegistry::default();
            registry.register(Box::new(ClassicRuntime))?;
            assert!(registry.register(Box::new(ClassicRuntime)).is_err());
            assert!(registry.get_mut("missing").is_err());
            Ok(registry.list())
        }
        _ => Err(Error::new("unhandled runtime extraction case")),
    }
}
