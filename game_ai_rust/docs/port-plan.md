# game_ai Rust port

Goal: preserve the existing TypeScript word-chain engine in a standalone `game_ai_rust` crate. The user explicitly excludes worker-based search splitting; all search is synchronous and uses cooperative deadlines.

Architecture: insertion-ordered bipartite graphs retain the original classification and word index semantics. Rules and Hangul transforms, graph storage, classification/analysis, word queries, and AI/search are separate Rust modules. Serializable rule structs accept the original camelCase JSON. Local dictionaries work offline; selected HTTP dictionaries are available through the optional `remote` feature.

Constraints: retain both classification flows, all 11 transformations, rule presets and precedence maps, per-word history consumption, stealing, three difficulty levels, callback search events, strategy trees, and exports. Keep the original TypeScript package intact. Fixtures come from `data/all_words.txt`, with a deterministic extraction script and provenance.

- [x] Rules: dictionary filtering, transformations and indexing, rules/loading/presets with behavior tests.
- [x] Graph storage: multiplicities, reachability and cycles, ordered graph and partitions with behavior tests.
- [x] Classification: forced chains, odd/even loops and paired cycles, depths, pairs, SCC summaries and move analysis.
- [x] Word/AI: history, postprocessing, mover win semantics, timeout fallback, stealing, synchronous search, callbacks and strategy trees.
- [x] Integration: real-dictionary fixtures and TypeScript golden outputs; transformations, graph types/depths/partitions, word exports and search results.
- [x] Verification: Rust tests, formatting, clippy, release example, complete dictionary comparisons, original TypeScript tests and independent review.

Review focus: empty dictionaries; invalid indices/regexes; parallel words sharing endpoints; repeated stolen words; zero deadline and terminal moves. Golden comparisons use the original engine as an independent oracle; exact random choices and timing are not compared.

## Validation results

- `cargo test --locked --offline --all-features`: 45 tests passed.
- `cargo fmt --check`, `cargo clippy --locked --offline --all-targets --all-features -- -D warnings`: passed.
- `cargo run --locked --release --offline --example play`: `AI: 과자`.
- Original TypeScript `npm test`: 23 tests passed; original package files are unchanged.
- 384 real sampled words with source hash/line provenance; 52 engine cases and 16,676 transformation records, including word exports and tiny-search optimal paths.
- Complete 265,360-word dictionary: TypeScript and Rust agree on word counts, 115,211 distinct pairs, node/word classifications and major-route statistics for change 1 / flow 0, change 1 / flow 1, and change 10 / flow 0.
- Independent review found history merge ordering and first-opening tie behavior; both were reproduced with failing tests, corrected and re-reviewed successfully.

Validation environment: Windows, Rust 1.97.1, Node 24.11.1. The optional remote feature was compiled and tested without fetching external dictionaries. Unicode/regex/TreeData interface differences are documented in README.md.
