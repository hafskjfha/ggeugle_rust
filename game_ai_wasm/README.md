# 끝말잇기 WASM 엔진

`game_ai_rust`를 경로 의존성으로 사용하는 브라우저용 WASM 패키지입니다. 그래프 구성, 승패 판별, 가지치기, DFS, 단어 선택은 기존 Rust 엔진이 수행합니다. TypeScript는 초기화와 사전 fetch, Web Worker 실행을 담당합니다.

## 빌드와 실행

Rust와 Node.js 22 이상이 필요합니다. 최초 한 번 WASM 타깃과 바인딩 도구를 설치합니다. `wasm-bindgen-cli` 버전은 `Cargo.toml`에 고정한 크레이트 버전과 일치해야 합니다.

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
cd game_ai_wasm
npm ci
npm run build
npm test
npm run example
```

예제는 `http://127.0.0.1:4173`에서 열립니다. `npm run build`는 Rust를 release WASM으로 빌드한 뒤 JS glue와 TypeScript 선언을 생성합니다. 빌드 결과는 `pkg/`, `dist/`에 있고 버전 관리에서 제외됩니다. `.tools/bin/wasm-bindgen`이 있으면 자동으로 사용하며, `WASM_BINDGEN` 환경 변수로 도구 경로를 지정할 수도 있습니다.

`wasm-pack`을 이미 사용한다면 다음 방식도 가능합니다.

```bash
wasm-pack build --target web --out-dir pkg
npm run build:ts
```

## 기본 API

```js
import { createEngineFunctions } from './game_ai_wasm/dist/index.js';

const engine = await createEngineFunctions();
const rule = {
  id: 'example', metadata: { title: '예제', updatedAt: 0, color: '' },
  content: {
    wordRule: {
      words: { type: 'manual', option: { content: '가가 가나 나가 나다 다가' } },
      regexFilter: '.*', addedWords: '', removedWords: '',
    },
    wordConnectionRule: {
      changeFuncIdx: 0, rawHeadIdx: 1, headDir: 0, rawTailIdx: 1, tailDir: 1,
    },
    postprocessing: { manner: { type: 0 }, addedWords: '', removedWords: '' },
  },
};
const solver = await engine.getWcData(rule, 0);
console.log(engine.getNextWords(solver, ['가나']));
console.log(engine.chooseMove(solver, ['가나'], { difficulty: 2 }));

// 모든 분류 파티션을 Rust에서 합친 그래프를 탐색합니다.
const graph = engine.getGraph(solver.graphSolver.graphs);
const result = engine.searchIsWin(graph, ['나', '가']);
console.log(result.isWin, result.optimalPath);

// 기보 이후 상태를 탐색할 때는 먼저 사용한 단어를 반영합니다.
const remaining = engine.afterHistory(solver, ['가나']);
const remainingGraph = engine.getGraph(remaining.graphs);
```

`createEngineFunctions()`는 기본적으로 glue 파일 옆의 `.wasm`을 로드합니다. 별도 URL 또는 `Uint8Array`/`WebAssembly.Module`도 전달할 수 있습니다. 같은 JavaScript realm에서는 초기화된 인스턴스를 재사용합니다. 직접 호출하는 분석과 탐색은 동기적으로 실행되므로 큰 사전은 워커 API를 사용하세요.

반환하는 solver와 그래프는 일반 객체 snapshot입니다. 기존 TypeScript 클래스의 메서드는 포함하지 않으며, `engine.getNextWords`, `engine.afterHistory` 같은 함수에 snapshot을 전달합니다. Map도 일반 객체로 전달되므로 워커 사이에 structured clone이 가능합니다.

## Web Worker 호출

```js
import { WorkerRunner } from './game_ai_wasm/dist/index.js';

const runner = new WorkerRunner();
const solver = await runner.callAndTerminate('getWcData', [rule, 0], 30000);
const graph = await runner.callAndTerminate('getGraph', [solver.graphSolver.graphs]);
const result = await runner.callAndTerminate('searchIsWin', [graph, ['나', '가']], 10000);

// AI 수 선택도 Rust 코드를 워커에서 실행할 수 있습니다.
const word = await runner.callAndTerminate('chooseMove',
  [solver, ['가나'], { difficulty: 2, calculatingDuration: 1000 }], 10000,
  (event) => console.log(event));

runner.terminate();
```

한 호출마다 모듈 Web Worker를 만들고 완료 후 종료합니다. 같은 runner에 새 호출을 시작하면 이전 호출은 `AbortError`로 취소됩니다. timeout은 초기화 시간까지 포함하며 `TimeoutError`를 반환합니다. WASM 오류도 Promise rejection으로 전달됩니다. 진행 callback은 마지막 인자로 전달하고, `args`에는 함수 값을 넣지 않습니다.

## 병렬 탐색

```js
import { ParallelSearchRunner } from './game_ai_wasm/dist/index.js';

const searcher = new ParallelSearchRunner();
const controller = new AbortController();
const result = await searcher.search(graph, ['나', '가'], undefined, {
  workers: 4,
  timeoutMillis: 10000,
  signal: controller.signal,
  onEvent(event) {
    if (event.action === 'stack') console.log(event.branchIndex, event.payload);
  },
});
// 중단할 때 controller.abort() 또는 searcher.terminate()를 호출합니다.
```

입력 이동을 적용한 뒤 상대의 응수들을 여러 워커에 배분합니다. 각 분기는 기존 Rust DFS를 실행하며, 준비와 결과 병합도 Rust에서 수행합니다. 응수 결과가 도착한 순서와 관계없이 원래 탐색 순서로 병합하여 단일 탐색의 승패, 경로와 방문 수를 유지합니다. 승패를 결정하는 순서상의 분기가 확인되면 나머지 작업을 종료합니다.

`workers`는 1~64의 정수이며 기본값은 브라우저 CPU 수를 고려해 최대 4개입니다. timeout은 준비, WASM 초기화, 대기, 탐색, 병합을 포함합니다. timeout은 탐색 실패로 반환하며 승리나 패배로 추정하지 않습니다. 취소 시 진행 이벤트도 더 이상 전달하지 않습니다.

각 워커가 독립 WASM 메모리를 가지므로 Rust 스레드, `SharedArrayBuffer`, COOP/COEP 헤더가 필요하지 않습니다. 워커 생성과 그래프 복사 비용이 있어 작은 탐색은 직접 호출보다 느릴 수 있습니다. 현재 워커는 작업마다 생성되며 상주 워커 풀은 사용하지 않습니다.

## 사전, 시간 단위, 배포

- `manual` 사전은 Rust의 기존 필터와 전처리를 그대로 적용합니다.
- `selected` 사전은 기존 Rust URL 목록을 JavaScript에서 fetch한 뒤 Rust에 전달합니다. 서버가 CORS를 허용해야 하며, `getWcData(rule, flow, { dictionaryUrl: url => proxyUrl })`로 같은 출처의 프록시를 지정할 수 있습니다. 이 함수 옵션은 직접 호출에서 사용하세요.
- 브라우저에서는 Rust의 파일 경로와 `reqwest`를 사용하지 않습니다. `File.text()` 결과를 `manual`의 `content`로 넣으세요.
- 새 패키지의 `duration`, `timeoutMillis`, `calculatingDuration`은 모두 **밀리초**입니다. 기존 TS/Rust 스트리밍 API의 초 단위와 차이가 있습니다.
- `SearchResult.isWin`은 입력 이동을 한 플레이어의 승리 여부입니다. `visited`는 단일 탐색과 같은 결정에 사용한 방문 수이며, 취소된 추가 분기의 실제 CPU 작업량은 포함하지 않습니다.
- 정적 배포 시 `dist/`와 `pkg/`를 형제 폴더로 유지하고 HTTP(S)로 제공하세요. `.wasm` MIME 타입은 `application/wasm`입니다.
- 번들러에서는 `workerFactory`와 `wasmUrl`로 리소스 위치를 지정할 수 있습니다.

```js
const searcher = new ParallelSearchRunner({
  workerFactory: () => new Worker(new URL('./game_ai_wasm/dist/worker-entry.js', import.meta.url),
    { type: 'module' }),
  wasmUrl: new URL('./game_ai_wasm/pkg/game_ai_wasm_bg.wasm', import.meta.url),
});
```

추가 API로 `updateSolver`, `isGameEnd`, `startStreamingSingleThreadSearch`, `startStreamingCriticalWordsInfo`, `dictionaryUrls`, `getPresets`를 제공합니다. 분기 준비/실행/병합을 직접 제어하려면 `prepareRootSearch`, `searchRootBranch`, `finishRootSearch`를 사용하세요.

## 검증

```bash
cargo test --manifest-path ../game_ai_rust/Cargo.toml --locked
cargo test --locked
npm test
npm run typecheck
```

Rust 테스트는 작은 그래프의 완전 열거와 기존 TypeScript golden fixture로 단일/분기 탐색의 승패와 경로를 비교합니다. JavaScript 테스트는 실제 WASM 초기화·시계·난수·사전 fetch, 실제 worker_threads를 브라우저 인터페이스로 연결한 탐색, 워커 취소·timeout·오류·순서 보존을 확인합니다. `examples/index.html`은 실제 브라우저 Web Worker 사용 예제입니다.
