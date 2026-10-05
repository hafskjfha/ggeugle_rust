import assert from 'node:assert/strict';
import test from 'node:test';
import { BipartiteDiGraph } from '../dist/wordchain/graph/bipartite-digraph.js';
import { GraphSolver } from '../dist/wordchain/graph/graph-solver.js';
import { WordSolver } from '../dist/wordchain/word/word-solver.js';
import { WorkerRunner } from '../dist/ai/worker-runner.js';

const precedence = { rule: 0, maps: { edge: {}, node: {} } };

function searchArgs() {
  const graph = new BipartiteDiGraph();
  graph.setEdge(1, '가', '나');
  return [graph, ['가', '나'], precedence];
}

async function assertWinningSearch(runner) {
  const result = await runner.callAndTerminate('searchIsWin', searchArgs(), 5000);
  assert.equal(result.isWin, true);
  assert.equal(typeof result.duration, 'number');
  assert.ok(result.duration >= 0);
}

test('runs an engine search in a real worker and can be reused', async (t) => {
  const runner = new WorkerRunner();
  t.after(() => runner.terminate());
  await assertWinningSearch(runner);
  await assertWinningSearch(runner);
});

test('loads words and updates a serialized solver through real workers', async (t) => {
  const runner = new WorkerRunner();
  t.after(() => runner.terminate());
  const rule = {
    id: 'worker-test',
    metadata: { title: 'Worker test', updatedAt: 0, color: '' },
    content: {
      wordRule: {
        words: { type: 'manual', option: { content: '가나 나다' } },
        regexFilter: '.*',
        addedWords: '',
        removedWords: '',
      },
      wordConnectionRule: {
        changeFuncIdx: 0,
        rawHeadIdx: 1,
        headDir: 0,
        rawTailIdx: 1,
        tailDir: 1,
      },
      postprocessing: { manner: { type: 0 }, addedWords: '', removedWords: '' },
    },
  };
  const solver = WordSolver.fromObj(
    await runner.callAndTerminate('getWcData', [rule, 0], 5000),
  );
  assert.deepEqual(solver.wordMap.get('가', '나'), ['가나']);
  assert.deepEqual(solver.wordMap.get('나', '다'), ['나다']);
  const updated = GraphSolver.fromObj(
    await runner.callAndTerminate(
      'updateSolver',
      [solver.graphSolver.graphs, [['가', '나', 1]], 0],
      5000,
    ),
  );
  assert.equal(updated.graphs.union().getEdgeNum('가', '나'), 0);
  assert.equal(updated.graphs.union().getEdgeNum('나', '다'), 1);
});

test('propagates an engine error and allows the next call', async (t) => {
  const runner = new WorkerRunner();
  t.after(() => runner.terminate());
  await assert.rejects(
    runner.callAndTerminate('searchIsWin', [null, ['가', '나'], precedence], 5000),
    { name: 'TypeError' },
  );
  await assertWinningSearch(runner);
});

test('rejects a timed out call and allows the next call', async (t) => {
  const runner = new WorkerRunner();
  t.after(() => runner.terminate());
  await assert.rejects(
    runner.callAndTerminate('searchIsWin', searchArgs(), 0),
    /Timeout exceeded/,
  );
  await assertWinningSearch(runner);
});

test('terminate rejects a pending call and allows the next call', async (t) => {
  const runner = new WorkerRunner();
  t.after(() => runner.terminate());
  const pending = runner.callAndTerminate('searchIsWin', searchArgs(), 5000);
  const rejection = assert.rejects(pending, { name: 'AbortError' });
  runner.terminate();
  runner.terminate();
  await rejection;
  await assertWinningSearch(runner);
});

test('a new call cancels the previous call without cancelling its replacement', async (t) => {
  const runner = new WorkerRunner();
  t.after(() => runner.terminate());
  const first = runner.callAndTerminate('searchIsWin', searchArgs(), 5000);
  const rejection = assert.rejects(first, { name: 'AbortError' });
  const second = runner.callAndTerminate('searchIsWin', searchArgs(), 5000);
  await rejection;
  assert.equal((await second).isWin, true);
  await assertWinningSearch(runner);
});

test('rejects uncloneable arguments and cleans up before reuse', async (t) => {
  const runner = new WorkerRunner();
  t.after(() => runner.terminate());
  const args = searchArgs();
  args[0].uncloneable = () => {};
  await assert.rejects(
    runner.callAndTerminate('searchIsWin', args, 5000),
    { name: 'DataCloneError' },
  );
  await assertWinningSearch(runner);
});
