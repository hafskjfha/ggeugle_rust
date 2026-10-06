import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { nodeWorkerFactory } from './support/node-worker.mjs';

const loaded = await import('../dist/index.js').catch((error) => ({ error }));
test('the WASM browser facade and worker entry have been built', () => {
  assert.equal(loaded.error, undefined);
});
if (!loaded.error) {
  const { createEngineFunctions, WorkerRunner, ParallelSearchRunner } = loaded;
  const bytes = await readFile(new URL('../pkg/game_ai_wasm_bg.wasm', import.meta.url));
  const wasmUrl = `data:application/wasm;base64,${bytes.toString('base64')}`;
  const api = await createEngineFunctions(bytes);
  const wordsRule = (words) => ({
    id: 'workers', metadata: { title: 'workers', color: '', updatedAt: 0 },
    content: {
      wordRule: { words: { type: 'manual', option: { content: words.join(' ') } },
        regexFilter: '.*', addedWords: '', removedWords: '' },
      wordConnectionRule: { changeFuncIdx: 0, rawHeadIdx: 1, headDir: 0, rawTailIdx: 1, tailDir: 1 },
      postprocessing: { manner: { type: 0 }, addedWords: '', removedWords: '' },
    },
  });

  test('real WASM worker builds a solver, searches, and emits progress', async (t) => {
    const runner = new WorkerRunner({ workerFactory: nodeWorkerFactory, wasmUrl });
    t.after(() => runner.terminate());
    const solver = await runner.callAndTerminate('getWcData', [wordsRule(['사과', '과자', '자두']), 0], 10000);
    assert.deepEqual(api.getNextWords(solver, ['사과']), ['과자']);
    const events = [];
    const result = await runner.callAndTerminate('startStreamingSingleThreadSearch',
      [solver.graphSolver.graphs.content.winlose, ['사', '과'], undefined, undefined],
      10000, (event) => events.push(event));
    assert.equal(result.isWin, true);
    assert.deepEqual(events.at(-1), { action: 'done', payload: result });
  });

  test('parallel search with real WASM workers retains serial outcome and path', async (t) => {
    const runner = new ParallelSearchRunner({ workerFactory: nodeWorkerFactory, wasmUrl });
    t.after(() => runner.terminate());
    const solver = await api.getWcData(wordsRule(['가가', '가나', '나가', '나다', '다가']), 0);
    const graph = api.getGraph(solver.graphSolver.graphs);
    const original = structuredClone(graph);
    const movement = ['나', '가'];
    assert.equal(api.prepareRootSearch(graph, movement).moves.length, 2);
    const serial = api.searchIsWin(graph, movement);
    const events = [];
    const parallel = await runner.search(graph, movement, undefined,
      { workers: 2, timeoutMillis: 10000, onEvent: (event) => events.push(event) });
    assert.equal(parallel.isWin, serial.isWin);
    assert.deepEqual(parallel.optimalPath, serial.optimalPath);
    assert.equal(parallel.visited, serial.visited);
    assert.deepEqual(graph, original);
    assert.equal(events.at(-1).action, 'done');
    assert.ok(events.some((event) => event.action === 'stack' && event.branchIndex !== null));
  });

  test('real worker engine failure and abort leave the runner reusable', async (t) => {
    const runner = new WorkerRunner({ workerFactory: nodeWorkerFactory, wasmUrl });
    t.after(() => runner.terminate());
    const solver = await api.getWcData(wordsRule(['사과']), 0);
    const graph = solver.graphSolver.graphs.content.winlose;
    await assert.rejects(runner.callAndTerminate('searchIsWin', [graph, ['사', '과'], undefined, 0], 10000),
      { name: 'TimeoutError' });
    const pending = runner.callAndTerminate('searchIsWin', [graph, ['사', '과']], 10000);
    const cancelled = assert.rejects(pending, { name: 'AbortError' });
    runner.terminate();
    await cancelled;
    assert.equal((await runner.callAndTerminate('searchIsWin', [graph, ['사', '과']], 10000)).isWin, true);
  });
}
