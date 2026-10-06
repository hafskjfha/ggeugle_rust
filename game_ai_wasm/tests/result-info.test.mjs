import assert from 'node:assert/strict';
import test from 'node:test';

const loaded = await import('../examples/result-info.mjs').catch((error) => ({ error }));
test('the result explanation helper is available', () => assert.equal(loaded.error, undefined));
if (!loaded.error) {
  const { explainSyllableResult } = loaded;
  test('classified wins explain skipped DFS and show an actual winning word', () => {
    const info = { nodeType: 'win', winningMove: ['죄', '칩'], winningWords: ['죄칩'] };
    const result = { isWin: true, optimalPath: [], visited: 0 };
    const explanation = explainSyllableResult(info, result, 1);
    assert.equal(explanation.ruleLabel, '표준 두음 법칙');
    assert.equal(explanation.classificationLabel, '승');
    assert.equal(explanation.methodLabel, '승패 분석으로 확정');
    assert.equal(explanation.winningWord, '죄칩');
    assert.match(explanation.pathExplanation, /DFS.*생성하지/);
  });
  test('an initial route remains visible when additional search proves a win', () => {
    const info = { nodeType: 'route', winningMove: null, winningWords: [] };
    const explanation = explainSyllableResult(info, { isWin: true, visited: 8, optimalPath: [['가', '나']] }, 0);
    assert.equal(explanation.ruleLabel, '두음 법칙 미적용');
    assert.equal(explanation.classificationLabel, '루트 (미확정)');
    assert.equal(explanation.methodLabel, '추가 탐색으로 확정');
    assert.equal(explanation.winningWord, null);
  });
  test('initial routes resolved by reductions do not claim an unproven winning word', () => {
    const explanation = explainSyllableResult({ nodeType: 'route', winningWords: [] },
      { isWin: true, visited: 0, optimalPath: [] }, 1);
    assert.equal(explanation.classificationLabel, '루트 (미확정)');
    assert.equal(explanation.methodLabel, '승패 분석으로 확정');
    assert.equal(explanation.winningWord, null);
  });
  test('losing results never display a winning word from an unrelated prior result', () => {
    const explanation = explainSyllableResult({ nodeType: 'lose', winningWords: ['이전수'] },
      { isWin: false, visited: 0, optimalPath: [] }, 0);
    assert.equal(explanation.classificationLabel, '패');
    assert.equal(explanation.winningWord, null);
  });
}
