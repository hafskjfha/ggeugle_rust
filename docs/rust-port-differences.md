# Rust 포팅의 유지 사항과 변경점

분석 기준: 2026-10-06, 저장소 커밋 `5dab9b3`의 `game-ai/`와 `game_ai_rust/` 소스. 원본 방식은 [원본 AI 문서](original-ai.md)에 설명했다. 이 문서는 **현재 소스의 차이**를 비교하며, 변경 이유나 추가 시점을 코드만으로 단정하지 않는다. `game_ai_wasm/`은 별도 어댑터로 필요한 경계를 함께 표시한다.

## 1. 핵심 알고리즘은 유지됐다

Rust는 원본의 그래프 축소와 승패 탐색을 포팅했다. 신경망이나 새로운 탐색 알고리즘으로 바꾼 구현은 아니다.

| 유지한 항목 | 구현·검증 근거 |
| --- | --- |
| 위치 0/1 이분 유향 그래프, 단어 간선 개수 | TS `BipartiteDiGraph` ↔ Rust `graph::BipartiteDiGraph` |
| 승패 전파, 강제 루프의 홀짝, 두 단어 돌림 쌍 | TS `classify.ts` ↔ Rust `classify.rs`·`pairs.rs` |
| `flow` 0/1의 처리 순서와 최대 200회 반복 | 양쪽 `classify()` |
| `removed`·`winlose`·`route` 파티션과 단어 인덱스 오프셋 | TS `graph-partitions.ts` ↔ Rust `partitions.rs` |
| 정적 깊이, 최적 수, SCC 분석과 내보내기 | TS `GraphSolver`·`WordSolver` ↔ Rust `solver.rs`·`words.rs` |
| 11개 음절 변환, 기본 규칙과 탐색 우선순위 데이터 | TS 규칙·상수 ↔ Rust `rules.rs`·`presets.rs` |
| 중복 이력은 한 번만 소비, 같은 음절 쌍의 다른 단어는 개별 소비 | 양쪽 이력 처리와 Rust 회귀 테스트 |
| 난이도 0/1/2의 기본 선택 정책과 첫 단어 뺏기 | TS `GameWorkerRunner` ↔ Rust `ai::choose_move_with_callback()` |
| 간선 → 끝 음절 → 휴리스틱의 탐색 순서와 승리 발견 시 조기 종료 | 양쪽 후보 비교·DFS |

Rust는 `IndexMap`·`IndexSet`으로 삽입 순서를 보존한다. 사전 순서, LIFO 전파, 파티션 병합 순서와 동률 선택이 실제 결과에 영향을 주므로 단순한 성능 최적화에 그치지 않는 호환성 조건이다. 전체 그래프 합치기와 이력 재분류용 합치기는 사용하는 순서도 구분한다.

원본의 특이한 동작도 일부 유지했다. 규칙 7/10의 역변환 특성, 단어 카드의 `connected` 표시와 그래프 연결 판정의 반전, 어려움 첫 수에서 승패보다 **측정 탐색시간**으로 후보를 고르는 정책이 이에 해당한다. 따라서 Rust로 옮겼다는 이유만으로 이런 정책이 교정되었다고 보면 안 된다.

패배 최적 수의 기존 위치 0 분기가 위치 1 후보 이름을 위치 0 깊이 맵에서 비교하는 특성도 유지했다. 여기서의 호환성은 원본 선택 동작의 유지이며, 모든 경우에 실제 최장 수순을 보장한다는 뜻은 아니다.

돌림 쌍 후보를 만드는 `getSingleInEdges()`가 변환 간선과 반대 순서의 `(시작 음절, 선행 끝 음절)` 튜플을 반환하는 특성도 Rust `get_single_in_edges()`에서 유지한다. 진출 차수 1 후보는 정방향 튜플이므로, 이 후보 합집합을 모두 같은 방향의 변환 간선으로 해석하면 안 된다.

근거: [graph.rs](../game_ai_rust/src/graph.rs), [classify.rs](../game_ai_rust/src/classify.rs), [partitions.rs](../game_ai_rust/src/partitions.rs), [solver.rs](../game_ai_rust/src/solver.rs), [rules.rs](../game_ai_rust/src/rules.rs), [engine 테스트](../game_ai_rust/tests/engine.rs), [solver 테스트](../game_ai_rust/tests/solver.rs).

## 2. 실행과 탐색 구현의 차이

| 항목 | 원본 TypeScript | Rust | 사용 시 영향 |
| --- | --- | --- | --- |
| 실행 모델 | Promise와 Node 워커 호출 | 호출한 스레드에서 동기 실행 | 큰 분석·탐색은 호출 스레드를 점유함 |
| DFS | `common.ts::isWin()` 재귀 | `ai.rs::search_next_turn()`의 `Vec<Frame>` 반복 | 깊은 수순이 프로세스 호출 스택에 의존하지 않음 |
| SCC 계산 | Tarjan 계열 재귀 DFS | 명시적 프레임으로 같은 SCC 탐색 | SCC 계산도 호출 스택 의존을 줄임 |
| 전략 트리 전개 | 승리·패배 전개 함수의 상호 재귀 | 반복문과 `Option`을 이용한 전개 | 긴 유한 전략을 반복문으로 펼침; 무한 콜백의 종료를 보장하지는 않음 |
| 시간제한 | 워커 외부 타이머로 워커 강제 종료 | DFS·전처리 경계에서 `Instant`와 `Duration`을 확인 | Rust는 개별 전처리나 콜백 도중에 강제 중단되지 않음 |
| 오류 | 예외·Promise rejection; 어려움 AI는 탐색 오류를 모두 미판정으로 처리 | `Result`·`Error`; AI는 `Error::Timeout`에만 후보 대체 적용 | 시간 초과 외의 오류는 호출자에게 전달됨 |
| 취소 | 워커 종료 또는 다음 호출로 이전 호출 취소 | 코어 탐색 함수에는 취소 신호 인자가 없음 | 병렬 실행과 강제 취소는 외부 실행 계층이 담당해야 함 |

Rust의 후보별 제한도 원본처럼 전체 턴 제한이 아니다. `Some(Duration::ZERO)`는 탐색 시작 단계에서 `Error::Timeout`, `None`은 제한 없음이다. 원본 직접 탐색 함수에는 이런 deadline 인자가 없었다.

Rust의 `Error::Cancelled` 열거값이 존재하더라도 코어 DFS에 협력적 취소 API가 구현되었다는 뜻은 아니다. 현재 탐색 함수가 확인하는 것은 마감시간이다.

원본 직접 `searchIsWin()`의 반환값은 `isWin`과 `duration`뿐이고, 탐색 경로는 스트리밍 완료 이벤트에서 제공한다. Rust 직접 `search_is_win()`은 같은 승패 관점에 `optimal_path`와 `visited`를 추가한 `SearchResult`를 반환한다. 탐색 스택 이벤트는 양쪽 모두 첫 이벤트를 즉시 전달하고 이후 최대 1초에 한 번 복사본을 전달한다.

근거: [원본 common.ts](../game-ai/src/ai/common.ts), [원본 worker-runner.ts](../game-ai/src/ai/worker-runner.ts), [Rust ai.rs](../game_ai_rust/src/ai.rs)의 `Frame`·`check_deadline()`·`search_next_turn()`·`choose_move_with_callback()`, [graph.rs](../game_ai_rust/src/graph.rs)의 `get_scc()`, [strategy.rs](../game_ai_rust/src/strategy.rs).

### 시간 단위

| API·설정 | 원본 | Rust 코어 |
| --- | --- | --- |
| AI 후보별 제한 | `calculatingDuration`: 초 | `AiOptions.calculating_duration`: `Duration` |
| 직접 탐색 결과 | `searchIsWin().duration`: 밀리초 | `search_is_win().duration`: 밀리초 |
| 스트리밍 완료 이벤트 | `done.duration`: 반올림한 초 | `SearchEvent::Done.duration`: 반올림한 초 |
| 스트리밍 함수 자체 반환 | 완료 이벤트로 결과 전달 | `Result<SearchResult>`를 추가 반환하며 그 값의 `duration`은 밀리초 |

Rust의 스트리밍 함수는 **콜백에 전달한 완료 결과와 함수 반환값의 시간 단위가 다르다**. 결과를 저장할 때 단위를 함께 관리해야 한다. 별도 WASM 어댑터는 외부 API의 시간 값을 밀리초로 통일하므로 코어와 구분한다.

## 3. 수 선택과 종료 처리의 차이

### 종료 상태를 반환값으로 표현

원본 `GameWorkerRunner.run()`은 선택 단어를 이벤트로 전달하며, 일반적인 종료 상태는 호출자가 먼저 `isGameEnd()`로 확인한다. 수 선택 코드에는 후보가 존재한다는 non-null 가정이 있다.

Rust `choose_move()`는 `Result<Option<String>>`을 반환한다. 일반 응수도 뺏기도 불가능하면 `Ok(None)`이며, 콜백 버전은 `MessageEnd`를 전달한다. 선택 정책으로 단어를 얻지 못한 경우에는 첫 합법 단어, 이어서 가능한 첫 단어 뺏기를 대체 선택으로 사용한다.

### 쉬움 난이도의 뺏기 경계 상황

이력이 첫 단어 하나이고 `stealable = true`인데 일반 응수가 없으면, 원본 `isGameEnd()`는 종료가 아니라고 판정한다. 하지만 난이도 0의 `getRandomNextWord()`는 빈 후보에서 음절 쌍을 구조 분해하므로 실패할 수 있다.

Rust는 이 상황에서 첫 단어를 반환한다. 예를 들어 사전이 `사과` 하나이고 이력이 `["사과"]`라면 난이도 0에서도 `사과`를 뺏을 수 있다. 이는 원본의 정상적인 무작위 선택을 바꾼 것이 아니라, 빈 일반 후보에서도 뺏기 규칙을 실행하도록 처리한 차이다.

### 로그·반환 정보

`Move`, `ComputerWin`, `MessageEnd`의 게임 이벤트 역할은 유지한다. `Debug` 문자열은 원본의 상세 Markdown 로그를 그대로 복제하지 않고 간략하게 전달한다. 특히 원본 어려움 첫 수의 무조건적인 `필패 확정` 로그는 Rust에서 출력하지 않는다. 디버그 텍스트의 완전한 문자열 호환성은 없다.

난수 라이브러리가 `lodash`/JavaScript에서 `rand`로 바뀌었고, 실제 탐색시간과 워커 비용도 다르다. 무작위 후보나 최대 탐색시간으로 선택하는 수는 동일한 입력에서도 양쪽 결과가 달라질 수 있다. 원본과 같은 승패 분석 정책을 유지한다는 사실이 매번 같은 단어를 고른다는 보장은 아니다.

근거: [원본 game-worker-runner.ts](../game-ai/src/ai/game-worker-runner.ts)의 `run()`·`getRandomNextWord()`·`takeDragingWord()`·`isGameEnd()`, [Rust ai.rs](../game_ai_rust/src/ai.rs)의 `choose_move_with_callback()`, [engine 테스트](../game_ai_rust/tests/engine.rs).

## 4. 입력, 사전, 직렬화의 차이

| 항목 | 차이 | 영향 |
| --- | --- | --- |
| 문자열 위치 | TS는 `String.at()`의 UTF-16 코드 단위, Rust는 `chars()`의 Unicode 스칼라 값 | BMP 한글은 같지만 이모지 등 보조 평면 문자가 있는 단어의 위치는 달라질 수 있음 |
| 잘못된 인덱스·설정 | Rust는 단어 위치 범위, 연결 인덱스·방향, 공개 입력의 규칙 번호·flow 등을 명시적으로 검사 | 오류가 `InvalidInput`으로 전달됨; 모든 내부 메서드가 동일한 검증을 수행하는 것은 아님 |
| 비한글 문자 변환 | Rust는 한글·호환 자모가 아닌 문자를 그대로 통과시킴 | 원본의 범위 검사 없는 한글 분해 산술이 만드는 비한글 변환을 재현하지 않음 |
| 정규식 | JS `RegExp`에서 `fancy-regex`로 변경; 양쪽 모두 `^...$` 필터 적용 | lookaround/backreference를 지원해도 JavaScript의 모든 문법과 문자 의미가 같지는 않음 |
| 로컬 파일 사전 | Rust에 `WordSource::File` 추가 | UTF-8 파일을 줄 단위로 읽으며 시작 BOM을 제거함 |
| 선택 사전 다운로드 | TS는 비동기 `fetch`·복수 URL 동시 다운로드, Rust는 선택 기능 `remote`의 동기 HTTP | Rust 기본 빌드에서 `selected`는 오류; 활성화하면 다운로드가 호출 스레드를 점유함 |
| 사전 순서 | TS `Map`·`Set`에서 Rust `IndexMap`·`IndexSet`으로 대응 | 중복의 첫 등장 순서와 음절 쌍의 단어 순서를 유지함 |
| 공개 API 이름 | `camelCase` 함수에서 Rust `snake_case` 함수로 변경 | 호출 코드는 변경 필요 |
| 규칙·solver 직렬화 | Rust serde는 규칙 등 주요 구조에 원본 camelCase 필드명을 대응 | JSON에서 Rust 구조로 복원하면 메서드 사용 가능; 원본 worker snapshot은 `fromObj()`로 클래스 복원 필요 |

정규식 차이는 지원하지 않는 문법의 오류에만 한정되지 않는다. 같은 문법이 양쪽에서 컴파일되어도 Unicode 문자 클래스 같은 세부 의미가 달라질 수 있으므로, 비한글·복잡한 정규식을 사용하는 사전은 별도 확인이 필요하다.

표시용 정렬도 일부 다르다. 원본 카드·돌림 단어 정렬 중 일부는 `localeCompare()`를 쓰고 Rust는 문자열·튜플의 `cmp()`를 사용한다. 현재 샘플의 결과는 일치하지만, 언어·문자 종류를 넓힌 입력의 표시 순서까지 같다고 보장하지 않는다.

serde 지원은 모든 내부 값과 전략 트리 JSON이 원본 객체 모양 그대로 교환된다는 보장은 아니다. 대표적인 전략 트리 형식 차이는 다음과 같다.

```text
원본 승리 단계: { isWin: true, word: ["자두"], depth: 0, ... }
Rust 승리 단계: { isWin: true, words: [["자두"]], depth: 0 }

원본·Rust 패배 단계: words에 응수별 단어 배열을 넣음
```

Rust `TreeData.words`는 항상 `Vec<Vec<String>>`이다. 일반 `StrategyState`도 원본의 `isWin` 구분 대신 `kind: "winning" | "losing"` 태그를 사용한다. 기존 소비자가 승리 단계의 `word`를 읽었다면 변환 계층이 필요하다.

근거: [rules.rs](../game_ai_rust/src/rules.rs)의 `get_idx()`·`get_head_tail()`·`load_words()`·`load_selected_words()`, [Cargo.toml](../game_ai_rust/Cargo.toml), [strategy.rs](../game_ai_rust/src/strategy.rs)의 `StrategyState`·`TreeData`, [원본 strategy-tree.ts](../game-ai/src/wordchain/graph/strategy-tree.ts), [rules 테스트](../game_ai_rust/tests/rules.rs).

## 5. 현재 Rust 구현에 추가된 API

### 외부 병렬 실행을 위한 분기 API

Rust 코어는 스레드나 워커를 직접 생성하지 않는다. 다만 현재 소스에는 탐색을 외부에서 분산 실행할 수 있게 나누는 API가 있다.

| API | 역할 |
| --- | --- |
| `prepare_root_search()` | 입력 수를 한 번 소비하고 그래프를 축소한 뒤, 정렬된 상대 응수 목록 또는 확정 결과 생성 |
| `search_root_branch()` | 독립 응수 분기 하나를 기존 DFS로 탐색 |
| `finish_root_search()` | 원래 후보 순서의 완료 결과를 병합 |

병합은 먼저 도착한 승리 결과를 무조건 사용하는 방식이 아니다. 앞쪽 후보들이 완료된 **순서상의 접두 구간**이 필요하고, 승패를 결정할 첫 응수 뒤의 결과는 생략할 수 있다. 단일 탐색의 승패·경로·방문 수를 보존하기 위한 처리다. `visited`는 병합 결정에 사용한 방문 수이며, 외부 병렬 실행에서 취소된 추가 분기의 실제 작업량을 모두 세는 값은 아니다.

`game_ai_wasm/ts/parallel-search.ts`의 `ParallelSearchRunner`가 이 API를 호출하여 독립 WASM 인스턴스를 가진 Web Worker들에 분기를 배분한다. 따라서 **코어 자체는 동기 실행**, **분산 조정은 WASM의 TypeScript 어댑터**라는 두 층으로 이해하면 된다. Rust README와 초기 `port-plan.md`의 워커 제외 설명만으로 현재 분기 API까지 없다고 판단하면 안 된다.

근거: [ai.rs](../game_ai_rust/src/ai.rs)의 분기 API, [parallel 테스트](../game_ai_rust/tests/parallel.rs), [parallel-search.ts](../game_ai_wasm/ts/parallel-search.ts).

### 단어를 두지 않고 음절에서 시작하는 조회

원본 공개 탐색은 입력 단어 이동을 기준으로 한다. Rust에는 별도로 다음 기능이 추가되어 있다.

- `WordSolver::get_syllable_info()`: 정적 음절 분류와 그 분류에서 확인한 첫 승리 음절 쌍·단어 목록 조회.
- `search_syllable()`: 해당 음절에서 시작할 차례인 사람의 승패 탐색.
- `search_syllable_with_witness()`: 탐색 결과와 독립적으로 승리가 확인된 첫 수 `winning_move` 반환.
- `prepare_syllable_search()`·`finish_syllable_search()`: 같은 조회를 외부 분기 탐색과 연결.

| 관점 | `is_win`의 의미 |
| --- | --- |
| 기존 `search_is_win(graph, movement, ...)` | **입력 이동을 둔 사람**이 이기는가 |
| 추가 `search_syllable(solver, syllable, ...)` | **조회 음절에서 지금 시작할 사람**이 이기는가 |

음절 조회는 실제 사전 단어를 소비하지 않는다. 내부의 가상 입력 이동 `("__none", 음절)`은 최종 경로와 방문 수에서 제외한다. 사전의 끝 음절 위치에 없는 글자도 연결 규칙에 따라 시작할 수 있도록 처리하며, 입력은 Unicode 스칼라 값 하나와 규칙 번호 0..=10을 요구한다.

정적 승패만으로 조회가 끝나면 `optimal_path`는 비어 있고 DFS 방문 수는 0이다. 이때 첫 승리 단어는 `get_syllable_info()`로 얻는다. 루트 상태를 탐색해 승리가 확정되면 witness API의 `winning_move`가 증명된 첫 수다. 기존 탐색의 경로 선택용 역추적 표식만 보고 첫 승리 수를 추정하지 않는다.

### 사용 이력까지 반영한 단어 solver

`after_history()`는 이력을 소비한 **GraphSolver만** 반환하고 원래 `WordMap`은 유지한다. `with_history()`는 재분류한 그래프와 실제 남은 단어 목록을 함께 담은 새 **WordSolver**를 만든다. 정적 첫 승리 단어 조회에서도 이미 쓴 단어를 제외하려면 `with_history()` 결과를 사용한다.

`with_history()`는 중복 이력을 한 번만 소비하고, 같은 음절 쌍의 다른 단어 순서와 원본 solver를 유지한다. 사전에 없는 사용 단어는 오류로 처리한다. 기존 `after_history()`는 사전에 없는 단어를 건너뛰므로 두 함수의 입력 처리도 다르다.

근거: [ai.rs](../game_ai_rust/src/ai.rs)의 음절 API, [words.rs](../game_ai_rust/src/words.rs)의 `get_syllable_info()`·`after_history()`·`with_history()`, [syllable 테스트](../game_ai_rust/tests/syllable.rs), [syllable_info 테스트](../game_ai_rust/tests/syllable_info.rs), [history_solver 테스트](../game_ai_rust/tests/history_solver.rs).

## 6. 확인한 호환성과 검증 범위

이번 문서 작성에서 다음을 실행했다.

| 실행 | 결과 |
| --- | --- |
| `game-ai/`에서 `npm test` | 원본 빌드와 테스트 23개 통과 |
| `game_ai_rust/`에서 `cargo test --locked --offline` | Rust 테스트 81개 통과 |
| 원본 `generate_golden.mjs`로 임시 경로에 기준 결과 재생성 | 52개 분석 사례, 16,676개 음절 변환 기록 생성 |
| 기존 golden과 재생성 JSON의 전체 값 비교 | 분석·변환 결과 일치; 입력 파일 체크섬 메타데이터만 차이 |

체크섬 차이는 현재 Windows 체크아웃의 `words.txt`가 CRLF인 데서 발생했다. LF로 정규화한 파일의 SHA-256은 저장된 golden의 체크섬과 일치했다. 비교 기준 파일은 덮어쓰지 않았다.

`tests/parity.rs`는 원본으로 계산한 결과를 기준으로 단어 목록, 두 위치의 음절 분류·깊이, 세 파티션의 단어 간선, 단어 분류, 루트·SCC 정보, 후속 단어와 종료 판정, 단어 카드·내보내기, 작은 그래프의 탐색 승패·경로, 음절 변환을 비교한다. 현재 golden에는 탐색 질의 36개와 단어 카드·내보내기 비교를 포함한 사례 18개가 있다.

분류 map과 파티션 간선의 일부 비교는 정렬 후 수행하므로, 그 테스트만으로 모든 내부 삽입 순서가 같다고 증명할 수는 없다. 반환 단어 배열·경로의 순서 비교와 별도의 동률·이력·파티션 회귀 테스트가 그 부분을 보완한다. `tests/parallel.rs`는 3정점 그래프의 512개 간선 조합과 다중 간선 사례에서 단일·분기 탐색의 승패, 경로와 방문 수를 비교한다.

이번 확인은 무작위 선택 문자열, 실행시간, 모든 JavaScript 정규식, 모든 Unicode 입력의 동등성을 보장하지 않는다. 전체 사전 비교, `remote` 기능을 이용한 실제 다운로드, WASM 빌드와 브라우저 병렬 실행은 이번 작업에서 재실행하지 않았다. `game_ai_rust/docs/port-plan.md`에 기록된 전체 사전 비교와 테스트 45개는 과거 검증 기록이다.

근거: [golden 생성 스크립트](../game_ai_rust/scripts/generate_golden.mjs), [parity 테스트](../game_ai_rust/tests/parity.rs), [parallel 테스트](../game_ai_rust/tests/parallel.rs), [기존 포팅 기록](../game_ai_rust/docs/port-plan.md).
