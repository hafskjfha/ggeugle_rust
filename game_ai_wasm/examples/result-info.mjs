export function describeSyllableInfo(info, changeFuncIdx) {
  const classificationLabels = { win: '승', loopwin: '루프승', lose: '패', route: '루트 (미확정)' };
  return {
    ruleLabel: changeFuncIdx === 0 ? '두음 법칙 미적용' : changeFuncIdx === 1 ? '표준 두음 법칙' : `연결 규칙 ${changeFuncIdx}`,
    classificationLabel: classificationLabels[info.nodeType] ?? '분류 정보 없음',
  };
}

/** Explain static classification separately from the exact search verdict. */
export function explainSyllableResult(info, result, changeFuncIdx) {
  const reductionOnly = result.visited === 0;
  const staticWin = info.nodeType === 'win' || info.nodeType === 'loopwin';
  return {
    ...describeSyllableInfo(info, changeFuncIdx),
    methodLabel: reductionOnly ? '승패 분석으로 확정' : '추가 탐색으로 확정',
    winningWord: result.isWin && staticWin ? info.winningWords[0] ?? null : null,
    pathExplanation: result.optimalPath.length
      ? '가지치기로 확정된 지점까지 저장한 탐색 경로입니다.'
      : reductionOnly ? '승패 분석으로 확정되어 DFS 경로는 생성하지 않았습니다.' : '저장된 탐색 경로가 없습니다.',
  };
}
