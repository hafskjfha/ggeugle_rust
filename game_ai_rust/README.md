# 끝말잇기 게임 AI — Rust

`../game-ai` TypeScript 패키지의 독립 Rust 포팅입니다. 승패 분류, 돌림 단어 제거, SCC 분석, 단어 조회와 내보내기, 필승 탐색, 난이도별 AI, 전략 트리를 제공합니다. 탐색은 호출한 스레드에서 실행하며 워커 생성이나 분산 탐색은 없습니다.

## 실행

Rust 1.97.1에서 빌드·검증했습니다(edition 2024). Cargo.lock을 포함합니다.

```bash
cd game_ai_rust
cargo test --locked
cargo run --release --example play
cargo run --release --example analyze
cargo run --release --example sample_analysis
```

`play`는 실제 사전에 존재하는 `사과 → 과자 → 자두` 수순을 사용하며 `AI: 과자`를 출력합니다. `analyze`는 기본적으로 포함된 384개 단어 샘플을 분석합니다. 전체 사전 경로, 음절 변환 번호, 분류 순서를 전달할 수 있습니다.

```bash
cargo run --release --example analyze -- ../data/all_words.txt 1 0
```

출력에는 단어 개수, 음절 쌍 개수, 분석 시간, 음절/단어 승패 분류와 주요 루트 통계가 포함됩니다. 사전 원본이 다른 위치에 있으면 첫 인자에 그 파일 경로를 지정하세요.

### 샘플 단어의 승/패 판정 확인

`sample_analysis`는 기본 샘플 384개를 읽어 다음 결과를 사람이 읽을 수 있는 표로 출력합니다.

- 단어 분류별 개수: 공격(승), 방어/양보(패), 루트, 돌림 단어 등.
- 받는 음절과 제시 음절 각각의 승/루프승/패/루트 판정과 깊이.
- 모든 개별 단어의 분류, 첫·끝 음절 판정, 깊이, 돌림 단어의 짝.

```bash
cargo run --release --example sample_analysis
# 사전 경로, 음절 변환 번호, 분류 순서를 바꿔 분석
cargo run --release --example sample_analysis -- tests/fixtures/words.txt 0 1
```

음절 판정은 해당 음절에서 차례를 시작하는 플레이어 기준입니다. 단어 표는 사용 이력이 없는 초기 사전의 정적 그래프 분류이며, 루트는 추가 필승 탐색이 필요한 상태입니다. 루프승은 홀수 개 남은 강제 루프를 활용한 승리 판정입니다. 실제 이력에 따른 분석에는 `after_history`와 `search_is_win`을 사용할 수 있습니다. 깊이는 돌림 쌍을 제거한 승패 그래프의 단어 수 기준이며, `-`는 정의된 값이 없다는 뜻입니다.

## Rust에서 사용

다른 Rust 프로젝트의 Cargo.toml에서 경로 의존성을 추가합니다.

```toml
[dependencies]
game_ai_rust = { path = "../game_ai_rust" }
```

```rust
use game_ai_rust::{AiOptions, RuleForm, choose_move, get_wc_data, is_game_end};

fn main() -> game_ai_rust::Result<()> {
    let rule = RuleForm::manual(&["사과", "과자", "자두"], 0);
    let solver = get_wc_data(&rule, 0)?;
    let history = vec!["사과".to_string()];
    if !is_game_end(&solver, &history, false)? {
        let word = choose_move(&solver, &history, &AiOptions::default())?;
        assert_eq!(word.as_deref(), Some("과자"));
    }
    Ok(())
}
```

`RuleForm`은 원본 camelCase JSON을 serde로 읽고 쓸 수 있습니다. 파일 사전은 `rule.content.word_rule.words = WordSource::File(FileWordsOption { path })`로 지정합니다. `RuleForm::manual`은 처음 음절/마지막 음절 연결, 후처리 없음으로 구성합니다.

## 공개 API

| TypeScript | Rust |
| --- | --- |
| `engineFunctions.getWcData` | `get_wc_data(&rule, flow)` |
| `engineFunctions.updateSolver` | `update_solver(&graphs, &moves, flow)` |
| `engineFunctions.searchIsWin` | `search_is_win(&graph, &move, &prec, timeout)` |
| `startStreamingSingleThreadSearch` | `start_streaming_single_thread_search(..., callback)` |
| `startStreamingCriticalWordsInfo` | `start_streaming_critical_words_info(..., callback)` |
| `GameWorkerRunner.run`의 수 선택 | `choose_move` / `choose_move_with_callback` |
| `isGameEnd` | `is_game_end` |
| `WordSolver`, `GraphSolver`, `WordMap` | 같은 이름, 메서드는 snake_case |
| `StrategyTree`, `WcStrategyTree` | 같은 이름, 동기 콜백/분기 선택 |
| `sampleRules`, `samplePrecedenceMaps` | `presets::sample_rules`, `presets::sample_precedence_maps` |
| `sampleChangeFuncs[i]` | `ChangeRule(i).forward` / `.backward` |

`edge_map`, `graph`, `partitions`, `classify`, `pairs`, `solver`, `rules`, `presets`, `words`, `engine`, `ai`, `strategy` 모듈을 공개합니다. `NodePos`는 0(받는 음절) 또는 1(단어 첫 음절)입니다. 변환 엣지는 0→1, 단어 엣지는 1→0이며 단어 엣지의 개수는 같은 첫/끝 음절 쌍에 속한 단어 개수입니다.

`GraphSolver`에는 최적 수, 음절/단어 분류, SCC 요약, 비교, 분포 조회가 있습니다. `WordSolver`에는 후속 단어, 단어 카드, 필승/필패/루트/돌림 단어 내보내기와 CSV 내보내기가 있습니다. `after_history`는 중복을 제거한 사용 단어를 하나씩 소비한 새 GraphSolver를 만듭니다. 원본 solver는 유지됩니다. serde로 solver를 저장하고 복원해도 메서드를 바로 사용할 수 있습니다.

## AI와 시간제한

- 난이도 0: 합법적인 음절 쌍을 무작위로 골라 첫 미사용 단어를 선택합니다.
- 난이도 1: 승패와 얕은 최적 수를 사용하고, 루트에서는 무작위로 선택합니다.
- 난이도 2: 후보별 단일 스레드 필승 탐색을 실행합니다. 시간제한에 걸리면 합법적인 후보를 선택합니다.

`AiOptions`의 `difficulty`는 0..=2, `calculating_duration`은 후보 하나의 `Duration` 제한, `stealable`은 첫 단어 뺏기 허용 여부, `precedence`는 탐색 순서, `flow`는 분류 순서 0/1입니다. 여러 후보의 총 시간은 후보별 제한보다 길어질 수 있습니다. solver 생성과 같은 flow를 사용하세요.

`choose_move`는 `Result<Option<String>>`을 반환합니다. 종료 상태는 `None`입니다. 콜백 API는 `Debug`, `Move`, `ComputerWin`, `MessageEnd` 이벤트를 전달합니다. 쉬움에서도 더 둘 단어가 없고 첫 단어 뺏기가 가능하면 해당 단어를 반환합니다.

탐색 마감시간은 DFS와 그래프 전처리 경계에서 협력적으로 확인합니다. 개별 그래프 전처리나 콜백 실행 시간까지 강제로 중단하는 제한은 아닙니다. `Some(Duration::ZERO)`는 즉시 `Error::Timeout`, `None`은 제한 없음입니다. DFS는 명시적인 프레임 스택을 사용합니다.

`SearchResult.is_win`은 방금 수를 둔 사람의 승리 여부입니다. `search_is_win`의 duration은 밀리초, 스트리밍 `Done`의 duration은 소수 둘째 자리까지 반올림한 초입니다. 스택 이벤트는 소유한 스냅샷이며, 최대 1초에 한 번 전달됩니다. `optimal_path`에는 음절 쌍이 들어 있습니다.

## 사전과 규칙

11개 변환 규칙, 18개 기본 규칙, 기존 탐색 우선순위 맵과 사전 메타데이터를 포함합니다. 수동/파일 사전은 기본 기능으로 동작합니다. 기존 `selected` 사전 URL 다운로드는 선택 기능입니다.

```bash
cargo test --features remote
```

사용 프로젝트에서는 `features = ["remote"]`를 추가하세요. 다운로드는 동기 HTTP 호출입니다. 기본 테스트는 외부 사전 서버에 접속하지 않습니다.

원본의 분류 순서, 돌림 단어 인덱스, 우선순위, 동률 수 선택과 규칙 7/10의 역변환 특성을 유지합니다. 원본 단어 카드의 `connected` 표시와 그래프의 연결 판정 간 반전도 호환성을 위해 유지합니다.

다음 입력 처리는 Rust에 맞게 명시했습니다.

- 인덱스는 Unicode 문자 기준이며 범위를 벗어나면 오류입니다. 제공된 전체 사전은 BMP 문자만 포함하므로 원본 UTF-16 인덱스와 같습니다.
- 한글/호환 자모가 아닌 문자는 그대로 통과합니다. 원본의 범위 검사 없는 산술로 영문이 자모로 바뀌는 동작은 재현하지 않습니다.
- 정규식은 fancy-regex를 사용해 lookaround/backreference도 지원합니다. JavaScript 전용 문법까지 동일하지는 않으며 지원되지 않는 식은 오류를 반환합니다.
- 전략 트리의 `TreeData.words`는 승리 수에 단어 배열 하나, 패배 턴에 선택 가능한 단어 배열들을 담는 통일된 구조입니다.

## 테스트 데이터 재생성

`tests/fixtures/words.txt`의 모든 단어는 `data/all_words.txt`에서 추출했습니다. 256개 균등 지점, 양 끝 음절의 초성/중성/종성 유형, 강제 수순·순환·동일 음절 쌍·두음 법칙·자모 사례를 함께 선택합니다. `provenance.json`에 원본 SHA-256과 단어별 원본 줄 번호가 있습니다.

```bash
node scripts/sample_words.mjs ../data/all_words.txt
```

`golden.json`은 원본 TypeScript 엔진이 계산한 독립 기준 결과입니다. 비교 테스트는 52개 규칙/분류 사례, 16,676개 변환 기록, 후속 단어, 종료 판정, 단일 스레드 탐색 결과/경로와 단어 내보내기를 검증합니다. 일반 `cargo test`에는 Node나 전체 사전 파일이 필요 없습니다. 기준 결과를 재생성할 때만 원본 패키지를 먼저 빌드합니다.

```bash
# 저장소 루트에서
cd game-ai
npm ci
npm run build
cd ../game_ai_rust
node scripts/generate_golden.mjs
cargo test
```

## 의존성

삽입 순서 보존은 [indexmap](https://docs.rs/indexmap/latest/indexmap/), 규칙 정규식은 [fancy-regex](https://docs.rs/fancy-regex/0.16.2/fancy_regex/), 직렬화는 serde/serde_json, 난수는 rand를 사용합니다. `remote`를 켰을 때만 reqwest를 사용합니다.
