import {
  RUN_STATUS,
  isAbortRequested,
  normalizeRunRequest,
  dispatchHostCarrier
} from '../runtime-contract/index.js';

function commonEvent(runId, sequence, eventType, payload = {}) {
  return {
    channel: 'runtime',
    event_id: `${runId}:classic:${sequence}`,
    event_type: eventType,
    run_id: runId,
    sequence,
    runtime: 'classic',
    payload
  };
}

export class ClassicRuntime {
  id = 'classic';
  version = '0.1.0-foundation';

  async run(inputRequest, host, observer, signal) {
    const request = normalizeRunRequest(inputRequest);
    const runId = request.runId ?? `run:${request.taskId}:classic`;
    let eventSequence = 0;
    let iteration = 0;
    let modelCalls = 0;
    let capabilityCalls = 0;
    const history = [{ role: 'user', content: request.input }];

    const emit = (eventType, payload) => {
      observer.emit(commonEvent(runId, eventSequence++, eventType, payload));
    };

    emit('run_started', { task_id: request.taskId });

    try {
      while (iteration < request.maxSteps) {
        if (isAbortRequested(signal)) {
          emit('run_cancelled', { iteration });
          return {
            status: RUN_STATUS.CANCELLED,
            runtime: this.id,
            runtimeVersion: this.version,
            runId,
            iterations: iteration,
            modelCalls,
            capabilityCalls,
            history
          };
        }

        iteration += 1;
        const modelResult = await dispatchHostCarrier({
          host,
          carrier: { kind: 'model' },
          request,
          signal,
          payload: { history: structuredClone(history), iteration }
        });
        modelCalls += 1;
        history.push({ role: 'assistant', ...structuredClone(modelResult) });

        const calls = Array.isArray(modelResult?.capabilityCalls)
          ? modelResult.capabilityCalls
          : [];

        if (calls.length > 0) {
          for (const call of calls) {
            if (isAbortRequested(signal)) {
              emit('run_cancelled', { iteration, during: 'capability' });
              return {
                status: RUN_STATUS.CANCELLED,
                runtime: this.id,
                runtimeVersion: this.version,
                runId,
                iterations: iteration,
                modelCalls,
                capabilityCalls,
                history
              };
            }

            let result;
            try {
              result = await dispatchHostCarrier({
                host,
                carrier: {
                  kind: 'capability',
                  name: call.name,
                  args: structuredClone(call.args ?? {})
                },
                request,
                signal
              });
            } catch (error) {
              result = {
                ok: false,
                error: error instanceof Error ? error.message : String(error)
              };
            }
            capabilityCalls += 1;
            history.push({
              role: 'capability',
              name: call.name,
              callId: call.id ?? null,
              result: structuredClone(result)
            });
          }
          continue;
        }

        const followUp = await dispatchHostCarrier({
          host,
          carrier: { kind: 'human', inputKind: 'follow_up' },
          request,
          signal,
          payload: { history: structuredClone(history) }
        });

        if (followUp !== null && followUp !== undefined) {
          history.push({ role: 'user', content: structuredClone(followUp) });
          continue;
        }

        const outcome = modelResult?.content ?? modelResult ?? null;
        emit('run_completed', { iteration, outcome });
        return {
          status: RUN_STATUS.COMPLETED,
          runtime: this.id,
          runtimeVersion: this.version,
          runId,
          iterations: iteration,
          modelCalls,
          capabilityCalls,
          outcome,
          history
        };
      }

      emit('run_exhausted', { max_steps: request.maxSteps });
      return {
        status: RUN_STATUS.EXHAUSTED,
        runtime: this.id,
        runtimeVersion: this.version,
        runId,
        iterations: iteration,
        modelCalls,
        capabilityCalls,
        history
      };
    } catch (error) {
      if (isAbortRequested(signal) || error?.name === 'AbortError') {
        emit('run_cancelled', { iteration });
        return {
          status: RUN_STATUS.CANCELLED,
          runtime: this.id,
          runtimeVersion: this.version,
          runId,
          iterations: iteration,
          modelCalls,
          capabilityCalls,
          history
        };
      }

      emit('run_failed', {
        iteration,
        error: error instanceof Error ? error.message : String(error)
      });
      return {
        status: RUN_STATUS.FAILED,
        runtime: this.id,
        runtimeVersion: this.version,
        runId,
        iterations: iteration,
        modelCalls,
        capabilityCalls,
        error: error instanceof Error ? error.message : String(error),
        history
      };
    }
  }
}
