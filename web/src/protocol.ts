// RustのAnalysisReportに対応。内部ASTや整数はRust側で表示文字列にする。
export type StageResult =
  | { status: 'success'; output: string }
  | { status: 'unimplemented' | 'error' | 'skipped'; message: string };

export interface AnalysisReport {
  tokens: StageResult;
  ast: StageResult;
  inferred_type: StageResult;
  value: StageResult;
}

export type WorkerRequest = { type: 'analyze'; source: string };
export type WorkerResponse =
  | { type: 'ready' }
  | { type: 'result'; report: AnalysisReport }
  | { type: 'failure'; message: string };
