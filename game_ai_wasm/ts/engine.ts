import init, * as wasm from '../pkg/game_ai_wasm.js';
import type { EngineFunctions, RuleForm, WasmInput } from './types.js';

export const DEFAULT_PRECEDENCE = { rule: 0, maps: { edge: {}, node: {} } };

/** Initialize once per JS realm. Worker realms own separate WASM instances. */
export async function createEngineFunctions(wasmInput?: WasmInput): Promise<EngineFunctions> {
  await init(wasmInput === undefined ? undefined : { module_or_path: wasmInput });
  // wasm-bindgen's generated declaration uses JsValue = any. Keep the public
  // snapshot contract explicit here instead of leaking it to callers.
  const raw = wasm as unknown as Omit<EngineFunctions, 'getWcData'> & {
    getWcData(rule: RuleForm, flow: number): Awaited<ReturnType<EngineFunctions['getWcData']>>;
  };
  return {
    async getWcData(rule, flow = 0, options = {}) {
      let prepared = rule;
      const source = rule.content.wordRule.words;
      if (source.type === 'file') {
        throw new Error('Browser file sources require reading the file and passing its content as manual words');
      }
      if (source.type === 'selected') {
        const urls = raw.dictionaryUrls(source.option);
        const contents = await Promise.all(urls.map(async (url) => {
          const response = await fetch(options.dictionaryUrl?.(url) ?? url, { signal: options.signal });
          if (!response.ok) throw new Error(`Dictionary fetch failed (${response.status}): ${url}`);
          const text = await response.text();
          return text.replace(/^\uFEFF/, '').split(/\r?\n/).map((word) => word.trim()).join(' ');
        }));
        prepared = {
          ...rule,
          content: {
            ...rule.content,
            wordRule: { ...rule.content.wordRule,
              words: { type: 'manual', option: { content: contents.join(' ') } } },
          },
        };
      }
      options.signal?.throwIfAborted();
      return raw.getWcData(prepared, flow);
    },
    updateSolver: (graphs, moves, flow = 0) => raw.updateSolver(graphs, moves, flow),
    getGraph: (graphs) => raw.getGraph(graphs),
    getNextWords: (solver, history) => raw.getNextWords(solver, history),
    getSyllableInfo: (solver, syllable, changeFuncIdx = 0) => raw.getSyllableInfo(solver, syllable, changeFuncIdx),
    afterHistory: (solver, history, flow = solver.flow as 0 | 1) => raw.afterHistory(solver, history, flow),
    withHistory: (solver, history, flow = solver.flow as 0 | 1) => raw.withHistory(solver, history, flow),
    chooseMove: (solver, history, options = {}, callback) => raw.chooseMove(solver, history, options, callback),
    isGameEnd: (solver, history, stealable = false) => raw.isGameEnd(solver, history, stealable),
    searchIsWin: (graph, move, prec = DEFAULT_PRECEDENCE, timeoutMillis) =>
      raw.searchIsWin(graph, move, prec, timeoutMillis),
    startStreamingSingleThreadSearch: (graph, move, prec = DEFAULT_PRECEDENCE, timeoutMillis, callback) =>
      raw.startStreamingSingleThreadSearch(graph, move, prec, timeoutMillis, callback),
    startStreamingCriticalWordsInfo: (graph, view, flow, callback) =>
      raw.startStreamingCriticalWordsInfo(graph, view, flow, callback),
    prepareRootSearch: (graph, move, prec = DEFAULT_PRECEDENCE, timeoutMillis) =>
      raw.prepareRootSearch(graph, move, prec, timeoutMillis),
    prepareSyllableSearch: (solver, syllable, changeFuncIdx = 0, prec = DEFAULT_PRECEDENCE, timeoutMillis) =>
      raw.prepareSyllableSearch(solver, syllable, changeFuncIdx, prec, timeoutMillis),
    searchSyllable: (solver, syllable, changeFuncIdx = 0, prec = DEFAULT_PRECEDENCE, timeoutMillis) =>
      raw.searchSyllable(solver, syllable, changeFuncIdx, prec, timeoutMillis),
    searchRootBranch: (graph, move, prec = DEFAULT_PRECEDENCE, timeoutMillis, callback) =>
      raw.searchRootBranch(graph, move, prec, timeoutMillis, callback),
    finishRootSearch: (plan, results, duration) => raw.finishRootSearch(plan, results, duration),
    finishSyllableSearch: (plan, results, duration) => raw.finishSyllableSearch(plan, results, duration),
    dictionaryUrls: (option) => raw.dictionaryUrls(option),
    getPresets: () => raw.getPresets(),
  };
}
