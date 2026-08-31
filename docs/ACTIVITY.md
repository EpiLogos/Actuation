# Semantic Activity

`actuation.activity/v1` is the portable semantic reading of acting that O:I and other consumers can use before drilling into raw runtime/provider traces.

The native relation is:

```text
Agency / Agent
  → realised Actuation
  → ActuationStream
  → Activity
  → optional Attention composition in the consuming whole
```

Activity does not replace `actuation.stream/v1`. It keeps the canonical stream as trace evidence and correlates semantic identities already owned elsewhere: AgentSession, World/Project/Run/Journey subject refs, Action/Invocation, Result/Evidence and Return.

Activity also does not imply that every meaningful event is an Action. `activityFromStreamEvent` can surface a world observation, material-host event, delegation or other meaningful stream event without manufacturing Action or Invocation identity.

## Boundary

```text
Activity != raw ACP / JSON-RPC / provider trace
Activity != Notification
Activity != Attention / Inbox
needs_attention != invocation or mutation authority
```

`needs_attention` is only a portable semantic signal. O:I composes it with the user's Attention/Inbox and notification policy. A normal live Activity remains globally visible even when `needs_attention` is false.

The `trace` member is the reversible seam back to the same `ActuationStream`: stream ref, event refs/sequences and native trace refs. This is what allows a Session Observatory to present a semantic reading first and then disclose raw evidence without maintaining a second activity log.
