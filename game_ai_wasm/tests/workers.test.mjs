import assert from 'node:assert/strict';
import test from 'node:test';
import { controlledFactory, nextRequest, until } from './support/controlled-worker.mjs';

const loaded = await Promise.all([
  import('../dist/worker-runner.js'),
  import('../dist/parallel-search.js'),
]).then(([worker, parallel]) => ({ ...worker, ...parallel })).catch((error) => ({ error }));

test('the browser worker API is available independently of Node worker threads', () => {
  assert.equal(loaded.error, undefined);
  assert.equal(typeof loaded.WorkerRunner, 'function');
  assert.equal(typeof loaded.ParallelSearchRunner, 'function');
});

if (!loaded.error) {
  const { WorkerRunner, ParallelSearchRunner } = loaded;
  const graph = { _nodes: [[], []], _succ: [{ content: {} }, { content: {} }], _pred: [{ content: {} }, { content: {} }] };
  const move = ['가', '나'];
  const prec = { rule: 0, maps: { edge: {}, node: {} } };
  const result = { isWin: true, duration: 4, optimalPath: [['가', '나']], visited: 1 };
  const branchResult = (isWin, movement) => ({
    result: { isWin, duration: 1, optimalPath: [movement], visited: 1 },
    outcome: isWin ? 'lose' : 'win',
  });
  const plan = (moves) => ({ graph, movement: move, moves, result: null });

  test('worker events do not settle a pending call and result cleanup allows reuse', async () => {
    const { factory, workers } = controlledFactory();
    const events = [];
    const runner = new WorkerRunner({ workerFactory: factory, wasmUrl: new URL('https://example.test/engine.wasm') });
    const pending = runner.callAndTerminate('searchIsWin', [graph, move, prec], 1000, (event) => events.push(event));
    assert.equal(workers[0].options.type, 'module');
    assert.equal(workers[0].request.wasmUrl, 'https://example.test/engine.wasm');
    workers[0].event({ action: 'stack', payload: [move] });
    workers[0].result(result);
    assert.deepEqual(await pending, result);
    assert.deepEqual(events, [{ action: 'stack', payload: [move] }]);
    assert.equal(workers[0].terminated, true);
    assert.equal([...workers[0].listeners.values()].every((set) => set.size === 0), true);
    const reused = runner.callAndTerminate('searchIsWin', [graph, move, prec]);
    workers[1].result(result);
    assert.deepEqual(await reused, result);
  });

  test('replacement cancels only the old call and queued old callbacks cannot leak', async () => {
    const { factory, workers } = controlledFactory();
    const events = [];
    const runner = new WorkerRunner({ workerFactory: factory });
    const first = runner.callAndTerminate('searchIsWin', [graph, move, prec], undefined, (event) => events.push(event));
    const rejected = assert.rejects(first, { name: 'AbortError' });
    const queued = [...workers[0].listeners.get('message')][0];
    const second = runner.callAndTerminate('searchIsWin', [graph, move, prec]);
    queued({ data: { type: 'event', event: { action: 'stack', payload: [move] } } });
    queued({ data: { type: 'result', result: { ...result, isWin: false } } });
    workers[1].result(result);
    await rejected;
    assert.deepEqual(await second, result);
    assert.deepEqual(events, []);
  });

  test('termination rejects once and remains idempotent and reusable', async () => {
    const { factory, workers } = controlledFactory();
    const runner = new WorkerRunner({ workerFactory: factory });
    const pending = runner.callAndTerminate('searchIsWin', [graph, move, prec]);
    const rejected = assert.rejects(pending, { name: 'AbortError' });
    runner.terminate();
    runner.terminate();
    await rejected;
    const next = runner.callAndTerminate('searchIsWin', [graph, move, prec]);
    workers[1].result(result);
    assert.equal((await next).isWin, true);
  });

  test('worker timeout aborts startup and rejects with TimeoutError', async () => {
    const { factory, workers } = controlledFactory();
    const runner = new WorkerRunner({ workerFactory: factory });
    await assert.rejects(runner.callAndTerminate('searchIsWin', [graph, move, prec], 0), { name: 'TimeoutError' });
    assert.equal(workers.every((worker) => worker.terminated), true);
  });

  test('worker responses after the absolute deadline reject without emitting queued progress', async () => {
    const { factory, workers } = controlledFactory();
    const events = [];
    const runner = new WorkerRunner({ workerFactory: factory });
    const pending = runner.callAndTerminate('searchIsWin', [graph, move, prec], 1, (event) => events.push(event));
    const rejected = assert.rejects(pending, { name: 'TimeoutError' });
    const untilExpired = performance.now() + 12;
    while (performance.now() < untilExpired) { /* Hold the timeout callback in the event queue. */ }
    workers[0].event({ action: 'stack', payload: [move] });
    workers[0].result(result);
    await rejected;
    assert.deepEqual(events, []);
    assert.equal(workers[0].terminated, true);
  });

  test('worker event callbacks can replace a call without stale results affecting the replacement', async () => {
    const { factory, workers } = controlledFactory();
    const runner = new WorkerRunner({ workerFactory: factory });
    let replacement;
    const pending = runner.callAndTerminate('searchIsWin', [graph, move, prec], 1000, () => {
      replacement = runner.callAndTerminate('searchIsWin', [graph, ['다', '라'], prec]);
    });
    const rejected = assert.rejects(pending, { name: 'AbortError' });
    const queued = [...workers[0].listeners.get('message')][0];
    workers[0].event({ action: 'stack', payload: [move] });
    queued({ data: { type: 'result', result: { ...result, isWin: false } } });
    workers[1].result(result);
    await rejected;
    assert.deepEqual(await replacement, result);
  });

  for (const failure of ['engine', 'runtime', 'messageerror', 'malformed', 'clone']) {
    test(`worker ${failure} failure rejects and cleans resources before reuse`, async () => {
      const { factory, workers } = controlledFactory();
      const runner = new WorkerRunner({ workerFactory: factory });
      const args = failure === 'clone' ? [() => {}] : [graph, move, prec];
      const pending = runner.callAndTerminate('searchIsWin', args, 1000);
      const rejected = assert.rejects(pending, failure === 'engine' ? { name: 'RangeError', message: 'engine failed' } : undefined);
      if (failure === 'engine') workers[0].error();
      if (failure === 'runtime') workers[0].dispatch('error', { message: 'worker crashed' });
      if (failure === 'messageerror') workers[0].dispatch('messageerror', {});
      if (failure === 'malformed') workers[0].dispatch('message', { data: { type: 'unexpected' } });
      await rejected;
      assert.equal(workers[0].terminated, true);
      const next = runner.callAndTerminate('searchIsWin', [graph, move, prec]);
      workers[1].result(result);
      assert.equal((await next).isWin, true);
    });
  }

  test('parallel search bounds concurrency and aggregates out-of-order results by original index', async () => {
    const { factory, workers } = controlledFactory();
    const runner = new ParallelSearchRunner({ workerFactory: factory });
    const branches = [['나', '다'], ['나', '라'], ['나', '마']];
    const events = [];
    const pending = runner.search(graph, move, prec, { workers: 2, onEvent: (event) => events.push(event) });
    (await nextRequest(workers, 'prepareRootSearch')).result(plan(branches));
    await until(() => workers.filter((worker) => worker.request?.method === 'searchRootBranch').length === 2);
    const initial = workers.filter((worker) => worker.request?.method === 'searchRootBranch');
    initial[1].event({ action: 'stack', payload: [branches[1]] });
    initial[1].result(branchResult(false, branches[1]));
    const seen = new Set(initial);
    const third = await nextRequest(workers, 'searchRootBranch', seen);
    assert.equal(initial[0].terminated, false);
    third.result(branchResult(false, branches[2]));
    initial[0].result(branchResult(false, branches[0]));
    const finalizer = await nextRequest(workers, 'finishRootSearch');
    assert.deepEqual(finalizer.request.args[1], [
      branchResult(false, branches[0]), branchResult(false, branches[1]), branchResult(false, branches[2]),
    ]);
    assert.equal(typeof finalizer.request.args[2], 'number');
    finalizer.result(result);
    assert.deepEqual(await pending, result);
    assert.equal(events[0].branchIndex, 1);
    assert.equal(events[0].action, 'stack');
    assert.equal(events.at(-1).action, 'done');
    assert.equal(events.at(-1).branchIndex, null);
    assert.equal(workers.every((worker) => worker.terminated), true);
  });

  test('parallel early stop waits for the winning prefix and cancels later responses', async () => {
    const { factory, workers } = controlledFactory();
    const runner = new ParallelSearchRunner({ workerFactory: factory });
    const branches = [['나', '다'], ['나', '라'], ['나', '마'], ['나', '바']];
    const pending = runner.search(graph, move, prec, { workers: 3 });
    (await nextRequest(workers, 'prepareRootSearch')).result(plan(branches));
    await until(() => workers.filter((worker) => worker.request?.method === 'searchRootBranch').length === 3);
    const initial = workers.filter((worker) => worker.request?.method === 'searchRootBranch');
    initial[1].result(branchResult(true, branches[1]));
    await new Promise((resolve) => setImmediate(resolve));
    assert.equal(workers.some((worker) => worker.request?.method === 'finishRootSearch'), false);
    assert.equal(workers.filter((worker) => worker.request?.method === 'searchRootBranch').length, 3);
    initial[0].result(branchResult(false, branches[0]));
    const finalizer = await nextRequest(workers, 'finishRootSearch');
    assert.equal(initial[2].terminated, true);
    assert.deepEqual(finalizer.request.args[1], [branchResult(false, branches[0]), branchResult(true, branches[1]), null, null]);
    finalizer.result({ ...result, isWin: false });
    assert.equal((await pending).isWin, false);
  });

  test('terminal plans skip branch workers and publish one completion', async () => {
    const { factory, workers } = controlledFactory();
    const runner = new ParallelSearchRunner({ workerFactory: factory });
    const events = [];
    const pending = runner.search(graph, move, prec, { onEvent: (event) => events.push(event) });
    (await nextRequest(workers, 'prepareRootSearch')).result({ ...plan([]), result });
    assert.deepEqual(await pending, result);
    assert.equal(workers.length, 1);
    assert.deepEqual(events, [{ action: 'done', payload: result, branchIndex: null }]);
  });

  test('parallel termination rejects and suppresses queued branch events and completion', async () => {
    const { factory, workers } = controlledFactory();
    const runner = new ParallelSearchRunner({ workerFactory: factory });
    const events = [];
    const pending = runner.search(graph, move, prec, { workers: 2, onEvent: (event) => events.push(event) });
    const rejected = assert.rejects(pending, { name: 'AbortError' });
    (await nextRequest(workers, 'prepareRootSearch')).result(plan([['나', '다'], ['나', '라']]));
    const branch = await nextRequest(workers, 'searchRootBranch');
    const queued = [...branch.listeners.get('message')][0];
    runner.terminate();
    queued({ data: { type: 'event', event: { action: 'stack', payload: [move] } } });
    queued({ data: { type: 'result', result: branchResult(true, move) } });
    await rejected;
    assert.deepEqual(events, []);
    assert.equal(workers.every((worker) => worker.terminated), true);
  });

  test('AbortSignal cancellation also stops planning and accepts an already aborted signal', async () => {
    const { factory, workers } = controlledFactory();
    const runner = new ParallelSearchRunner({ workerFactory: factory });
    const controller = new AbortController();
    const pending = runner.search(graph, move, prec, { signal: controller.signal });
    const rejected = assert.rejects(pending, { name: 'AbortError' });
    controller.abort();
    await rejected;
    assert.equal(workers.every((worker) => worker.terminated), true);
    await assert.rejects(runner.search(graph, move, prec, { signal: controller.signal }), { name: 'AbortError' });
  });

  test('parallel replacement cannot deliver the old result into a new search', async () => {
    const { factory, workers } = controlledFactory();
    const runner = new ParallelSearchRunner({ workerFactory: factory });
    const first = runner.search(graph, move, prec);
    const rejected = assert.rejects(first, { name: 'AbortError' });
    const old = await nextRequest(workers, 'prepareRootSearch');
    const queued = [...old.listeners.get('message')][0];
    const second = runner.search(graph, ['다', '라'], prec);
    queued({ data: { type: 'result', result: { ...plan([]), result: { ...result, isWin: false } } } });
    const newPlanner = await nextRequest(workers, 'prepareRootSearch', new Set([old]));
    newPlanner.result({ ...plan([]), result });
    await rejected;
    assert.equal((await second).isWin, true);
  });

  test('the total deadline covers planning and finalization rather than resetting per phase', async (t) => {
    t.mock.timers.enable({ apis: ['setTimeout'] });
    const { factory, workers } = controlledFactory();
    const runner = new ParallelSearchRunner({ workerFactory: factory });
    const pending = runner.search(graph, move, prec, { timeoutMillis: 20, workers: 1 });
    const rejected = assert.rejects(pending, { name: 'TimeoutError' });
    const planner = await nextRequest(workers, 'prepareRootSearch');
    t.mock.timers.tick(8);
    planner.result(plan([['나', '다']]));
    const branch = await nextRequest(workers, 'searchRootBranch');
    branch.result(branchResult(false, ['나', '다']));
    const finalizer = await nextRequest(workers, 'finishRootSearch');
    t.mock.timers.tick(12);
    await rejected;
    assert.equal(finalizer.terminated, true);
    assert.equal(workers.every((worker) => worker.terminated), true);
  });

  test('an expired deadline rejects even if a worker result beats the timeout callback', async () => {
    const { factory, workers } = controlledFactory();
    const runner = new ParallelSearchRunner({ workerFactory: factory });
    const pending = runner.search(graph, move, prec, { timeoutMillis: 1 });
    const rejected = assert.rejects(pending, { name: 'TimeoutError' });
    const untilExpired = performance.now() + 5;
    while (performance.now() < untilExpired) { /* Keep the timer callback queued. */ }
    workers[0].result({ ...plan([]), result });
    await rejected;
    assert.equal(workers.every((worker) => worker.terminated), true);
  });

  test('completion callbacks may start a new search without aborting the completed result', async () => {
    const { factory, workers } = controlledFactory();
    const runner = new ParallelSearchRunner({ workerFactory: factory });
    let replacement;
    const pending = runner.search(graph, move, prec, { onEvent(event) {
      if (event.action === 'done') replacement = runner.search(graph, ['다', '라'], prec);
    } });
    const old = await nextRequest(workers, 'prepareRootSearch');
    old.result({ ...plan([]), result });
    assert.deepEqual(await pending, result);
    const next = await nextRequest(workers, 'prepareRootSearch', new Set([old]));
    next.result({ ...plan([]), result: { ...result, isWin: false } });
    assert.equal((await replacement).isWin, false);
  });

  test('branch errors reject the whole search and clean up every active slot', async () => {
    const { factory, workers } = controlledFactory();
    const runner = new ParallelSearchRunner({ workerFactory: factory });
    const pending = runner.search(graph, move, prec, { workers: 2 });
    const rejected = assert.rejects(pending, { name: 'RangeError' });
    (await nextRequest(workers, 'prepareRootSearch')).result(plan([['나', '다'], ['나', '라']]));
    (await nextRequest(workers, 'searchRootBranch')).error();
    await rejected;
    assert.equal(workers.every((worker) => worker.terminated), true);
  });

  for (const options of [{ workers: 0 }, { workers: 1.5 }, { workers: Infinity }, { workers: 1000 }, { timeoutMillis: -1 }, { timeoutMillis: NaN }]) {
    test(`invalid parallel options reject before creating workers: ${JSON.stringify(options)}`, async () => {
      const { factory, workers } = controlledFactory();
      const runner = new ParallelSearchRunner({ workerFactory: factory });
      await assert.rejects(runner.search(graph, move, prec, options), { name: 'RangeError' });
      assert.equal(workers.length, 0);
    });
  }
}
