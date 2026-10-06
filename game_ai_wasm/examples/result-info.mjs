export function describeSyllableInfo(info, changeFuncIdx) {
  const classificationLabels = { win: '승', loopwin: '루프승', lose: '패', route: '루트 (미확정)' };
  return {
    ruleLabel: changeFuncIdx === 0 ? '두음 법칙 미적용' : changeFuncIdx === 1 ? '표준 두음 법칙' : `연결 규칙 ${changeFuncIdx}`,
    classificationLabel: classificationLabels[info.nodeType] ?? '분류 정보 없음',
  };
}

/** Explain static classification separately from the exact search verdict. */
export function explainSyllableResult(info, result, changeFuncIdx, wordMap) {
  const reductionOnly = result.visited === 0;
  const staticWin = info.nodeType === 'win' || info.nodeType === 'loopwin';
  const provenWord = result.winningMove
    ? wordMap?.content[result.winningMove[0]]?.[result.winningMove[1]]?.[0] ?? null : null;
  return {
    ...describeSyllableInfo(info, changeFuncIdx),
    methodLabel: reductionOnly ? '승패 분석으로 확정' : '추가 탐색으로 확정',
    winningWord: result.isWin ? provenWord ?? (staticWin ? info.winningWords[0] ?? null : null) : null,
    pathExplanation: result.optimalPath.length
      ? '검증된 첫 승리 수는 위에 표시합니다. 아래는 가지치기까지 저장한 참고용 탐색 기록입니다.'
      : reductionOnly ? '승패 분석으로 확정되어 DFS 경로는 생성하지 않았습니다.' : '저장된 탐색 경로가 없습니다.',
  };
}
