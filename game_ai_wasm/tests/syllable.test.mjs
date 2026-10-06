import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { createEngineFunctions, ParallelSearchRunner } from '../dist/index.js';
import { nodeWorkerFactory } from './support/node-worker.mjs';

const bytes = await readFile(new URL('../pkg/game_ai_wasm_bg.wasm', import.meta.url));
const api = await createEngineFunctions(bytes);
const wasmUrl = `data:application/wasm;base64,${bytes.toString('base64')}`;
const rule = (words, change = 0) => ({
  id: 'syllable', metadata: { title: 'syllable', updatedAt: 0, color: '' },
  content: {
    wordRule: { words: { type: 'manual', option: { content: words.join(' ') } },
      regexFilter: '.*', addedWords: '', removedWords: '' },
    wordConnectionRule: { changeFuncIdx: change, rawHeadIdx: 1, headDir: 0, rawTailIdx: 1, tailDir: 1 },
    postprocessing: { manner: { type: 0 }, addedWords: '', removedWords: '' },
  },
});

test('the syllable API judges the player who must start with that syllable', async () => {
  assert.equal(typeof api.searchSyllable, 'function');
  const solver = await api.getWcData(rule(['사과', '과자', '자두']));
  const before = structuredClone(solver);
  for (const [syllable, wins] of [['사', true], ['과', false], ['자', true], ['두', false], ['힣', false]]) {
    const result = api.searchSyllable(solver.graphSolver, syllable);
    assert.equal(result.isWin, wins, syllable);
    assert.ok(result.optimalPath.every(([head]) => head !== '__none'));
  }
  assert.deepEqual(solver, before);
});

test('syllables outside the tail map still obey initial-sound rules', async () => {
  assert.equal(typeof api.searchSyllable, 'function');
  const solver = await api.getWcData(rule(['이름'], 1));
  assert.equal(solver.graphSolver.typeMap[0]['리'], undefined);
  assert.equal(api.searchSyllable(solver.graphSolver, '리', 1).isWin, true);
  assert.equal(api.searchSyllable(solver.graphSolver, '리', 0).isWin, false);
});

test('parallel syllable search preserves current-player outcomes and callback perspective', async (t) => {
  const runner = new ParallelSearchRunner({ workerFactory: nodeWorkerFactory, wasmUrl });
  t.after(() => runner.terminate());
  assert.equal(typeof runner.searchSyllable, 'function');
  const solver = await api.getWcData(rule(['가나', '나무바다', '다가']));
  assert.equal(solver.graphSolver.typeMap[0]['가'], 'route');
  const events = [];
  const result = await runner.searchSyllable(solver.graphSolver, '가', 0, undefined,
    { workers: 2, timeoutMillis: 10000, onEvent: (event) => events.push(event) });
  assert.equal(result.isWin, true); // The three-word cycle ends after the starter's third move.
  assert.deepEqual(result, events.at(-1).payload);
  assert.deepEqual(result.optimalPath, api.searchSyllable(solver.graphSolver, '가').optimalPath);
  assert.ok(events.filter((event) => event.action === 'stack').every((event) =>
    event.payload.every(([head]) => head !== '__none')));
  const losing = await api.getWcData(rule(['사과', '과자', '자두']));
  assert.equal((await runner.searchSyllable(losing.graphSolver, '과')).isWin, false);
  assert.equal((await runner.searchSyllable(losing.graphSolver, '사')).isWin, true);
});

test('parallel syllable abort and timeout never return a guessed verdict', async (t) => {
  const runner = new ParallelSearchRunner({ workerFactory: nodeWorkerFactory, wasmUrl });
  t.after(() => runner.terminate());
  assert.equal(typeof runner.searchSyllable, 'function');
  const solver = await api.getWcData(rule(['사과', '과자', '자두']));
  await assert.rejects(runner.searchSyllable(solver.graphSolver, '과', 0, undefined,
    { timeoutMillis: 0 }), { name: 'TimeoutError' });
  const pending = runner.searchSyllable(solver.graphSolver, '과');
  const cancelled = assert.rejects(pending, { name: 'AbortError' });
  runner.terminate();
  await cancelled;
  assert.equal((await runner.searchSyllable(solver.graphSolver, '자')).isWin, true);
});

test('syllable validation rejects empty or multiple characters and keeps the engine usable', async () => {
  assert.equal(typeof api.searchSyllable, 'function');
  const solver = await api.getWcData(rule(['사과']));
  for (const syllable of ['', '사과']) {
    assert.throws(() => api.searchSyllable(solver.graphSolver, syllable));
  }
  assert.throws(() => api.searchSyllable(solver.graphSolver, '사', 11));
  assert.throws(() => api.searchSyllable(solver.graphSolver, '사', 0, undefined, 0),
    { name: 'TimeoutError' });
  assert.equal(api.searchSyllable(solver.graphSolver, '사').isWin, true);
});

test('a winning syllable returns a genuinely winning first move instead of a longer losing path', async (t) => {
  const runner = new ParallelSearchRunner({ workerFactory: nodeWorkerFactory, wasmUrl });
  t.after(() => runner.terminate());
  for (const [words, change, syllable] of [
    [['가가', '가나', '나나', '나다', '다가'], 0, '나'],
    [['라니', '라가니', '나라', '니나'], 1, '라'],
  ]) {
    const solver = await api.getWcData(rule(words, change));
    const graph = api.getGraph(solver.graphSolver.graphs);
    const serial = api.searchSyllable(solver.graphSolver, syllable, change);
    const parallel = await runner.searchSyllable(solver.graphSolver, syllable, change, undefined,
      { workers: 2, timeoutMillis: 10000 });
    assert.equal(serial.isWin, true);
    assert.equal(parallel.isWin, true);
    assert.equal(api.searchIsWin(graph, serial.optimalPath[0]).isWin, true);
    assert.ok(serial.winningMove);
    assert.deepEqual(serial.optimalPath[0], serial.winningMove);
    assert.equal(api.searchIsWin(graph, serial.winningMove).isWin, true);
    assert.deepEqual(parallel.winningMove, serial.winningMove);
    assert.deepEqual(parallel.optimalPath, serial.optimalPath);
  }
});

test('history queries consume exact words and do not change the original dictionary', async () => {
  assert.equal(typeof api.withHistory, 'function');
  const solver = await api.getWcData(rule(['가나', '나가']));
  assert.equal(api.searchSyllable(solver.graphSolver, '나').isWin, false);
  const remaining = api.withHistory(solver, ['가나']);
  assert.deepEqual(api.getNextWords(remaining, []), ['나가']);
  assert.equal(api.searchSyllable(remaining.graphSolver, '나').isWin, true);
  assert.deepEqual(api.getNextWords(solver, []), ['가나', '나가']);
  assert.throws(() => api.withHistory(solver, ['없는단어']));
  const pair = await api.getWcData(rule(['가나', '가시나', '나가']));
  const reduced = api.withHistory(pair, ['가나', '가나']);
  assert.deepEqual(reduced.wordMap.content['가']['나'], ['가시나']);
  assert.ok(!api.getSyllableInfo(reduced, '가').winningWords.includes('가나'));
});
