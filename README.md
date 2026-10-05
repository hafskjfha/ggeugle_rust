# 끝말잇기 게임 AI

게임 인공지능과 분석 엔진을 [`game-ai/`](game-ai/) 독립 폴더에 정리했습니다.

```bash
cd game-ai
npm ci
npm test
npm run example
```

구조와 사용법은 [`game-ai/README.md`](game-ai/README.md)를 참고하세요.

Rust 포팅은 [`game_ai_rust/`](game_ai_rust/)에 있습니다. 워커 없이 단일 스레드에서 분석과 탐색을 실행합니다.

```bash
cd game_ai_rust
cargo test
cargo run --release --example play
```

Rust API, 사전 파일 분석과 테스트 샘플 재생성 방법은 [`game_ai_rust/README.md`](game_ai_rust/README.md)를 참고하세요.
