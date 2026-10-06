import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { createEngineFunctions } from '../dist/index.js';
const api = await createEngineFunctions(await readFile(new URL('../pkg/game_ai_wasm_bg.wasm', import.meta.url)));
function rule(content, change = 0) {
  return { id: 'info', metadata: { title: '', color: '', updatedAt: 0 }, content: {
    wordRule: { words: { type: 'manual', option: { content } }, regexFilter: '.*', addedWords: '', removedWords: '' },
    wordConnectionRule: { changeFuncIdx: change, rawHeadIdx: 1, headDir: 0, rawTailIdx: 1, tailDir: 1 },
    postprocessing: { manner: { type: 0 }, addedWords: '', removedWords: '' },
  } };
}
test('WASM exposes initial classification and a genuine winning dictionary word', async () => {
  assert.equal(typeof api.getSyllableInfo, 'function');
  const solver = await api.getWcData(rule('사과 과자 자두'));
  assert.deepEqual(api.getSyllableInfo(solver, '자'), {
    nodeType: 'win', winningMove: ['자', '두'], winningWords: ['자두'],
  });
  assert.deepEqual(api.getSyllableInfo(solver, '과'), {
    nodeType: 'lose', winningMove: null, winningWords: [],
  });
  assert.equal(api.getSyllableInfo(solver, '사').nodeType, 'win');
});
test('initial route information never invents a static winning word', async () => {
  assert.equal(typeof api.getSyllableInfo, 'function');
  const solver = await api.getWcData(rule('가나 나무바다 다가'));
  assert.deepEqual(api.getSyllableInfo(solver, '가'), {
    nodeType: 'route', winningMove: null, winningWords: [],
  });
});
test('missing-tail witnesses choose a winning transformed head over a losing alternative', async () => {
  const solver = await api.getWcData(rule('리이 이나 나이', 1));
  const info = api.getSyllableInfo(solver, '리', 1);
  assert.equal(info.nodeType, 'win');
  assert.deepEqual(info.winningMove, ['리', '이']);
  assert.deepEqual(info.winningWords, ['리이']);
  assert.equal(api.searchIsWin(api.getGraph(solver.graphSolver.graphs), info.winningMove).isWin, true);
});
