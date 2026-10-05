use crate::{
    Error, Result,
    graph::BipartiteDiGraph,
    partitions::GraphPartitions,
    rules::{ChangeRule, RuleForm, get_head_tail, get_idx, load_words},
    solver::GraphSolver,
    types::*,
    words::{WordMap, WordSolver},
};
use serde::{Deserialize, Serialize};

pub fn get_wc_data(rule: &RuleForm, flow: usize) -> Result<WordSolver> {
    if flow > 1 {
        return Err(Error::InvalidInput("flow must be 0 or 1".into()));
    }
    let connection = &rule.content.word_connection_rule;
    if connection.change_func_idx > 10 {
        return Err(Error::InvalidInput(
            "changeFuncIdx must be in 0..=10".into(),
        ));
    }
    let head = get_idx(connection.raw_head_idx, connection.head_dir)?;
    let tail = get_idx(connection.raw_tail_idx, connection.tail_dir)?;
    let change = ChangeRule(connection.change_func_idx);
    let words = load_words(&rule.content.word_rule)?;
    let mut word_map = WordMap::from_words(&words, head, tail)?;
    let mut graph = BipartiteDiGraph::from_word_map(&word_map, change);
    let post = &rule.content.postprocessing;
    match post.manner.r#type {
        0 => {}
        1 | 2 => loop {
            let sinks = graph.get_sinks(0);
            if sinks.is_empty() {
                break;
            }
            for node in sinks {
                for (start, end, _) in &graph.remove_node(0, &node)[1] {
                    word_map.remove(start, Some(end));
                }
            }
            for node in graph.get_sinks(1) {
                graph.remove_node(1, &node);
            }
            if post.manner.r#type == 1 {
                break;
            }
        },
        3 => {
            let limit = post.manner.next_words_limit.ok_or_else(|| {
                Error::InvalidInput("nextWordsLimit is required for manner 3".into())
            })?;
            for node in graph.next_words_limit_nodes(0, limit) {
                for pred in graph.predecessors(0, &node) {
                    graph.remove_edge(1, &pred, &node);
                    word_map.remove(&pred, Some(&node));
                }
            }
            for node in graph.get_sinks(1) {
                graph.remove_node(1, &node);
            }
        }
        _ => return Err(Error::InvalidInput("manner must be 0..=3".into())),
    }
    let removed: Vec<_> = post.removed_words.split_whitespace().collect();
    let added: Vec<_> = post.added_words.split_whitespace().collect();
    if !removed.is_empty() || !added.is_empty() {
        for word in removed {
            let (s, e) = get_head_tail(word, head, tail)?;
            word_map.remove_word(&s, &e, word);
        }
        for word in added {
            if !word_map.has_word(word, head, tail)? {
                word_map.add_word(word, head, tail)?;
            }
        }
        graph = BipartiteDiGraph::from_word_map(&word_map, change);
    }
    for node in graph.get_sinks(1) {
        graph.remove_node(1, &node);
    }
    for node in graph.get_sources(0) {
        graph.remove_node(0, &node);
    }
    Ok(WordSolver::new(graph, word_map, head, tail, flow))
}

pub fn update_solver(graphs: &GraphPartitions, moves: &[Edge], flow: usize) -> Result<GraphSolver> {
    if flow > 1 {
        return Err(Error::InvalidInput("flow must be 0 or 1".into()));
    }
    let mut partitions = graphs.clone();
    let mut graph = partitions.update_union("winlose").clone();
    for (start, end, num) in moves {
        graph.decrease_edge(start, end, *num)?;
    }
    Ok(GraphSolver::new(graph, flow))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CriticalDifference {
    pub win: Vec<String>,
    pub lose: Vec<String>,
    pub loopwin: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CriticalWordsInfo {
    pub r#move: SingleMove,
    pub difference: CriticalDifference,
}

pub fn start_streaming_critical_words_info(
    graph: &BipartiteDiGraph,
    view: usize,
    flow: usize,
    mut callback: impl FnMut(CriticalWordsInfo),
) -> Result<()> {
    if view > 1 || flow > 1 {
        return Err(Error::InvalidInput("view and flow must be 0 or 1".into()));
    }
    for (start, end) in graph.get_critical_edges() {
        let mut remaining = graph.clone();
        remaining.decrease_edge(&start, &end, 1)?;
        let solver = GraphSolver::new(remaining, flow);
        let mut difference = CriticalDifference {
            win: vec![],
            lose: vec![],
            loopwin: vec![],
        };
        for (node, node_type) in &solver.type_map[view] {
            match node_type {
                NodeType::Win => difference.win.push(node.clone()),
                NodeType::Lose => difference.lose.push(node.clone()),
                NodeType::LoopWin => difference.loopwin.push(node.clone()),
                NodeType::Route => {}
            }
        }
        callback(CriticalWordsInfo {
            r#move: (start, end),
            difference,
        });
    }
    Ok(())
}
