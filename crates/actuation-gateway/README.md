# actuation-gateway

The first-party **Agency Gateway** (EpiLogos/O-I #154): one persistent encounter
plane over relations Actuation already owns — `Agency`, `AgentSession`,
`ActuationStream` — so the same situated Agency can be encountered through
terminal/CLI surfaces today and messaging platforms through the same connector
seam, with attributable Return and explicit co-internal invocation.

The gateway invents no new event ontology. Every write lands as a portable
`actuation.stream/v1` event in the caller's own durable
[`JsonlStreamStore`](../../docs/ACTUATION-STREAM.md) (one append-only JSONL file
per stream, exact contiguous cursor, identity consistency refused on drift).
Canonical identity is supplied by the attacher and checked against the stream's
durable header; the gateway never mints or rewrites an AgentSession.

## The vertical

```text
CLI / connector surface            resident agent-locus session
        │ send (challenge)              │ post tool evidence + return
        ▼                               ▼
   actuation-gateway  ── grants ── ActuationStream (JSONL store)
        ▲                               ▲
        └── invoke (delegation) ────────┘  attributable Return correlated
```

- **in**: `send` from a connector subject becomes an attributed `human-message`
  (granted `participant_ref` + `surface_ref`; a provider conversation id stays
  opaque in `metadata.conversation`, never collapsed into session identity).
- **out**: the resident session posts `tool-request` / `tool-result` / `return`
  events attributed exactly to its granted locus, with real resource and
  evidence refs.
- **co-internal invocation**: a controller Agency `invoke`s another Agency's
  live session. Allowed modes (`communique`, `session-contribution`,
  `delegation`) are explicit policy; the delegation lands in the target's
  stream attributed to the controller while the stream keeps its governing
  Agency; the Return is correlated by an explicit `return_ref` in both
  directions. A refused invocation is retained as an attributed `refusal`
  event where the attempt happened — it never touches the target stream and
  fabricates no Return.

## Wire protocol

Transport-independent frames: one JSON object per line, request/reply. `hello`
negotiates `actuation.gateway/v1` (a mismatch is refused by name) and
authenticates. Ops: `status`, `attach`, `send`, `post`, `replay`, `wait`,
`invoke`, `discover`, `ping`. Every mutating op replies with a receipt naming
the durable event, cursor and lifecycle. Refusals are
`{"ok":false,"denied":true,...}` with a reason, never silence.

## Authority is explicit

A connection can do only what its policy grant says (`actuation.gateway-policy/v1`):

```json
{
  "schema": "actuation.gateway-policy/v1",
  "attach": [
    {"subject": "connector:cli", "role": "connector", "stream_ref": "stream:worker",
     "surface_ref": "surface:cli", "participant_ref": "participant:alice"},
    {"subject": "agent:worker-1", "role": "agent", "stream_ref": "stream:worker",
     "agency_ref": "agency:worker", "agent_ref": "agent:worker", "locus_ref": "locus:worker"},
    {"subject": "agent:ctl-1", "role": "agent", "stream_ref": "stream:ctl",
     "agency_ref": "agency:ctl", "agent_ref": "agent:ctl", "may_invoke": true}
  ],
  "invoke": [
    {"controller_agency_ref": "agency:ctl", "target_agency_ref": "agency:worker",
     "modes": ["delegation", "communique", "session-contribution"]}
  ]
}
```

Presence does not imply authority: what is not granted is refused. Co-internal
invocation additionally requires a **live** granted resident session for the
target Agency on this gateway — the connection table is presence, not identity.

## Transport and authentication seam

The frame protocol is transport-independent. The first carrier is a same-host
Unix domain socket. Authentication is a shared bearer token (`--token`,
`--token-env`, or `ACTUATION_GATEWAY_TOKEN` on both sides), checked on `hello`
— the same arrangement as the Workcell control plane's
`WORKCELL_CONTROL_TOKEN`.

The network carrier named by O-I #154 is authenticated **WebSocket** on a
private fabric. This vertical does not open any network listener: until the
WebSocket carrier lands, remote reachability is provided by Workcell Fabric
(e.g. a private Tailscale path) fronting this same-host service, or by an SSH
tunnel. **The gateway must never be exposed plaintext on a public network.**
Transport admission (fabric), gateway protocol identity (this token), and
Actuation/action authority (the grants and the store) remain distinct layers.

## Connector seam

`SurfaceConnector` is the seam every platform connector implements:
`capabilities` (declared, not assumed), `provenance`, `health`, `admit`
(platform-native inbound → canonical stream), `await_return`. The fully
functional `LocalConnector` proves it; Telegram/Discord/Slack implement the
same trait against their native bodies with their capability differences kept
explicit.

## Executable

```text
actuation-gateway serve      --socket P --store D --policy F [--token T | --token-env NAME]
actuation-gateway connector  --socket P --subject S --stream R --actuation R --agency R \
                             --session R --message TEXT [--conversation C] [--token T]
actuation-gateway agent      --socket P --subject S --stream R --actuation R --agency R \
                             --session R [--agent-ref R] [--locus-ref R] [--token T] [--once]
actuation-gateway invoke     --socket P --subject S --stream R --actuation R --agency R \
                             --session R --agent-ref R --target-agency R --payload TEXT \
                             [--mode delegation] [--invocation-ref R] [--return-ref R]
actuation-gateway version
```

All subcommands print JSON receipts on stdout. `serve` runs in the foreground
until its supervisor stops it; a stale socket file from a dead process is
removed before binding.

## Hosting on Workcell (material note)

The gateway is an ordinary persistent service in Workcell's sense: Workcell
hosts the material conditions; it does not own the agency semantics inside.
Declare it in the state root's `services.json`
(`workcell.service-declaration/v1`, per Workcell's
`docs/CAW-MATERIAL-OPERATIONS.md` vocabulary):

```json
{
  "schema": "workcell.service-declaration/v1",
  "services": [
    {
      "logical_ref": "agent-gateway",
      "lifetime": "provider-process-scoped",
      "command": {
        "executable": "/usr/local/bin/actuation-gateway",
        "args": ["serve", "--socket", "/var/lib/actuation-gateway/gateway.sock",
                 "--store", "/var/lib/actuation-gateway/streams",
                 "--policy", "/var/lib/actuation-gateway/policy.json",
                 "--token-env", "ACTUATION_GATEWAY_TOKEN"],
        "cwd": "/var/lib/actuation-gateway"
      },
      "health": {"readiness": {"kind": "socket-present", "path": "/var/lib/actuation-gateway/gateway.sock"}}
    }
  ]
}
```

Bind the durable stream store through the ordinary storage requirement —
writable, shared, `persistence: "external"`, `retention: "preserve"` — so the
canonical streams survive restart, rematerialisation and relocation. The
logical binding is `interactive-stream`, transport `uds` (local) with scope
`private-network`; a later WebSocket binding is a new `NetworkRelationship`,
not a new gateway identity. Restart/rebind continuity holds because canonical
identity lives in the streams, not in the process or the socket path.

## Tests

`cargo test -p actuation-gateway` covers frame/protocol admission, policy
exactness, stream mapping and attribution (unit), and the full vertical in
process: challenge in → attributable durable events → Return out; delegation
with correlated Return; refused invocation (ungranted pair, and granted pair
with no live resident session) retained as attributed refusal; replay after
reattach folding the same durable stream.
