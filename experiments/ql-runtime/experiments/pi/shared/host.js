export const PI_UPSTREAM = Object.freeze({
  repository: 'earendil-works/pi',
  revision: '9d2ec7ffabe927bfad2214c1cee25b6632a78dcf',
  inspected_surfaces: [
    'packages/agent/src/agent.ts',
    'packages/coding-agent/src/core/agent-session-runtime.ts'
  ]
});

function clone(value) {
  return value === undefined ? undefined : structuredClone(value);
}

export class DeterministicPiPort {
  constructor() {
    this.sessionId = 'pi-fixture-session';
    this.messages = [];
    this.followUps = [];
    this.listeners = new Set();
    this.modelIndex = 0;
    this.tools = new Map([
      ['inspect', async (args) => ({
        ok: true,
        subject: args?.subject ?? 'matched-runtime-baseline',
        observation: 'subject verified',
        source: 'pi-tool'
      })]
    ]);
    this.modelResponses = [
      {
        content: 'Inspect the matched subject before completing.',
        capabilityCalls: [
          { id: 'pi-call-1', name: 'inspect', args: { subject: 'matched-runtime-baseline' } }
        ]
      },
      {
        content: 'Matched subject verified; baseline outcome is ready.',
        capabilityCalls: []
      }
    ];
  }

  subscribe(listener) {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  emit(event) {
    for (const listener of this.listeners) listener(clone(event));
  }

  async modelTurn() {
    const response = clone(this.modelResponses[Math.min(this.modelIndex, this.modelResponses.length - 1)]);
    this.modelIndex += 1;
    const message = {
      role: 'assistant',
      content: response.content,
      capabilityCalls: clone(response.capabilityCalls),
      timestamp: this.modelIndex
    };
    this.emit({ type: 'message_start', message });
    this.messages.push(message);
    this.emit({ type: 'message_end', message });
    this.emit({ type: 'turn_end', message, toolResults: [] });
    return response;
  }

  async executeTool(name, args) {
    const tool = this.tools.get(name);
    if (!tool) throw new Error(`Unknown Pi tool '${name}'.`);
    const toolCallId = `pi-tool-${this.messages.length}-${name}`;
    this.emit({ type: 'tool_execution_start', toolCallId, toolName: name, args: clone(args) });
    const result = await tool(clone(args));
    this.emit({ type: 'tool_execution_end', toolCallId, toolName: name, result: clone(result) });
    return result;
  }

  async takeFollowUp() {
    return this.followUps.shift() ?? null;
  }

  async readContext(kind, input) {
    return {
      kind,
      input: clone(input),
      sessionId: this.sessionId,
      messageCount: this.messages.length,
      toolNames: [...this.tools.keys()]
    };
  }
}

export class PiRuntimeHost {
  id = 'pi';
  revision = PI_UPSTREAM.revision;

  constructor(port = new DeterministicPiPort()) {
    this.port = port;
    this.observer = null;
    this.runId = null;
    this.hostSequence = 0;
    this.unsubscribe = null;
  }

  attachObserver(observer, runId) {
    this.observer = observer;
    this.runId = runId;
    this.unsubscribe?.();
    this.unsubscribe = this.port.subscribe((event) => this.emitHost('pi_event', event));
  }

  emitHost(eventType, payload) {
    this.observer?.emit({
      channel: 'host',
      event_id: `${this.runId}:pi-host:${this.hostSequence}`,
      event_type: eventType,
      run_id: this.runId,
      sequence: this.hostSequence++,
      host: this.id,
      payload: clone(payload)
    });
  }

  async callModel() {
    this.emitHost('model_requested', { session_id: this.port.sessionId });
    const result = await this.port.modelTurn();
    this.emitHost('model_returned', {
      content: result?.content ?? null,
      capability_call_count: result?.capabilityCalls?.length ?? 0
    });
    return result;
  }

  async executeCapability({ name, args } = {}) {
    this.emitHost('capability_requested', { name, args: clone(args ?? {}) });
    const result = await this.port.executeTool(name, args ?? {});
    this.emitHost('capability_returned', { name, result: clone(result) });
    return result;
  }

  async receiveExternalInput() {
    const value = await this.port.takeFollowUp();
    this.emitHost('external_input_polled', { available: value !== null && value !== undefined });
    return value;
  }

  async readContext({ kind, input } = {}) {
    const value = await this.port.readContext(kind, input);
    this.emitHost('context_read', { kind, value: clone(value) });
    return value;
  }
}

export function createPiHost() {
  return new PiRuntimeHost(new DeterministicPiPort());
}
