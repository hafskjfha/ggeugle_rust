import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import test from 'node:test';

// These tests catch changes at the real Rust/JS boundary, including wasm clocks,
// entropy, serializable snapshots, duration units and dictionary loading.
const apiModule = await import('../dist/index.js');
const wasmBytes = await readFile(new URL('../pkg/game_ai_wasm_bg.wasm', import.meta.url));
const api = await apiModule.createEngineFunctions(wasmBytes);
const rule = (words) => ({
  id: 'test', metadata: { title: 'test', updatedAt: 0, color: '' },
  content: {
    wordRule: { words: { type: 'manual', option: { content: words.join(' ') } },
      regexFilter: '.*', addedWords: '', removedWords: '' },
    wordConnectionRule: { changeFuncIdx: 0, rawHeadIdx: 1, headDir: 0,
      rawTailIdx: 1, tailDir: 1 },
    postprocessing: { manner: { type: 0 }, addedWords: '', removedWords: '' },
  },
});

test('WASM solver preserves word history and exposes plain snapshots', async () => {
  const solver = await api.getWcData(rule(['가나', '가시나', '나니가']), 0);
  assert.deepEqual(api.getNextWords(solver, ['가나', '나니가']), ['가시나']);
  assert.equal(solver.wordMap.content instanceof Map, false);
  assert.equal(api.isGameEnd(solver, ['가나', '나니가', '가시나']), true);
  const before = structuredClone(solver);
  const next = api.afterHistory(solver, ['가나'], 0);
  assert.ok(next.graphs);
  assert.deepEqual(solver, before);
});

test('WASM search reports the mover outcome, milliseconds, and immutable stacks', async () => {
  const solver = await api.getWcData(rule(['사과', '과자', '자두']), 0);
  const graph = solver.graphSolver.graphs.content.winlose;
  const before = structuredClone(graph);
  const result = api.searchIsWin(graph, ['사', '과']);
  assert.equal(result.isWin, true);
  assert.deepEqual(result.optimalPath, [['사', '과']]);
  assert.equal(result.visited, 1);
  assert.ok(Number.isFinite(result.duration) && result.duration >= 0);
  const events = [];
  const streamed = api.startStreamingSingleThreadSearch(graph, ['사', '과'],
    apiModule.DEFAULT_PRECEDENCE, undefined, (event) => events.push(event));
  assert.deepEqual(events.at(-1), { action: 'done', payload: streamed });
  assert.deepEqual(events[0], { action: 'stack', payload: [['사', '과']] });
  assert.deepEqual(graph, before);
});

test('WASM AI uses browser entropy and retains game event order', async () => {
  const solver = await api.getWcData(rule(['사과']), 0);
  const events = [];
  assert.equal(api.chooseMove(solver, [], { difficulty: 0 }, (event) => events.push(event)), '사과');
  assert.deepEqual(events.filter((event) => event.action !== 'debug').map((event) => event.action),
    ['move', 'computerWin', 'messageEnd']);
  assert.equal(api.chooseMove(solver, ['사과']), null);
  assert.equal(api.chooseMove(solver, ['사과'], { stealable: true }), '사과');
});

test('invalid and expired deadlines are JavaScript errors and the module stays usable', async () => {
  const solver = await api.getWcData(rule(['사과']), 0);
  const graph = solver.graphSolver.graphs.content.winlose;
  for (const timeout of [-1, NaN, Infinity]) {
    assert.throws(() => api.searchIsWin(graph, ['사', '과'], undefined, timeout), /timeout/i);
  }
  assert.throws(() => api.searchIsWin(graph, ['사', '과'], undefined, 0),
    (error) => error.name === 'TimeoutError');
  assert.equal(api.searchIsWin(graph, ['사', '과']).isWin, true);
  assert.throws(() => api.getNextWords(solver, ['']), /character/);
});

test('dictionary URLs and built-in presets come from the Rust engine', () => {
  assert.equal(api.dictionaryUrls({ dict: 0, pos: { 명사: 1 }, cate: { 일반어: 1 } }).length, 1);
  assert.ok(api.getPresets().rules.length > 0);
});

test('actual WASM results match the existing TypeScript golden fixtures', async () => {
  const golden = JSON.parse(await readFile(new URL('../../game_ai_rust/tests/fixtures/golden.json', import.meta.url), 'utf8'));
  for (const fixture of golden.cases) {
    const solver = await api.getWcData(fixture.rule, fixture.flow);
    // Golden type lists are sorted for export; classification maps keep traversal order.
    assert.deepEqual(solver.graphSolver.typeMap[0], Object.fromEntries(fixture.types[0]), fixture.name);
    assert.deepEqual(solver.graphSolver.typeMap[1], Object.fromEntries(fixture.types[1]), fixture.name);
    assert.deepEqual(api.getNextWords(solver, []), fixture.words, fixture.name);
    const graph = api.getGraph(solver.graphSolver.graphs);
    for (const search of fixture.search) {
      const result = api.searchIsWin(graph, search.move);
      assert.equal(result.isWin, search.isWin, fixture.name);
      assert.deepEqual(result.optimalPath, search.optimalPath, fixture.name);
    }
  }
});

test('selected sources are fetched before Rust filtering and postprocessing', async (t) => {
  const server = createServer((request, response) => {
    if (request.url === '/missing') {
      response.writeHead(404).end('missing');
    } else {
      response.end('\uFEFF사과\n과자\n사과\n자두\n');
    }
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  t.after(() => new Promise((resolve) => server.close(resolve)));
  const url = `http://127.0.0.1:${server.address().port}`;
  const selected = rule([]);
  selected.content.wordRule.words = { type: 'selected', option: {
    dict: 0, pos: { 명사: 1 }, cate: { 일반어: 1 } } };
  selected.content.wordRule.removedWords = '과자';
  selected.content.wordRule.addedWords = '사과 사탕';
  const solver = await api.getWcData(selected, 0, { dictionaryUrl: () => url });
  // WordMap iterates by head/tail pair, so both 사 words precede the 자 pair.
  assert.deepEqual(api.getNextWords(solver, []), ['사과', '사탕', '자두']);
  await assert.rejects(api.getWcData(selected, 0, { dictionaryUrl: () => `${url}/missing` }), /404/);
  assert.equal(selected.content.wordRule.words.type, 'selected');
});
