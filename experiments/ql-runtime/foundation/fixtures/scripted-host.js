export class ScriptedHost {
  constructor({
    id = 'scripted-host',
    revision = '1',
    modelResponses = [],
    capabilities = {},
    externalInputs = [],
    context = {}
  } = {}) {
    this.id = id;
    this.revision = revision;
    this.modelResponses = structuredClone(modelResponses);
    this.capabilities = capabilities;
    this.externalInputs = structuredClone(externalInputs);
    this.context = structuredClone(context);
    this.modelIndex = 0;
    this.externalIndex = 0;
    this.hostSequence = 0;
    this.observer = null;
    this.runId = 'unbound-run';
  }

  attachObserver(observer, runId) {
    this.observer = observer;
    this.runId = runId;
  }

  emit(eventType, payload = {}) {
    this.observer?.emit({
      channel: 'host',
      event_id: `${this.runId}:host:${this.hostSequence}`,
      event_type: eventType,
      run_id: this.runId,
      sequence: this.hostSequence++,
      host_id: this.id,
      payload: structuredClone(payload)
    });
  }

  async callModel(input) {
    this.emit('model_call', { index: this.modelIndex, input });
    const response = this.modelResponses[this.modelIndex++] ?? { content: 'scripted-model-result', capabilityCalls: [] };
    this.emit('model_return', { response });
    return structuredClone(response);
  }

  async executeCapability({ name, args }) {
    this.emit('capability_call', { name, args });
    const handler = this.capabilities[name];
    if (!handler) {
      const result = { ok: false, error: `Unknown capability '${name}'.` };
      this.emit('capability_return', { name, result });
      return result;
    }
    const result = await handler(structuredClone(args));
    this.emit('capability_return', { name, result });
    return structuredClone(result);
  }

  async receiveExternalInput(input) {
    this.emit('external_input_poll', { input });
    if (this.externalIndex >= this.externalInputs.length) {
      return null;
    }
    const result = this.externalInputs[this.externalIndex++];
    this.emit('external_input_return', { result });
    return structuredClone(result);
  }

  async readContext(input) {
    this.emit('context_read', { input });
    return structuredClone(this.context);
  }
}
