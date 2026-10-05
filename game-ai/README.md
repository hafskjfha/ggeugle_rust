# 끝말잇기 게임 AI

기존 끝말잇기 웹 서비스에서 게임 인공지능과 분석 엔진만 추출한 독립 TypeScript 패키지입니다. Node.js 22 이상에서 실행합니다.

원본 프로젝트: [singrum/ggeugle](https://github.com/singrum/ggeugle).

## 설치 및 실행

이 폴더를 다른 프로젝트로 복사해도 독립적으로 사용할 수 있습니다.

```bash
cd game-ai
npm ci
npm run typecheck
npm test
npm run example
```

`npm run build`는 `src/`를 실행 가능한 ESM 코드와 타입 선언이 있는 `dist/`로 컴파일합니다. 예제는 수동 단어 목록을 사용하므로 외부 사전 다운로드 없이 실행됩니다.

## 폴더 구성

```text
game-ai/
├── src/
│   ├── index.ts           # 공개 API
│   ├── ai/                # 난이도별 수 선택, 필승 탐색, Node 워커
│   ├── wordchain/
│   │   ├── classes/       # 엣지 자료구조
│   │   ├── graph/         # 승패 분류, 가지치기, 최적 수와 전략 트리
│   │   ├── rule/          # 두음 법칙 등 음절 변환
│   │   └── word/          # 사전 로드, 단어 맵과 단어 분석
│   ├── constants/         # 사전 정보, 기본 규칙, 탐색 우선순위
│   ├── types/             # 규칙 및 분석 결과 타입
│   └── utils.ts           # 엔진이 사용하는 순수 함수
├── examples/play.mjs      # AI의 다음 수 선택 예제
├── tests/                 # 엔진과 실제 워커 검증
└── docs/                  # 기존 그래프 모델 및 알고리즘 설명
```

## 사용법

[`examples/play.mjs`](examples/play.mjs)에 규칙 설정, 사전 생성, AI 실행의 전체 예제가 있습니다. 빌드 후 다음처럼 가져옵니다.

```js
import { engineFunctions, GameWorkerRunner, isGameEnd } from "./dist/index.js";

const flow = 0;
const solver = await engineFunctions.getWcData(rule, flow);
const history = ["사과"];
const precedence = { rule: 0, maps: { edge: {}, node: {} } };

if (!isGameEnd(solver, history, false)) {
  const ai = new GameWorkerRunner(
    solver, 2, 1, false, history, precedence,
    (event) => { if (event.action === "move") console.log(event.payload); },
    "game-1",
  );
  try { await ai.run(flow); }
  finally { ai.terminate(); }
}
```

- 난이도 `0`: 가능한 단어 중 무작위 선택.
- 난이도 `1`: 승패 분류와 얕은 최적 수를 사용하고 루트에서는 무작위 선택.
- 난이도 `2`: 후보별 필승 탐색을 실행하며 시간제한을 넘긴 탐색은 워커를 종료.
- `calculatingDuration`: 후보 하나의 탐색 제한 시간(초). 여러 후보를 탐색하면 총 실행 시간은 더 길어질 수 있습니다.
- `flow`: 기존 승패 전파·돌림 단어 제거 순서(`0` 또는 `1`). 사전 생성과 `run()`에서 같은 값을 사용합니다.
- `stealable`: 상대의 첫 단어를 뺏을 수 있는 규칙.
- 콜백: `debug`, `move`, `computerWin`, `messageEnd` 이벤트. 선택된 단어는 `move.payload`로 전달됩니다.

단어를 둘 수 없는 상태에서는 `isGameEnd()`로 종료를 확인한 뒤 AI 실행을 생략합니다. 매 턴 현재 단어 이력을 전달해 새 `GameWorkerRunner`를 만듭니다.

`WordRule.words.type = "manual"`이면 제공한 단어만 사용합니다. `"selected"`이면 기존 사전 URL에서 단어를 가져오므로 인터넷 연결이 필요합니다. React, Vite, Comlink, 웹 UI, IndexedDB는 실행에 필요하지 않습니다.

`engineFunctions`에는 사전/solver 생성, 이력 반영, 승패 탐색, 전략 탐색 콜백 API가 있습니다. `WorkerRunner`를 직접 사용할 때는 반환 객체를 `WordSolver.fromObj()` 또는 `GraphSolver.fromObj()`로 복원해야 클래스 메서드를 사용할 수 있습니다. `GameWorkerRunner`는 내부적으로 이를 처리합니다.

런타임 의존성은 `denque`, `lodash-es`, `hangul-js`, `es-hangul` 네 개입니다. 기존 두음 법칙, 그래프 알고리즘, 기본 규칙과 우선순위 데이터는 유지했습니다.
