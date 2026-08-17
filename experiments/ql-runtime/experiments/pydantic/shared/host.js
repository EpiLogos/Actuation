export const PYDANTIC_UPSTREAM = Object.freeze({
  repository: 'pydantic/pydantic-ai',
  revision: '00db3a4b391eb9a46f3d6e704070bcf725121f75',
  inspected_surfaces: ['pydantic_ai_slim/pydantic_ai/run.py', 'pydantic_ai_slim/pydantic_ai/_agent_graph.py']
});

const clone = (value) => value === undefined ? undefined : structuredClone(value);

export class DeterministicPydanticPort {
  constructor() {
    this.nativeRunId = 'pydantic-fixture-run';
    this.listeners = new Set();
    this.modelIndex = 0;
    this.state = { message_history: [], node_history: ['UserPromptNode'], next_node: 'ModelRequestNode' };
    this.responses = [
      { content: 'Inspect the matched subject before completing.', capabilityCalls: [{ id: 'pydantic-call-1', name: 'inspect', args: { subject: 'matched-runtime-baseline' } }] },
      { content: 'Matched subject verified; baseline outcome is ready.', capabilityCalls: [] }
    ];
  }

  subscribe(listener) {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  notify(event) {
    for (const listener of this.listeners) listener(clone(event));
  }

  advance(node, nextNode, payload = {}) {
    this.notify({ type: 'graph_node_start', node, state: clone(this.state) });
    this.state.node_history.push(node);
    this.state.next_node = nextNode;
    this.notify({ type: 'graph_node_end', node, next_node: nextNode, payload: clone(payload), state: clone(this.state) });
  }

  async modelTurn() {
    const response = clone(this.responses[Math.min(this.modelIndex, this.responses.length - 1)]);
    this.modelIndex += 1;
    this.state.message_history.push({ role: 'assistant', content: response.content });
    this.advance('ModelRequestNode', 'CallToolsNode', { response });
    return response;
  }

  async capability(name, args) {
    if (name !== 'inspect') throw new Error(`Unknown Pydantic capability '${name}'.`);
    const result = { ok: true, subject: args?.subject ?? 'matched-runtime-baseline', observation: 'subject verified', source: 'pydantic-tool' };
    this.advance('CallToolsNode', 'ModelRequestNode', { name, args: clone(args), result: clone(result) });
    return result;
  }

  async externalInput() { return null; }
  async context(kind, input) { return { kind, input: clone(input), graph: clone(this.state), native_run_id: this.nativeRunId }; }
}

export class PydanticRuntimeHost {
  id = 'pydantic-ai';
  revision = PYDANTIC_UPSTREAM.revision;

  constructor(port = new DeterministicPydanticPort()) {
    this.port = port;
    this.observer = null;
    this.runId = null;
    this.sequence = 0;
  }

  attachObserver(observer, runId) {
    this.observer = observer;
    this.runId = runId;
    this.port.subscribe((event) => this.emitHost('pydantic_graph_event', event));
  }

  emitHost(eventType, payload) {
    this.observer?.emit({ channel: 'host', event_id: `${this.runId}:pydantic-host:${this.sequence}`, event_type: eventType, run_id: this.runId, sequence: this.sequence++, host: this.id, payload: clone(payload) });
  }

  async callModel() {
    this.emitHost('model_requested', { native_run_id: this.port.nativeRunId });
    const result = await this.port.modelTurn();
    this.emitHost('model_returned', { content: result.content, capability_call_count: result.capabilityCalls.length });
    return result;
  }

  async executeCapability({ name, args } = {}) {
    this.emitHost('capability_requested', { name, args: clone(args ?? {}) });
    const result = await this.port.capability(name, args ?? {});
    this.emitHost('capability_returned', { name, result: clone(result) });
    return result;
  }

  async receiveExternalInput() {
    const result = await this.port.externalInput();
    this.emitHost('external_input_polled', { available: result !== null && result !== undefined });
    return result;
  }

  async readContext({ kind, input } = {}) {
    const result = await this.port.context(kind, input);
    this.emitHost('context_read', { kind, result: clone(result) });
    return result;
  }
}

export function createPydanticHost() { return new PydanticRuntimeHost(new DeterministicPydanticPort()); }
