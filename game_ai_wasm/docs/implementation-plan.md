# game_ai_wasm Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans and superpowers:test-driven-development. Independent Rust and TypeScript implementation may run in parallel against the interfaces below.

**Goal:** 기존 Rust 엔진을 재사용하는 WASM 패키지와 병렬 Web Worker 탐색을 제공한다.

**Architecture:** Rust path dependency + wasm-bindgen adapter + typed browser facade. Web Workers own independent WASM instances and run existing Rust DFS branches.

**Tech Stack:** Rust 2024, wasm-bindgen, serde-wasm-bindgen, web-time, TypeScript, Node test runner.

**Spec:** `game_ai_wasm/docs/design.md`

## Global Constraints

- 별도 `game_ai_wasm/` 패키지; `game_ai_rust` 경로 의존성; 기존 API 유지.
- 새 패키지의 시간 단위는 밀리초; Map은 일반 객체로 직렬화.
- WASM 인스턴스는 워커마다 독립; 공유 메모리 및 Rust OS 스레드 없음.

## Review Focus

- 응수 승패의 반전과 입력 이동의 중복 소비: Rust parity tests.
- 완료 순서와 동률의 최적 경로 변화: out-of-order worker and Rust tests.
- 취소 후 이벤트 또는 수 선택이 다시 전달됨: worker cancellation tests.
- 초기화 오류 및 timeout 후 워커/타이머 누수: worker lifecycle tests.
- WASM에서 시계 및 난수 호출 실패: actual WASM search and chooseMove integration tests.

### Task 1: Rust 코어와 WASM 바인딩

**Files:** `game_ai_rust/src/ai.rs`, `game_ai_rust/Cargo.toml`, `game_ai_rust/tests/parallel.rs`; `game_ai_wasm/Cargo.toml`, `game_ai_wasm/src/lib.rs`.

**Interfaces:** `RootSearchPlan { graph, movement, moves, result }`; `RootBranchResult { result: SearchResult, outcome: NodeType }`; `prepare_root_search(graph, movement, prec, limit)`; `search_root_branch(graph, movement, prec, limit, callback)`; `finish_root_search(plan, results: &[Option<RootBranchResult>], duration: f64)`.
WASM exports: `getWcData(rule, flow)`, `updateSolver(graphs, moves, flow)`, `getGraph(graphs)`, `getNextWords(solver, history)`, `afterHistory(solver, history, flow)`, `chooseMove(solver, history, options, callback?)`, `isGameEnd(solver, history, stealable)`, `searchIsWin(graph, move, prec, timeoutMillis?)`, `startStreamingSingleThreadSearch(graph, move, prec, timeoutMillis, callback)`, `prepareRootSearch(graph, move, prec, timeoutMillis?)`, `searchRootBranch(graph, move, prec, timeoutMillis?, callback?)`, `finishRootSearch(plan, results, duration)`, `dictionaryUrls(option)`, `getPresets()`.

- [x] 먼저 단일 탐색과 분기 병합의 승패 및 경로 일치 테스트를 작성하고 실패 확인.
- [x] 코어 준비/병합 구현, WASM 시계와 난수 호환성 추가.
- [x] 바인딩, 입력 검증, JS 예외 변환 구현.
- [x] `cargo test`, WASM target build, 실제 WASM integration 검증.

### Task 2: Web Worker와 병렬 탐색

**Files:** `game_ai_wasm/ts/worker-runner.ts`, `parallel-search.ts`, `worker-entry.ts`, `tests/workers.test.mjs`.

**Interfaces:** `WorkerRunner.callAndTerminate(method,args,timeoutMillis?,onEvent?)`, `terminate()`; `ParallelSearchRunner.search(graph,move,prec?,options?)`, `terminate()`; options `{ workers?, timeoutMillis?, signal?, onEvent? }`. Worker factory receives worker-entry URL and module options, allowing browser bundler overrides and Node test adapters. Worker methods dispatch the exports from Task 1 via the facade from Task 3.

- [x] 먼저 병렬 실행/순서 보존/취소/timeout/오류 테스트를 작성하고 실패 확인.
- [x] 제한된 동시 실행, 순서 있는 결과 병합, 종료와 오류 정리 구현.
- [x] 실제 WASM worker tests와 typecheck 확인.

### Task 3: 브라우저 API, 빌드, 예제

**Files:** `game_ai_wasm/ts/types.ts`, `engine.ts`, `index.ts`, `package.json`, `tsconfig.json`, `scripts/`, `tests/wasm.test.mjs`, `examples/`, `README.md`; root README.

**Interfaces:** `createEngineFunctions(wasmInput?)` resolves an initialized `EngineFunctions` with Task 1 exports. `DEFAULT_PRECEDENCE = { rule: 0, maps: { edge: {}, node: {} } }`; getWcData asynchronously resolves selected dictionary sources using WASM dictionaryUrls and fetch, preserving filtering in Rust. Worker entry can pass an absolute wasmUrl to initialization.

- [x] 실제 WASM의 solver/탐색/난수/오류 및 로딩 테스트를 먼저 작성하고 실패 확인.
- [x] Cargo/WASM/TS 빌드 스크립트와 typed facade 구현.
- [x] 브라우저 예제와 한국어 사용 설명 작성.
- [x] 기존 Rust/TS 전체 회귀 및 새 패키지 전체 검증 후 독립 리뷰.

## 검증 기록

기존 TS 23개, Rust 코어 50개, WASM 바인딩 네이티브 테스트 2개, JavaScript/WASM/워커 39개를 검증했다. 실제 브라우저의 두 워커 병렬 탐색도 확인했다. 독립 리뷰에서 발견한 WorkerRunner의 지연된 timeout callback 경계를 재현 테스트 후 수정했다.
