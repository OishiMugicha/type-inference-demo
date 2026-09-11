import './style.css';
import type { AnalysisReport, StageResult, WorkerRequest, WorkerResponse } from './protocol';

function element<T extends HTMLElement>(selector: string): T {
  const result = document.querySelector<T>(selector);
  if (!result) throw new Error(`Missing element: ${selector}`);
  return result;
}

const source = element<HTMLTextAreaElement>('#source');
const sample = element<HTMLSelectElement>('#sample');
const run = element<HTMLButtonElement>('#run');
const cancel = element<HTMLButtonElement>('#cancel');
const status = element<HTMLParagraphElement>('#status');
const results = element<HTMLDivElement>('#results');

const samples = [
  { label: 'let多相', code: 'let id = fun x -> x in\nlet a = id 42 in\nid true' },
  { label: '算術と条件分岐', code: 'if 1 + 2 * 3 < 10 then 42 else 0' },
  { label: '恒等関数', code: 'fun x -> x' },
  { label: '静的スコープ', code: 'let x = 10 in\nlet f = fun y -> x + y in\nlet x = 100 in\nf 2' },
  { label: '型エラー', code: '1 + true' },
];
samples.forEach(({ label }, index) => sample.add(new Option(label, String(index))));
source.value = samples[0].code;
sample.addEventListener('change', () => {
  source.value = samples[Number(sample.value)].code;
  clearResults('入力が変わりました。実行して結果を確認してください。');
});

const stages: { key: keyof AnalysisReport; name: string; detail: string }[] = [
  { key: 'tokens', name: 'Token', detail: '字句解析' },
  { key: 'ast', name: 'AST', detail: '構文解析' },
  { key: 'inferred_type', name: 'Type', detail: '型推論' },
  { key: 'value', name: 'Value', detail: '評価' },
];
const panels = new Map<keyof AnalysisReport, { badge: HTMLElement; output: HTMLElement }>();
stages.forEach(({ key, name, detail }, index) => {
  const panel = document.createElement('section');
  panel.className = 'result-panel';
  panel.dataset.stage = key;
  panel.setAttribute('aria-label', detail);
  panel.innerHTML = `<div class="panel-heading"><h2><span class="step">0${index + 1}</span>${name}<small>${detail}</small></h2><span class="badge"></span></div><pre></pre>`;
  panels.set(key, { badge: panel.querySelector('.badge')!, output: panel.querySelector('pre')! });
  results.append(panel);
});

const labels: Record<StageResult['status'], string> = {
  success: '成功', unimplemented: '未実装', error: 'エラー', skipped: '未実行',
};

function render(key: keyof AnalysisReport, result: StageResult): void {
  const panel = panels.get(key)!;
  panel.badge.textContent = labels[result.status];
  panel.badge.dataset.status = result.status;
  panel.output.textContent = result.status === 'success' ? result.output : result.message;
}

function clearResults(message: string): void {
  for (const { key } of stages) render(key, { status: 'skipped', message });
}
clearResults('式を実行すると、ここに結果が表示されます。');

let worker: Worker | undefined;
let ready = false;
let busy = false;

function controls(): void {
  run.disabled = !ready || busy;
  cancel.disabled = !busy;
  source.disabled = busy;
  sample.disabled = busy;
  results.setAttribute('aria-busy', String(busy));
}

function startWorker(message = '実行できます。'): void {
  worker?.terminate();
  ready = false;
  busy = false;
  controls();
  const current = new Worker(new URL('./analysis.worker.ts', import.meta.url), { type: 'module' });
  worker = current;
  current.onmessage = (event: MessageEvent<WorkerResponse>) => {
    if (worker !== current) return;
    const response = event.data;
    if (response.type === 'ready') {
      ready = true;
      status.textContent = message;
    } else if (response.type === 'result') {
      busy = false;
      for (const { key } of stages) render(key, response.report[key]);
      status.textContent = '処理が完了しました。各段階の結果を確認してください。';
    } else {
      recover(response.message);
      return;
    }
    controls();
  };
  current.onerror = (event) => {
    if (worker !== current) return;
    event.preventDefault();
    recover(event.message || 'Workerの起動に失敗しました。');
  };
}

function recover(message: string): void {
  worker?.terminate();
  worker = undefined;
  ready = false;
  busy = false;
  controls();
  status.textContent = `実行環境エラー: ${message} 「再接続」で読み込み直せます。`;
  clearResults('処理を完了できませんでした。');
  run.textContent = '再接続';
  run.disabled = false;
}

function execute(): void {
  if (busy) return;
  if (!ready) {
    run.textContent = '実行する →';
    status.textContent = 'Wasmを読み込んでいます…';
    startWorker();
    return;
  }
  busy = true;
  controls();
  clearResults('処理中…');
  status.textContent = '実行中です。時間がかかる場合は中止できます。';
  const request: WorkerRequest = { type: 'analyze', source: source.value };
  worker!.postMessage(request);
}

run.addEventListener('click', execute);
cancel.addEventListener('click', () => {
  clearResults('実行を中止しました。');
  status.textContent = '実行を中止しました。Wasmを読み込み直しています…';
  startWorker('実行を中止しました。もう一度実行できます。');
});
source.addEventListener('input', () => clearResults('入力が変わりました。実行して結果を確認してください。'));
source.addEventListener('keydown', (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault();
    if (!run.disabled) execute();
  }
});
startWorker();
