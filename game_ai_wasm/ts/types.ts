export type NodePos = 0 | 1;
export type NodeType = 'win' | 'loopwin' | 'lose' | 'route';
export type SingleMove = [string, string];
export type Edge = [string, string, number];
export type NodeMap<T> = [Record<string, T>, Record<string, T>];
export interface EdgeMap<T> { content: Record<string, Record<string, T>> }
/** Serializable Rust graph; use engine functions to operate on snapshots. */
export interface Graph {
  _nodes: [string[], string[]];
  _succ: [EdgeMap<number>, EdgeMap<number>];
  _pred: [EdgeMap<number>, EdgeMap<number>];
}
export interface GraphPartitions {
  content: Record<string, Graph>;
  keys: string[];
}
export interface GraphSolver {
  graphs: GraphPartitions;
  typeMap: NodeMap<NodeType>;
  depthMap: NodeMap<number>;
  loopMap: Record<string, string>;
  pairManager: unknown;
  sccMap: NodeMap<number>;
  flow: number;
}
export interface WordSolver {
  graphSolver: GraphSolver;
  wordMap: EdgeMap<string[]>;
  headIdx: number;
  tailIdx: number;
  flow: number;
}
export interface SyllableInfo {
  nodeType: NodeType;
  winningMove: SingleMove | null;
  winningWords: string[];
}
export interface PrecInfo {
  rule: number;
  maps: { edge: Record<string, Record<string, number>>; node: Record<string, number> };
}
export interface SelectedWordsOption {
  dict: number;
  pos: Record<string, 0 | 1>;
  cate: Record<string, 0 | 1>;
}
export type WordSource =
  | { type: 'manual'; option: { content: string } }
  | { type: 'selected'; option: SelectedWordsOption }
  | { type: 'file'; option: { path: string } };
export interface RuleForm {
  id: string;
  metadata: { title: string; updatedAt: number; color: string };
  content: {
    wordRule: { words: WordSource; regexFilter: string; addedWords: string; removedWords: string };
    wordConnectionRule: {
      changeFuncIdx: number; rawHeadIdx: number; headDir: NodePos;
      rawTailIdx: number; tailDir: NodePos;
    };
    postprocessing: {
      manner: { type: 0 | 1 | 2 | 3; nextWordsLimit?: number };
      addedWords: string; removedWords: string;
    };
  };
}
/** duration is milliseconds throughout this package. */
export interface SearchResult {
  isWin: boolean;
  duration: number;
  optimalPath: SingleMove[];
  visited: number;
}
export interface SyllableSearchResult extends SearchResult {
  /** A first move proven to win in this remaining-word state, when available. */
  winningMove: SingleMove | null;
}
export type SearchEvent =
  | { action: 'stack'; payload: SingleMove[] }
  | { action: 'done'; payload: SearchResult };
export interface RootSearchPlan {
  graph: Graph;
  movement: SingleMove;
  moves: SingleMove[];
  result: SearchResult | null;
}
export interface RootBranchResult { result: SearchResult; outcome: NodeType }
export interface AiOptions {
  difficulty?: 0 | 1 | 2;
  /** Milliseconds per candidate; defaults to 1000. */
  calculatingDuration?: number;
  stealable?: boolean;
  precedence?: PrecInfo;
  flow?: NodePos;
}
export type GameEvent =
  | { action: 'debug' | 'move'; payload: string }
  | { action: 'computerWin' | 'messageEnd' };
export interface CriticalWordsInfo {
  move: SingleMove;
  difference: { win: string[]; lose: string[]; loopwin: string[] };
}
export interface Presets {
  rules: RuleForm[];
  precedenceMaps: Record<string, PrecInfo['maps']>;
  changeFuncInfo: { title: string }[];
  dicts: unknown[];
}
export type WasmInput = string | URL | Uint8Array | WebAssembly.Module;
export interface DictionaryLoadOptions {
  signal?: AbortSignal;
  /** Redirect the built-in dictionary URLs, e.g. to a same-origin proxy. */
  dictionaryUrl?: (url: string) => string;
}
export interface EngineFunctions {
  getWcData(rule: RuleForm, flow?: NodePos, options?: DictionaryLoadOptions): Promise<WordSolver>;
  updateSolver(graphs: GraphPartitions, moves: Edge[], flow?: NodePos): GraphSolver;
  getGraph(graphs: GraphPartitions): Graph;
  getNextWords(solver: WordSolver, history: string[]): string[];
  getSyllableInfo(solver: WordSolver, syllable: string, changeFuncIdx?: number): SyllableInfo;
  afterHistory(solver: WordSolver, history: string[], flow?: NodePos): GraphSolver;
  withHistory(solver: WordSolver, history: string[], flow?: NodePos): WordSolver;
  chooseMove(solver: WordSolver, history: string[], options?: AiOptions,
    callback?: (event: GameEvent) => void): string | null;
  isGameEnd(solver: WordSolver, history: string[], stealable?: boolean): boolean;
  searchIsWin(graph: Graph, move: SingleMove, prec?: PrecInfo, timeoutMillis?: number): SearchResult;
  startStreamingSingleThreadSearch(graph: Graph, move: SingleMove, prec: PrecInfo | undefined,
    timeoutMillis: number | undefined, callback: (event: SearchEvent) => void): SearchResult;
  startStreamingCriticalWordsInfo(graph: Graph, view: NodePos, flow: NodePos,
    callback: (event: CriticalWordsInfo) => void): void;
  prepareRootSearch(graph: Graph, move: SingleMove, prec?: PrecInfo,
    timeoutMillis?: number): RootSearchPlan;
  prepareSyllableSearch(solver: GraphSolver, syllable: string, changeFuncIdx?: number,
    prec?: PrecInfo, timeoutMillis?: number): RootSearchPlan;
  /** isWin describes the player whose turn starts with the queried syllable. */
  searchSyllable(solver: GraphSolver, syllable: string, changeFuncIdx?: number,
    prec?: PrecInfo, timeoutMillis?: number): SyllableSearchResult;
  searchRootBranch(graph: Graph, move: SingleMove, prec?: PrecInfo, timeoutMillis?: number,
    callback?: (event: SearchEvent) => void): RootBranchResult;
  finishRootSearch(plan: RootSearchPlan, results: (RootBranchResult | null)[], duration: number): SearchResult;
  finishSyllableSearch(plan: RootSearchPlan, results: (RootBranchResult | null)[], duration: number): SyllableSearchResult;
  dictionaryUrls(option: SelectedWordsOption): string[];
  getPresets(): Presets;
}
