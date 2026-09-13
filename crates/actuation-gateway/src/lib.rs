//! First-party Agency Gateway for the EpiLogos O:I suite.
//!
//! One persistent encounter plane over the same canonical relations the rest
//! of Actuation already owns: `Agency` / `AgentSession` / `ActuationStream`.
//! The gateway consumes `actuation-stream`'s portable contract and durable
//! JSONL store; it promotes no provider trace into a second event ontology.
//!
//! ```text
//! Surface / connector ─┐                       ┌─ agent-locus connection
//!                      ├─ actuation.gateway/v1 ─┤   (post / invoke / wait)
//! co-internal invoke ──┘        over UDS        └─ canonical Stream events
//! ```
//!
//! Authority is explicit: nothing is attachable or invocable without a
//! policy grant, and every event is attributed exactly as granted.

mod client;
mod connector;
mod mapping;
mod policy;
mod server;
mod wire;

pub use client::{AttachSpec, GatewayClient};
pub use connector::{
    last_sequence, ConnectorCapabilities, Health, InboundEvent, LocalConnector, SurfaceConnector,
};
pub use mapping::{
    agent_event, delegation_event, find_return, human_message, invocation_refusal, mint_event_ref,
    mint_return_ref, InvokeRequest, PostRequest, SendRequest, POSTABLE_KINDS,
};
pub use policy::{AttachGrant, GatewayPolicy, GrantRole, InvocationMode, InvokeGrant};
pub use server::{
    start, Binding, Gateway, GatewayConfig, GatewayHandle,
};
pub use wire::{
    decode_frame, encode_frame, error_frame, denied_frame, ok_frame, read_frame, write_frame,
    HelloFrame, Reply, GATEWAY_CONTRACT, GATEWAY_IDENTITY, POLICY_SCHEMA,
};
