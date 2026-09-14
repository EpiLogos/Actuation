//! `actuation-gateway` — the Workcell-hostable Agency Gateway executable.
//!
//! Subcommands print JSON receipts on stdout, one per line, so a session can
//! be driven and audited from scripts. The gateway is an ordinary persistent
//! service: Workcell may materialise and supervise it, but the gateway — not
//! Workcell — owns agency semantics (see docs/CONTROL-SERVICE-AND-AGENT-HOSTING
//! in the Workcell repository).

use actuation_core::Result;
use actuation_gateway::{
    AttachSpec, GatewayClient, GatewayConfig, GatewayPolicy, InboundEvent, LocalConnector,
    SurfaceConnector,
};
use actuation_stream::JsonlStreamStore;
use serde_json::{json, Value};
use std::{collections::HashMap, path::PathBuf, thread, time::Duration};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(args) {
        eprintln!("actuation-gateway: {error}");
        std::process::exit(1);
    }
}

fn usage() -> String {
    "usage: actuation-gateway <command>\n\
     \n\
     commands:\n  \
     serve      --socket P --store D --policy F [--token T | --token-env NAME]\n  \
     connector  --socket P --subject S --stream R --actuation R --agency R --session R\n  \
                --message TEXT [--conversation C] [--token T] [--wait-ms N]\n  \
     agent      --socket P --subject S --stream R --actuation R --agency R --session R\n  \
                [--agent-ref R] [--locus-ref R] [--token T] [--once]\n  \
     invoke     --socket P --subject S --stream R --actuation R --agency R --session R\n  \
                --agent-ref R --target-agency R [--mode delegation] --payload TEXT\n  \
                [--invocation-ref R] [--return-ref R] [--target-session R] [--timeout-ms N]\n  \
     version"
        .to_string()
}

struct Flags(HashMap<String, String>);
impl Flags {
    fn parse(args: &[String]) -> Result<Self> {
        let mut map = HashMap::new();
        let mut index = 0;
        while index < args.len() {
            let arg = &args[index];
            let Some(name) = arg.strip_prefix("--") else {
                return Err(actuation_core::Error::new(format!(
                    "unexpected argument {arg}; flags are --name value"
                )));
            };
            let value = args.get(index + 1).ok_or_else(|| {
                actuation_core::Error::new(format!("flag --{name} requires a value"))
            })?;
            map.insert(name.to_string(), value.clone());
            index += 2;
        }
        Ok(Self(map))
    }
    fn require(&self, name: &str) -> Result<String> {
        self.0
            .get(name)
            .cloned()
            .ok_or_else(|| actuation_core::Error::new(format!("missing required flag --{name}")))
    }
    fn opt(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }
    fn opt_u64(&self, name: &str) -> Result<Option<u64>> {
        self.opt(name)
            .map(|value| {
                value.parse::<u64>().map_err(|_| {
                    actuation_core::Error::new(format!("--{name} must be a non-negative integer"))
                })
            })
            .transpose()
    }
}

fn token_from(flags: &Flags) -> Result<Option<String>> {
    if let Some(token) = flags.opt("token") {
        return Ok(Some(token));
    }
    if let Some(name) = flags.opt("token-env") {
        return std::env::var(&name).map(Some).map_err(|_| {
            actuation_core::Error::new(format!("environment variable {name} is unset"))
        });
    }
    Ok(std::env::var("ACTUATION_GATEWAY_TOKEN").ok())
}

fn line(value: &Value) {
    println!("{value}");
}

fn connect(socket: &str, flags: &Flags, subject: &str) -> Result<GatewayClient> {
    GatewayClient::connect(
        PathBuf::from(socket).as_path(),
        token_from(flags)?.as_deref(),
        subject,
    )
}

fn attach_spec(flags: &Flags) -> Result<AttachSpec> {
    Ok(AttachSpec {
        stream_ref: actuation_core::StreamRef::new(flags.require("stream")?)?,
        actuation_ref: actuation_core::ActuationRef::new(flags.require("actuation")?)?,
        agency_ref: actuation_core::AgencyRef::new(flags.require("agency")?)?,
        agent_session_ref: actuation_core::AgentSessionRef::new(flags.require("session")?)?,
        world_binding_ref: flags
            .opt("world-binding")
            .map(actuation_core::WorldBindingRef::new)
            .transpose()?,
        provenance: None,
        started_at: None,
        conversation: flags.opt("conversation"),
    })
}

fn run(args: Vec<String>) -> Result<()> {
    let Some(command) = args.first().cloned() else {
        return Err(actuation_core::Error::new(usage()));
    };
    let flags = Flags::parse(&args[1..])?;
    match command.as_str() {
        "version" | "--version" | "-V" => {
            line(&json!({
                "gateway": "actuation-gateway",
                "version": env!("CARGO_PKG_VERSION"),
                "contract": actuation_gateway::GATEWAY_CONTRACT,
            }));
            Ok(())
        }
        "serve" => serve(&flags),
        "connector" => connector(&flags),
        "agent" => agent(&flags),
        "invoke" => invoke(&flags),
        other => Err(actuation_core::Error::new(format!(
            "unknown command {other:?}; {}",
            usage()
        ))),
    }
}

fn load_policy(path: &str) -> Result<GatewayPolicy> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| actuation_core::Error::new(format!("cannot read policy {path}: {e}")))?;
    let value: Value = serde_json::from_str(&raw)
        .map_err(|e| actuation_core::Error::new(format!("policy {path} is not valid JSON: {e}")))?;
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .ok_or_else(|| actuation_core::Error::new("policy requires a schema string"))?;
    GatewayPolicy::admitted(schema)?;
    serde_json::from_value(value)
        .map_err(|e| actuation_core::Error::new(format!("policy {path} is invalid: {e}")))
}

fn serve(flags: &Flags) -> Result<()> {
    let store = JsonlStreamStore::new(PathBuf::from(flags.require("store")?))?;
    let policy = load_policy(&flags.require("policy")?)?;
    let config = GatewayConfig {
        socket_path: PathBuf::from(flags.require("socket")?),
        token: token_from(flags)?,
        max_wait_ms: flags.opt_u64("max-wait-ms")?.unwrap_or(120_000),
    };
    let handle = actuation_gateway::start(config, store, policy)?;
    eprintln!(
        "{}",
        json!({
            "ready": true,
            "gateway": "actuation-gateway",
            "contract": actuation_gateway::GATEWAY_CONTRACT,
            "socket": flags.require("socket")?,
            "store": flags.require("store")?,
        })
    );
    // The service runs until its supervisor (or operator) stops the process.
    loop {
        if handle
            .gateway
            .shutdown
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            break;
        }
        thread::sleep(Duration::from_secs(1));
    }
    handle.stop();
    Ok(())
}

fn connector(flags: &Flags) -> Result<()> {
    let subject = flags.require("subject")?;
    let mut client = connect(&flags.require("socket")?, flags, &subject)?;
    let attach = client.attach(attach_spec(flags)?)?;
    line(&json!({"stage":"attach","reply":attach}));
    let mut connector = LocalConnector::new(client, flags.opt("conversation").unwrap_or_default());
    let message = flags.require("message")?;
    let receipt = connector.admit(InboundEvent {
        conversation: String::new(),
        text: message,
    })?;
    line(&json!({"stage":"send","receipt":receipt}));
    let after = receipt
        .get("event")
        .and_then(|event| event.get("sequence"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let wait_ms = flags.opt_u64("wait-ms")?.unwrap_or(10_000);
    let returned = connector.await_return(after, wait_ms)?;
    line(&json!({"stage":"return","event":returned}));
    Ok(())
}

fn agent(flags: &Flags) -> Result<()> {
    let subject = flags.require("subject")?;
    let once = flags.opt("once").is_some();
    let mut client = connect(&flags.require("socket")?, flags, &subject)?;
    let attach = client.attach(attach_spec(flags)?)?;
    line(&json!({"stage":"attach","reply":attach}));
    let mut cursor = attach
        .get("cursor")
        .and_then(|cursor| cursor.get("last_sequence"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    loop {
        let reply = client.wait(cursor, 15_000)?;
        cursor = reply
            .get("last_sequence")
            .and_then(Value::as_u64)
            .unwrap_or(cursor);
        let events = reply
            .get("events")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for event in events {
            let kind = event.get("kind").and_then(Value::as_str).unwrap_or("");
            let ingress = matches!(kind, "human-message" | "delegation" | "locus-event");
            if !ingress {
                continue;
            }
            let challenge = event
                .get("content")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let requested = client.post(json!({
                "kind":"tool-request",
                "content":challenge,
                "resource_refs":["tool:echo-agent"],
            }))?;
            line(&json!({"stage":"tool-request","receipt":requested}));
            let answer = format!("echo: {challenge}");
            let evidenced = client.post(json!({
                "kind":"tool-result",
                "content":answer,
                "evidence_refs":["tool:echo-agent"],
            }))?;
            line(&json!({"stage":"tool-result","receipt":evidenced}));
            let return_ref = event
                .pointer("/metadata/return_ref")
                .and_then(Value::as_str)
                .map(str::to_string)
                .map(Ok::<String, actuation_core::Error>)
                .unwrap_or_else(|| {
                    actuation_gateway::mint_return_ref().map(|reference| reference.into_string())
                })?;
            let returned = client.post(json!({
                "kind":"return",
                "content":answer,
                "return_ref":return_ref,
            }))?;
            line(&json!({"stage":"return","receipt":returned,"return_ref":return_ref}));
            cursor = returned
                .get("cursor")
                .and_then(|cursor| cursor.get("last_sequence"))
                .and_then(Value::as_u64)
                .unwrap_or(cursor);
            if once {
                return Ok(());
            }
        }
        if reply.get("timed_out") == Some(&Value::Bool(true)) && once {
            return Err(actuation_core::Error::new(
                "no ingress arrived within the wait window",
            ));
        }
    }
}

fn invoke(flags: &Flags) -> Result<()> {
    let subject = flags.require("subject")?;
    let mut client = connect(&flags.require("socket")?, flags, &subject)?;
    let attach = client.attach(attach_spec(flags)?)?;
    line(&json!({"stage":"attach","reply":attach}));
    let invocation_ref = flags
        .opt("invocation-ref")
        .unwrap_or_else(|| format!("invocation:{}", std::process::id()));
    let request = json!({
        "target_agency_ref": flags.require("target-agency")?,
        "target_agent_session_ref": flags.opt("target-session"),
        "mode": flags.opt("mode").unwrap_or_else(|| "delegation".into()),
        "invocation_ref": invocation_ref,
        "return_ref": flags.opt("return-ref"),
        "payload": flags.require("payload")?,
        "timeout_ms": flags.opt_u64("timeout-ms")?,
    });
    let reply = client.invoke_raw(request)?;
    line(&json!({"stage":"invoke","reply":reply}));
    if reply.get("ok") == Some(&Value::Bool(true))
        && reply.get("denied") != Some(&Value::Bool(true))
    {
        Ok(())
    } else {
        // A refused invocation is a printed receipt, not a crash.
        std::process::exit(3);
    }
}
