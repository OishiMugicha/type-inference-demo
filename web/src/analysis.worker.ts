import init, { analyze } from '../pkg/tiny_ml_wasm';
import wasmUrl from '../pkg/tiny_ml_wasm_bg.wasm?url';
import type { AnalysisReport, WorkerRequest, WorkerResponse } from './protocol';

function send(message: WorkerResponse): void {
  self.postMessage(message);
}

function failure(error: unknown): void {
  send({ type: 'failure', message: error instanceof Error ? error.message : String(error) });
}

async function start(): Promise<void> {
  await init({ module_or_path: wasmUrl });
  self.onmessage = (event: MessageEvent<WorkerRequest>) => {
    if (event.data.type !== 'analyze') return;
    try {
      const report = JSON.parse(analyze(event.data.source)) as AnalysisReport;
      send({ type: 'result', report });
    } catch (error) {
      failure(error);
    }
  };
  send({ type: 'ready' });
}

void start().catch(failure);
