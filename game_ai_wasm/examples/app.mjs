import { WorkerRunner, ParallelSearchRunner } from '../dist/index.js';
import { parseDictionary, readDictionaryFile } from './dictionary.mjs';
import { describeSyllableInfo, explainSyllableResult } from './result-info.mjs';

const loader = new WorkerRunner();
const searcher = new ParallelSearchRunner();
const form = document.querySelector('#query-form');
const words = document.querySelector('#words');
const historyInput = document.querySelector('#history');
const syllableInput = document.querySelector('#syllable');
const ruleInput = document.querySelector('#change-rule');
const workersInput = document.querySelector('#workers');
const timeoutInput = document.querySelector('#timeout');
const fileInput = document.querySelector('#dictionary-file');
const fileSummary = document.querySelector('#file-summary');
const searchButton = document.querySelector('#search');
const cancelButton = document.querySelector('#cancel');
const resultPanel = document.querySelector('#result');
const resultTitle = document.querySelector('#result-title');
const resultMessage = document.querySelector('#result-message');
const progress = document.querySelector('#progress');
const details = document.querySelector('#result-details');
const resultContext = document.querySelector('#result-context');
const resultProof = document.querySelector('#result-proof');
const winningWord = document.querySelector('#winning-word');

let queryVersion = 0;
let activeRun;
let fileVersion = 0;
let fileReading = false;
let uploadedDictionary;

function source() {
  return form.querySelector('input[name="source"]:checked').value;
}

function updateButtons() {
  searchButton.disabled = Boolean(activeRun) || (source() === 'file' && fileReading);
  searchButton.textContent = activeRun ? '조회 중…' : '승패 조회';
  cancelButton.disabled = !activeRun;
  resultPanel.setAttribute('aria-busy', String(Boolean(activeRun)));
}

function showStatus(title, message, state = 'idle') {
  resultPanel.dataset.state = state;
  resultTitle.textContent = title;
  resultMessage.textContent = message;
  progress.textContent = '';
  details.hidden = true;
  details.open = false;
  for (const element of [resultContext, resultProof, winningWord]) {
    element.textContent = '';
    element.hidden = true;
  }
}

function stopActive() {
  queryVersion += 1;
  const previous = activeRun;
  activeRun = undefined;
  previous?.controller.abort();
  loader.terminate();
  searcher.terminate();
  updateButtons();
  return Boolean(previous);
}

function invalidateQuery() {
  const stopped = stopActive();
  showStatus('조회 대기 중', stopped
    ? '입력이 변경되어 조회를 중단했습니다. 다시 조회하세요.'
    : '입력이 변경되었습니다. 다시 조회하세요.');
}

function updateManualSummary() {
  const { wordCount } = parseDictionary(words.value);
  document.querySelector('#manual-summary').textContent = `입력된 단어 ${wordCount.toLocaleString('ko-KR')}개`;
}

function validationError(message, input) {
  showStatus('입력을 확인해 주세요', message, 'error');
  input.focus();
}

function makeRule(content, changeFuncIdx) {
  return {
    id: 'example', metadata: { title: '음절 승패 조회', updatedAt: 0, color: '' },
    content: {
      wordRule: {
        words: { type: 'manual', option: { content } },
        regexFilter: '.*', addedWords: '', removedWords: '',
      },
      wordConnectionRule: { changeFuncIdx, rawHeadIdx: 1, headDir: 0, rawTailIdx: 1, tailDir: 1 },
      postprocessing: { manner: { type: 0 }, addedWords: '', removedWords: '' },
    },
  };
}

function isCurrent(run) {
  return activeRun === run && run.version === queryVersion;
}

function showInitialInfo(info, changeFuncIdx, usedCount = 0) {
  const description = describeSyllableInfo(info, changeFuncIdx);
  resultContext.textContent = `${description.ruleLabel} · 초기 분류: ${description.classificationLabel}${usedCount ? ` · 사용 단어 ${usedCount}개 제외` : ''}`;
  resultContext.hidden = false;
}

function showResult(result, syllable, elapsedMillis, info, changeFuncIdx, solver, usedCount) {
  showStatus(`‘${syllable}’로 시작할 차례인 사람은 ${result.isWin ? '필승' : '필패'}입니다.`,
    `두 사람이 최선의 수를 두면, 이 차례의 사람은 ${result.isWin ? '승리' : '패배'}합니다.`,
    result.isWin ? 'win' : 'lose');
  const explanation = explainSyllableResult(info, result, changeFuncIdx, solver.wordMap);
  showInitialInfo(info, changeFuncIdx, usedCount);
  resultProof.textContent = `판정 방식: ${explanation.methodLabel}`;
  resultProof.hidden = false;
  if (explanation.winningWord) {
    winningWord.textContent = `첫 승리 단어: ${explanation.winningWord}`;
    winningWord.hidden = false;
  }
  document.querySelector('#metrics').textContent =
    `전체 소요 ${(elapsedMillis / 1000).toFixed(2)}초 · 탐색 ${(result.duration / 1000).toFixed(2)}초 · 방문한 상태 ${result.visited.toLocaleString('ko-KR')}개`;
  document.querySelector('#path').textContent = result.optimalPath.length
    ? result.optimalPath.map(([head, tail]) => `${head} → ${tail}`).join('  /  ')
    : explanation.pathExplanation;
  details.hidden = false;
}

form.addEventListener('submit', async (event) => {
  event.preventDefault();
  if (activeRun || (source() === 'file' && fileReading)) return;
  const dictionary = source() === 'file' ? uploadedDictionary : parseDictionary(words.value);
  if (!dictionary) {
    validationError('조회할 사전 텍스트 파일을 선택해 주세요.', fileInput);
    return;
  }
  if (!dictionary.wordCount) {
    validationError('사전이 비어 있습니다. 한 개 이상의 단어를 넣어 주세요.', source() === 'file' ? fileInput : words);
    return;
  }
  const syllable = syllableInput.value.trim().normalize('NFC');
  if (Array.from(syllable).length !== 1) {
    validationError('조회할 음절을 한 글자로 입력해 주세요.', syllableInput);
    return;
  }
  const seconds = Number(timeoutInput.value);
  if (!Number.isInteger(seconds) || seconds < 1 || seconds > 600) {
    validationError('제한 시간은 1~600초 사이의 정수로 입력해 주세요.', timeoutInput);
    return;
  }
  const changeFuncIdx = Number(ruleInput.value);
  const workers = Number(workersInput.value);
  const history = [...new Set(historyInput.value.trim().split(/\s+/u).filter(Boolean))];
  const run = { version: ++queryVersion, controller: new AbortController() };
  activeRun = run;
  updateButtons();
  showStatus('사전 분석 중…', `${dictionary.wordCount.toLocaleString('ko-KR')}개의 입력 단어를 분석하고 있습니다.`, 'loading');
  const started = performance.now();
  const deadline = started + seconds * 1000;
  let lastProgress = 0;
  let info;
  try {
    let solver = await loader.callAndTerminate('getWcData', [makeRule(dictionary.content, changeFuncIdx), 0], seconds * 1000);
    if (!isCurrent(run)) return;
    if (history.length) {
      solver = await loader.callAndTerminate('withHistory', [solver, history, 0],
        Math.max(0, deadline - performance.now()));
      if (!isCurrent(run)) return;
    }
    info = await loader.callAndTerminate('getSyllableInfo', [solver, syllable, changeFuncIdx],
      Math.max(0, deadline - performance.now()));
    if (!isCurrent(run)) return;
    showStatus('승패 탐색 중…', `‘${syllable}’로 시작할 차례인 사람의 승패를 조회하고 있습니다.`, 'loading');
    showInitialInfo(info, changeFuncIdx, history.length);
    const result = await searcher.searchSyllable(solver.graphSolver, syllable, changeFuncIdx, undefined, {
      workers, timeoutMillis: Math.max(0, deadline - performance.now()), signal: run.controller.signal,
      onEvent(event) {
        if (!isCurrent(run) || event.action !== 'stack' || performance.now() - lastProgress < 150) return;
        lastProgress = performance.now();
        const branch = event.branchIndex === null ? '' : `분기 ${event.branchIndex + 1} · `;
        progress.textContent = `${branch}현재 ${event.payload.length.toLocaleString('ko-KR')}수까지 탐색 중`;
      },
    });
    if (isCurrent(run)) showResult(result, syllable, performance.now() - started, info, changeFuncIdx, solver, history.length);
  } catch (error) {
    if (!isCurrent(run)) return;
    if (error?.name === 'AbortError') {
      showStatus('조회 중단', '조회를 중단했습니다. 다시 조회할 수 있습니다.');
    } else if (error?.name === 'TimeoutError') {
      showStatus('미판정 — 제한 시간 초과', `${seconds}초 안에 승패를 확정하지 못했습니다. 제한 시간을 늘려 다시 조회하세요.`, 'error');
      if (info) showInitialInfo(info, changeFuncIdx, history.length);
    } else {
      showStatus('조회 실패', `승패를 조회할 수 없습니다. ${error?.message ?? String(error)}`, 'error');
    }
  } finally {
    if (isCurrent(run)) {
      activeRun = undefined;
      updateButtons();
    }
  }
});

cancelButton.addEventListener('click', () => {
  stopActive();
  showStatus('조회 중단', '조회를 중단했습니다. 다시 조회할 수 있습니다.');
});

words.addEventListener('input', () => {
  updateManualSummary();
  invalidateQuery();
});
for (const input of [syllableInput, timeoutInput, historyInput]) input.addEventListener('input', invalidateQuery);
for (const input of [ruleInput, workersInput]) input.addEventListener('change', invalidateQuery);
for (const radio of form.querySelectorAll('input[name="source"]')) {
  radio.addEventListener('change', () => {
    document.querySelector('#manual-panel').hidden = source() !== 'manual';
    document.querySelector('#file-panel').hidden = source() !== 'file';
    invalidateQuery();
  });
}

fileInput.addEventListener('change', async () => {
  const version = ++fileVersion;
  const file = fileInput.files[0];
  uploadedDictionary = undefined;
  fileReading = Boolean(file);
  invalidateQuery();
  if (!file) {
    fileSummary.textContent = '선택한 파일이 없습니다.';
    return;
  }
  fileSummary.textContent = `${file.name} · 파일을 읽는 중…`;
  try {
    const dictionary = await readDictionaryFile(file);
    if (version !== fileVersion) return;
    uploadedDictionary = dictionary;
    fileSummary.textContent = `${dictionary.name} · 입력된 단어 ${dictionary.wordCount.toLocaleString('ko-KR')}개${dictionary.wordCount ? '' : ' (빈 사전)'}`;
  } catch (error) {
    if (version !== fileVersion) return;
    fileSummary.textContent = error?.name === 'TypeError'
      ? `${file.name} · 텍스트를 해석할 수 없습니다. UTF-8 파일을 선택해 주세요.`
      : `${file.name} · 파일을 읽을 수 없습니다. 파일을 다시 선택해 주세요.`;
  } finally {
    if (version === fileVersion) {
      fileReading = false;
      updateButtons();
    }
  }
});

window.addEventListener('pagehide', () => {
  fileVersion += 1;
  if (fileReading) {
    fileReading = false;
    fileInput.value = '';
    fileSummary.textContent = '파일 읽기가 중단되었습니다. 파일을 다시 선택해 주세요.';
  }
  if (stopActive()) showStatus('조회 중단', '페이지 이동으로 조회를 중단했습니다. 다시 조회할 수 있습니다.');
});

updateManualSummary();
updateButtons();
