//! Korean word-chain game analysis. Searches run synchronously on the calling thread.
pub mod types;
pub use types::*;
pub mod ai;
pub mod classify;
pub mod edge_map;
pub mod engine;
pub mod graph;
pub mod pairs;
pub mod partitions;
pub mod presets;
pub mod rules;
pub mod solver;
pub mod strategy;
pub mod words;
pub use ai::{
    AiOptions, GameEvent, RootBranchResult, RootSearchPlan, SearchEvent, SearchResult, choose_move,
    choose_move_with_callback, finish_root_search, is_game_end, prepare_root_search, search_is_win,
    search_root_branch, start_streaming_single_thread_search,
};
pub use edge_map::{EdgeCounter, EdgeMap};
pub use engine::{get_wc_data, start_streaming_critical_words_info, update_solver};
pub use graph::{BipartiteDiGraph, DiGraph};
pub use partitions::GraphPartitions;
pub use rules::{ChangeRule, RuleForm, WordRule, load_words};
pub use solver::GraphSolver;
pub use strategy::{StrategyTree, WcStrategyTree};
pub use words::{WordMap, WordSolver};

#[derive(Debug)]
pub enum Error {
    InvalidInput(String),
    Io(std::io::Error),
    Regex(Box<fancy_regex::Error>),
    Json(serde_json::Error),
    Timeout,
    Cancelled,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(s) => f.write_str(s),
            Self::Io(e) => e.fmt(f),
            Self::Regex(e) => e.fmt(f),
            Self::Json(e) => e.fmt(f),
            Self::Timeout => f.write_str("search deadline exceeded"),
            Self::Cancelled => f.write_str("search cancelled"),
        }
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<fancy_regex::Error> for Error {
    fn from(e: fancy_regex::Error) -> Self {
        Self::Regex(Box::new(e))
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
pub type Result<T> = std::result::Result<T, Error>;
