# Native experiment

Issue 103 baseline is active over the frozen runtime foundation.

The host contract is intentionally minimal: model, capability, exterior input, context and raw transport events. It does not copy Pi turn/session machinery or Pydantic graph machinery. Classic and ql-core use the same Native adapter and matched inputs.

QL circuit state, relations, P5 determination, closure and re-entry remain owned by the unchanged shared ql-core runtime. Deep QL operators remain out of scope until issue 109.
