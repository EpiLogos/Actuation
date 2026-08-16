const clone = (value) => value === undefined ? undefined : structuredClone(value);

/**
 * Minimal repo-owned transport. It intentionally introduces no turn, session,
 * graph, planner, critic, or subagent abstraction; the runtime owns recurrence.
 */
export class DeterministicNativePort {
  constructor() {
    this.revision = 'native-host-contract-v1';
    this.listeners = new Set();
    this.modelIndex = 0;
    this.responses = [
      { content: 'Inspect the matched subject before completing.', capabilityCalls: [{ id: 'native-call-1', name: 'inspect', args: { subject: 'matched-runtime-baseline' } }] },
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

  async model() {
    const index = Math.min(this.modelIndex, this.responses.length - 1);
    this.notify({ type: 'model_request', index });
    const result = clone(this.responses[index]);
    this.modelIndex += 1;
    this.notify({ type: 'model_response', index, result: clone(result) });
    return result;
  }

  async capability(name, args) {
    if (name !== 'inspect') throw new Error(`Unknown native capability '${name}'.`);
    this.notify({ type: 'capability_request', name, args: clone(args) });
    const result = { ok: true, subject: args?.subject ?? 'matched-runtime-baseline', observation: 'subject verified', source: 'native-capability' };
    this.notify({ type: 'capability_result', name, result: clone(result) });
    return result;
  }

  async external() { return null; }
  async context(kind, input) { return { kind, input: clone(input), transport_revision: this.revision }; }
}

export class NativeRuntimeHost {
  id = 'native';
  revision = 'native-host-contract-v1';

  constructor(port = new DeterministicNativePort()) {
    this.port = port;
    this.observer = null;
    this.runId = null;
    this.sequence = 0;
  }

  attachObserver(observer, runId) {
    this.observer = observer;
    this.runId = runId;
    this.port.subscribe((event) => this.emitHost('native_event', event));
  }

  emitHost(eventType, payload) {
    this.observer?.emit({ channel: 'host', event_id: `${this.runId}:native-host:${this.sequence}`, event_type: eventType, run_id: this.runId, sequence: this.sequence++, host: this.id, payload: clone(payload) });
  }

  async callModel() {
    const result = await this.port.model();
    return result;
  }

  async executeCapability({ name, args } = {}) {
    return this.port.capability(name, args ?? {});
  }

  async receiveExternalInput() {
    return this.port.external();
  }

  async readContext({ kind, input } = {}) {
    return this.port.context(kind, input);
  }
}

export function createNativeHost() { return new NativeRuntimeHost(new DeterministicNativePort()); }
