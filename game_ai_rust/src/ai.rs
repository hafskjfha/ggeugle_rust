//! Synchronous searches and turn selection. No threads or workers are created.
use crate::{
    Error, Result, classify::prune_win_lose_nodes, graph::BipartiteDiGraph,
    partitions::GraphPartitions, rules::get_head_tail, types::*, words::WordSolver,
};
use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use std::time::Duration;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::time::Instant;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use web_time::Instant;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub is_win: bool,
    /// Milliseconds for search_is_win; rounded seconds for a streamed Done event.
    pub duration: f64,
    pub optimal_path: Vec<SingleMove>,
    pub visited: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum SearchEvent {
    Stack { payload: Vec<SingleMove> },
    Done { payload: SearchResult },
}

/// A root movement after the same reductions used by the sequential search.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootSearchPlan {
    pub graph: BipartiteDiGraph,
    pub movement: SingleMove,
    pub moves: Vec<SingleMove>,
    #[serde(default)]
    pub result: Option<SearchResult>,
}

/// Includes the original root backtracking marker, needed to preserve path ties.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RootBranchResult {
    pub result: SearchResult,
    pub outcome: NodeType,
}

enum Visit {
    Push(SingleMove),
    Pop(NodeType),
}
struct Frame {
    graph: BipartiteDiGraph,
    moves: Vec<SingleMove>,
    next: usize,
}

fn check_deadline(start: Instant, limit: Option<Duration>) -> Result<()> {
    if limit.is_some_and(|limit| start.elapsed() >= limit) {
        Err(Error::Timeout)
    } else {
        Ok(())
    }
}

/// Apply the same determined-node and paired-cycle reductions as the TypeScript search.
pub fn check_initial_condition(graph: &mut BipartiteDiGraph, node: &str) -> NodeType {
    let mut graphs = GraphPartitions::new(std::mem::take(graph));
    let first = prune_win_lose_nodes(&mut graphs);
    let mut result = first.type_map[0]
        .get(node)
        .copied()
        .map(NodeType::normalized);
    if result.is_none() {
        for (start, end, num) in graphs.get_graph("route").get_even_loops() {
            graphs
                .transfer_edge("route", "removed", &start, &end, num)
                .expect("existing even loop");
        }
        for ((s1, e1), (s2, e2), num) in graphs.get_graph("route").get_two_cycles() {
            graphs
                .transfer_edge("route", "removed", &s1, &e1, num)
                .expect("existing cycle edge");
            graphs
                .transfer_edge("route", "removed", &s2, &e2, num)
                .expect("existing cycle edge");
        }
        let second = prune_win_lose_nodes(&mut graphs);
        result = second.type_map[0]
            .get(node)
            .copied()
            .map(NodeType::normalized);
    }
    *graph = std::mem::take(graphs.get_graph_mut("route"));
    result.unwrap_or(NodeType::Route)
}

fn prepare(
    graph: &BipartiteDiGraph,
    movement: &SingleMove,
    prec: &PrecInfo,
    start: Instant,
    limit: Option<Duration>,
    visit: &mut impl FnMut(Visit),
) -> Result<std::result::Result<Frame, (BipartiteDiGraph, bool)>> {
    check_deadline(start, limit)?;
    visit(Visit::Push(movement.clone()));
    let mut graph = graph.get_reachable_graph(0, &movement.1, None);
    if graph.get_edge_num(&movement.0, &movement.1) > 0 {
        graph.decrease_edge(&movement.0, &movement.1, 1)?;
    }
    let node_type = check_initial_condition(&mut graph, &movement.1);
    check_deadline(start, limit)?;
    match node_type {
        NodeType::Win => {
            visit(Visit::Pop(NodeType::Win));
            Ok(Err((graph, true)))
        }
        NodeType::Lose => {
            visit(Visit::Pop(NodeType::Lose));
            Ok(Err((graph, false)))
        }
        _ => {
            let mut moves = graph.get_moves_from_node(&movement.1, 0, 0, None);
            moves.sort_by(|a, b| graph.compare_next_move_num(a, b, prec));
            Ok(Ok(Frame {
                graph,
                moves,
                next: 0,
            }))
        }
    }
}

// Explicit DFS frames avoid depending on the process stack for long word chains.
fn search_next_turn(
    graph: &BipartiteDiGraph,
    movement: &SingleMove,
    prec: &PrecInfo,
    start: Instant,
    limit: Option<Duration>,
    mut visit: impl FnMut(Visit),
) -> Result<bool> {
    let frame = match prepare(graph, movement, prec, start, limit, &mut visit)? {
        Ok(frame) => frame,
        Err((_, outcome)) => return Ok(outcome),
    };
    let mut frames = vec![frame];
    let mut child_result = None;
    loop {
        check_deadline(start, limit)?;
        if child_result == Some(false) {
            frames.pop();
            visit(Visit::Pop(NodeType::Lose));
            if frames.is_empty() {
                return Ok(true);
            }
            child_result = Some(true);
            continue;
        }
        child_result = None;
        let frame = frames.last_mut().unwrap();
        if frame.next == frame.moves.len() {
            frames.pop();
            visit(Visit::Pop(NodeType::Win));
            if frames.is_empty() {
                return Ok(false);
            }
            child_result = Some(false);
            continue;
        }
        let movement = frame.moves[frame.next].clone();
        frame.next += 1;
        match prepare(&frame.graph, &movement, prec, start, limit, &mut visit)? {
            Ok(next) => frames.push(next),
            Err((_, outcome)) => child_result = Some(outcome),
        }
    }
}

fn run_search(
    graph: &BipartiteDiGraph,
    movement: &SingleMove,
    prec: &PrecInfo,
    limit: Option<Duration>,
    mut on_stack: impl FnMut(Vec<SingleMove>),
) -> Result<RootBranchResult> {
    let start = Instant::now();
    let mut stack: Vec<SingleMove> = vec![];
    let mut max_branch: Vec<Option<Vec<SingleMove>>> = vec![];
    let mut visited = 0;
    let mut root_outcome = NodeType::Route;
    let next_wins = search_next_turn(graph, movement, prec, start, limit, |visit| {
        match visit {
            Visit::Push(movement) => {
                stack.push(movement);
                visited += 1;
            }
            Visit::Pop(outcome) => {
                let movement = stack.pop().expect("balanced search traversal");
                let depth = stack.len();
                if depth == 0 {
                    root_outcome = outcome;
                }
                max_branch.resize_with(max_branch.len().max(depth + 2), || None);
                let mut branch = max_branch[depth + 1].take().unwrap_or_default();
                branch.push(movement);
                if outcome == NodeType::Win
                    || max_branch[depth]
                        .as_ref()
                        .is_none_or(|b| b.len() < branch.len())
                {
                    max_branch[depth] = Some(branch);
                }
            }
        }
        on_stack(stack.clone());
    })?;
    let mut optimal_path = max_branch
        .first_mut()
        .and_then(Option::take)
        .unwrap_or_default();
    optimal_path.reverse();
    Ok(RootBranchResult {
        result: SearchResult {
            is_win: !next_wins,
            duration: start.elapsed().as_secs_f64() * 1000.0,
            optimal_path,
            visited,
        },
        outcome: root_outcome,
    })
}

/// Consume the incoming movement once and enumerate independent ordered replies.
pub fn prepare_root_search(
    graph: &BipartiteDiGraph,
    movement: &SingleMove,
    prec: &PrecInfo,
    limit: Option<Duration>,
) -> Result<RootSearchPlan> {
    let start = Instant::now();
    let (graph, moves, result) = match prepare(graph, movement, prec, start, limit, &mut |_| {})? {
        Ok(frame) => (frame.graph, frame.moves, None),
        Err((graph, next_wins)) => (
            graph,
            vec![],
            Some(SearchResult {
                is_win: !next_wins,
                duration: start.elapsed().as_secs_f64() * 1000.0,
                optimal_path: vec![movement.clone()],
                visited: 1,
            }),
        ),
    };
    Ok(RootSearchPlan {
        graph,
        movement: movement.clone(),
        moves,
        result,
    })
}

/// Search an independent reply and stream owned stack snapshots at most once a second.
/// The final result is returned; this callback receives only Stack events.
pub fn search_root_branch(
    graph: &BipartiteDiGraph,
    movement: &SingleMove,
    prec: &PrecInfo,
    limit: Option<Duration>,
    mut callback: impl FnMut(SearchEvent),
) -> Result<RootBranchResult> {
    let mut last_sent = None;
    run_search(graph, movement, prec, limit, |stack| {
        if last_sent.is_none_or(|last: Instant| last.elapsed() >= Duration::from_secs(1)) {
            callback(SearchEvent::Stack { payload: stack });
            last_sent = Some(Instant::now());
        }
    })
}

/// Merge the completed ordered prefix exactly as sequential DFS backtracking does.
/// Results after the first winning reply need not be present and are not counted.
pub fn finish_root_search(
    plan: &RootSearchPlan,
    results: &[Option<RootBranchResult>],
    duration: f64,
) -> Result<SearchResult> {
    if !duration.is_finite() || duration < 0.0 {
        return Err(Error::InvalidInput(
            "duration must be finite and nonnegative".into(),
        ));
    }
    if let Some(result) = &plan.result {
        let mut result = result.clone();
        result.duration = duration;
        return Ok(result);
    }
    if results.len() > plan.moves.len() {
        return Err(Error::InvalidInput("too many root branch results".into()));
    }
    let mut selected: Option<&Vec<SingleMove>> = None;
    let mut visited = 1usize;
    let mut is_win = true;
    for (index, movement) in plan.moves.iter().enumerate() {
        let branch = results
            .get(index)
            .and_then(Option::as_ref)
            .ok_or_else(|| Error::InvalidInput(format!("root branch {index} has not completed")))?;
        if !matches!(branch.outcome, NodeType::Win | NodeType::Lose)
            || branch.result.optimal_path.first() != Some(movement)
        {
            return Err(Error::InvalidInput(format!(
                "invalid result for root branch {index}"
            )));
        }
        visited = visited
            .checked_add(branch.result.visited)
            .ok_or_else(|| Error::InvalidInput("root search visited count overflow".into()))?;
        if branch.outcome == NodeType::Win
            || selected.is_none_or(|path| path.len() < branch.result.optimal_path.len())
        {
            selected = Some(&branch.result.optimal_path);
        }
        if branch.result.is_win {
            is_win = false;
            break;
        }
    }
    let mut optimal_path = vec![plan.movement.clone()];
    if let Some(branch) = selected {
        optimal_path.extend_from_slice(branch);
    }
    Ok(SearchResult {
        is_win,
        duration,
        optimal_path,
        visited,
    })
}

pub fn search_is_win(
    graph: &BipartiteDiGraph,
    movement: &SingleMove,
    prec: &PrecInfo,
    limit: Option<Duration>,
) -> Result<SearchResult> {
    run_search(graph, movement, prec, limit, |_| {}).map(|branch| branch.result)
}

pub fn start_streaming_single_thread_search(
    graph: &BipartiteDiGraph,
    movement: &SingleMove,
    prec: &PrecInfo,
    limit: Option<Duration>,
    mut callback: impl FnMut(SearchEvent),
) -> Result<SearchResult> {
    let mut last_sent = None;
    let result = run_search(graph, movement, prec, limit, |stack| {
        if last_sent.is_none_or(|last: Instant| last.elapsed() >= Duration::from_secs(1)) {
            callback(SearchEvent::Stack { payload: stack });
            last_sent = Some(Instant::now());
        }
    })?
    .result;
    let mut streamed = result.clone();
    streamed.duration = (streamed.duration / 10.0).round() / 100.0;
    callback(SearchEvent::Done { payload: streamed });
    Ok(result)
}

#[derive(Clone, Debug)]
pub struct AiOptions {
    pub difficulty: usize,
    /// Deadline per candidate, as in the original AI.
    pub calculating_duration: Duration,
    pub stealable: bool,
    pub precedence: PrecInfo,
    pub flow: usize,
}
impl Default for AiOptions {
    fn default() -> Self {
        Self {
            difficulty: 2,
            calculating_duration: Duration::from_secs(1),
            stealable: false,
            precedence: PrecInfo::default(),
            flow: 0,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum GameEvent {
    #[serde(rename = "debug")]
    Debug { payload: String },
    #[serde(rename = "move")]
    Move { payload: String },
    #[serde(rename = "computerWin")]
    ComputerWin,
    #[serde(rename = "messageEnd")]
    MessageEnd,
}

pub fn is_game_end(solver: &WordSolver, history: &[String], stealable: bool) -> Result<bool> {
    if history.len() == 1 && stealable {
        return Ok(false);
    }
    Ok(solver.get_next_words(history)?.is_empty())
}
pub fn choose_move(
    solver: &WordSolver,
    history: &[String],
    options: &AiOptions,
) -> Result<Option<String>> {
    choose_move_with_callback(solver, history, options, |_| {})
}
pub fn choose_move_with_callback(
    solver: &WordSolver,
    history: &[String],
    options: &AiOptions,
    mut callback: impl FnMut(GameEvent),
) -> Result<Option<String>> {
    if options.difficulty > 2 || options.flow > 1 {
        return Err(Error::InvalidInput(
            "difficulty must be 0..=2 and flow must be 0 or 1".into(),
        ));
    }
    let legal = solver.get_next_words(history)?;
    let steal = options.stealable && history.len() == 1;
    if legal.is_empty() && !steal {
        callback(GameEvent::MessageEnd);
        return Ok(None);
    }
    let mut rng = rand::rng();
    let word_for_move = |movement: &SingleMove| {
        solver
            .word_map
            .get(&movement.0, &movement.1)
            .and_then(|words| words.iter().find(|word| !history.contains(word)))
            .cloned()
    };
    let mut selected = None;
    if options.difficulty == 0 && !legal.is_empty() {
        // Select a pair uniformly, then its first unused word, matching the original.
        let mut pairs = indexmap::IndexMap::<SingleMove, String>::new();
        for word in &legal {
            let pair = get_head_tail(word, solver.head_idx, solver.tail_idx)?;
            pairs.entry(pair).or_insert_with(|| word.clone());
        }
        selected = pairs
            .get_index(rng.random_range(0..pairs.len()))
            .map(|(_, word)| word.clone());
        callback(GameEvent::Debug {
            payload: "랜덤 단어".into(),
        });
    } else if history.is_empty() {
        let route = solver.graph_solver.graphs.get_graph("route");
        let mut movements: Vec<_> = route.edges(1).into_iter().map(|(s, e, _)| (s, e)).collect();
        if !movements.is_empty() {
            if options.difficulty == 1 {
                selected = word_for_move(&movements[rng.random_range(0..movements.len())]);
            } else {
                movements.shuffle(&mut rng);
                let mut longest = 0.0;
                for movement in movements {
                    callback(GameEvent::Debug {
                        payload: format!("({},{}) 탐색", movement.0, movement.1),
                    });
                    match search_is_win(
                        route,
                        &movement,
                        &options.precedence,
                        Some(options.calculating_duration),
                    ) {
                        Ok(result) => {
                            if selected.is_none() || result.duration >= longest {
                                longest = result.duration;
                                selected = word_for_move(&movement);
                            }
                        }
                        Err(Error::Timeout) => {
                            selected = word_for_move(&movement);
                            break;
                        }
                        Err(error) => return Err(error),
                    }
                }
            }
        } else {
            let graph_solver = &solver.graph_solver;
            if let Some((node, _)) = graph_solver.depth_map[1]
                .iter()
                .reduce(|first, next| if next.1 > first.1 { next } else { first })
            {
                let movement =
                    if graph_solver.get_node_type(node, 1, None).normalized() == NodeType::Win {
                        graph_solver.get_winning_optimal_move(1, node, None)
                    } else {
                        graph_solver.get_losing_optimal_move(1, node, None)
                    };
                selected = movement.as_ref().and_then(word_for_move);
            }
        }
    } else {
        let (_, node) = get_head_tail(history.last().unwrap(), solver.head_idx, solver.tail_idx)?;
        let next = solver.after_history(history, options.flow)?;
        let node_type = next.get_node_type(&node, 0, None).normalized();
        match node_type {
            NodeType::Win => {
                selected = next
                    .get_winning_optimal_move(0, &node, None)
                    .as_ref()
                    .and_then(word_for_move);
            }
            NodeType::Lose => {
                if steal {
                    selected = Some(history[0].clone());
                } else {
                    selected = next
                        .get_losing_optimal_move(0, &node, None)
                        .as_ref()
                        .and_then(word_for_move);
                }
            }
            _ => {
                let route = next.graphs.get_graph("route");
                let mut movements = route.get_moves_from_node(&node, 0, 0, None);
                movements.sort_by(|a, b| route.compare_next_move_num(a, b, &options.precedence));
                if !movements.is_empty() {
                    selected = word_for_move(&movements[rng.random_range(0..movements.len())]);
                    if options.difficulty == 2 {
                        let mut longest = 0.0;
                        let mut all_losing = true;
                        for movement in movements {
                            callback(GameEvent::Debug {
                                payload: format!("({},{}) 탐색", movement.0, movement.1),
                            });
                            match search_is_win(
                                route,
                                &movement,
                                &options.precedence,
                                Some(options.calculating_duration),
                            ) {
                                Ok(result) => {
                                    if result.is_win {
                                        selected = word_for_move(&movement);
                                        all_losing = false;
                                        break;
                                    }
                                    if result.duration >= longest {
                                        longest = result.duration;
                                        selected = word_for_move(&movement);
                                    }
                                }
                                Err(Error::Timeout) => {
                                    selected = word_for_move(&movement);
                                    all_losing = false;
                                    break;
                                }
                                Err(error) => return Err(error),
                            }
                        }
                        if all_losing && steal {
                            selected = Some(history[0].clone());
                        }
                    }
                }
            }
        }
    }
    let word = selected
        .or_else(|| legal.first().cloned())
        .or_else(|| steal.then(|| history[0].clone()));
    if let Some(word) = &word {
        callback(GameEvent::Move {
            payload: word.clone(),
        });
        let mut following = history.to_vec();
        following.push(word.clone());
        if is_game_end(solver, &following, options.stealable)? {
            callback(GameEvent::ComputerWin);
        }
    }
    callback(GameEvent::MessageEnd);
    Ok(word)
}
