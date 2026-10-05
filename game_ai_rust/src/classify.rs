//! Outcome propagation and parity-preserving reduction of the word graph.
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    Edge, NodeMap, NodeType, SingleMove, graph::BipartiteDiGraph, partitions::GraphPartitions,
};

pub type TwoCycle = (SingleMove, SingleMove, usize);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PruneResult {
    pub type_map: NodeMap<NodeType>,
    pub even_loops: Vec<Edge>,
    pub loop_map: IndexMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Classification {
    pub graphs: GraphPartitions,
    pub even_loops: Vec<Edge>,
    pub two_cycles: Vec<TwoCycle>,
    pub type_map: NodeMap<NodeType>,
    pub loop_map: IndexMap<String, String>,
}

fn seed_type(
    graphs: &mut GraphPartitions,
    pos: usize,
    node: &str,
    even_loops: &mut Vec<Edge>,
    loop_map: &mut IndexMap<String, String>,
) -> NodeType {
    let graph = graphs.get_graph("route");
    let degree = graph.out_degree(pos, node);
    if degree == 0 {
        return NodeType::Lose;
    }
    if pos == 0 || degree >= 2 {
        return NodeType::Route;
    }
    let successor = graph.successors(1, node)[0].clone();
    if graph.out_degree(0, &successor) != 1 || graph.successors(0, &successor)[0] != node {
        return NodeType::Route;
    }
    let count = graph.get_edge_num(node, &successor);
    let parity = count % 2;
    if count > parity {
        graphs
            .transfer_edge("route", "removed", node, &successor, count - parity)
            .expect("loop reduction cannot exceed its edge count");
    }
    if count >= 2 {
        even_loops.push((node.to_owned(), successor.clone(), count - parity));
    }
    if parity == 1 {
        loop_map.insert(node.to_owned(), successor);
        NodeType::LoopWin
    } else {
        NodeType::Lose
    }
}

pub fn prune_win_lose_nodes(graphs: &mut GraphPartitions) -> PruneResult {
    let mut result = PruneResult {
        type_map: [IndexMap::new(), IndexMap::new()],
        even_loops: Vec::new(),
        loop_map: IndexMap::new(),
    };
    let mut stack = Vec::new();
    for pos in 0..2 {
        for node in graphs.get_graph("route").nodes(pos) {
            let kind = seed_type(
                graphs,
                pos,
                &node,
                &mut result.even_loops,
                &mut result.loop_map,
            );
            if kind != NodeType::Route {
                result.type_map[pos].insert(node.clone(), kind);
                stack.push((pos, node));
            }
        }
    }
    while let Some((pos, node)) = stack.pop() {
        let opposite = 1 - pos;
        let predecessors: Vec<_> = graphs
            .get_graph("route")
            .predecessors(pos, &node)
            .into_iter()
            .filter(|name| !result.type_map[opposite].contains_key(name))
            .collect();
        let kind = result.type_map[pos][&node];
        graphs.transfer_node("route", "winlose", pos, &node);
        for predecessor in predecessors {
            let next_kind = match (pos, kind) {
                (0, NodeType::Lose) => NodeType::Win,
                (1, NodeType::Win | NodeType::LoopWin) => kind,
                _ => seed_type(
                    graphs,
                    opposite,
                    &predecessor,
                    &mut result.even_loops,
                    &mut result.loop_map,
                ),
            };
            if next_kind != NodeType::Route {
                // A losing head propagates loss only after every transformation
                // option of its predecessor has been pruned.
                let next_kind = if pos == 1 && kind == NodeType::Lose {
                    NodeType::Lose
                } else {
                    next_kind
                };
                result.type_map[opposite].insert(predecessor.clone(), next_kind);
                stack.push((opposite, predecessor));
            }
        }
    }
    result
}

fn sink_kind(
    pos: usize,
    node: &str,
    types: &NodeMap<NodeType>,
    degrees: &NodeMap<usize>,
) -> Option<NodeType> {
    let degree = degrees[pos].get(node).copied().unwrap_or(0);
    if types[pos].get(node) == Some(&NodeType::Lose) && degree == 0 {
        Some(NodeType::Lose)
    } else if pos == 1 && types[pos].get(node) == Some(&NodeType::LoopWin) && degree == 1 {
        Some(NodeType::LoopWin)
    } else {
        None
    }
}

/// Depth measures words played. Transformation edges do not increment it.
pub fn get_depth_map(graph: &BipartiteDiGraph, type_map: &NodeMap<NodeType>) -> NodeMap<usize> {
    let mut lose_stack: [Vec<String>; 2] = [Vec::new(), Vec::new()];
    let mut win_stack: [Vec<String>; 2] = [Vec::new(), Vec::new()];
    let mut degrees = graph.get_out_degree_map();
    let mut depths = [IndexMap::new(), IndexMap::new()];
    for pos in 0..2 {
        for node in graph.nodes(pos) {
            match sink_kind(pos, &node, type_map, &degrees) {
                Some(NodeType::Lose) => {
                    lose_stack[pos].push(node.clone());
                    depths[pos].insert(node, 0);
                }
                Some(NodeType::LoopWin) => {
                    win_stack[pos].push(node.clone());
                    depths[pos].insert(node, 1);
                }
                _ => {}
            }
        }
    }
    while lose_stack
        .iter()
        .chain(win_stack.iter())
        .any(|stack| !stack.is_empty())
    {
        while let Some(node) = lose_stack[1].pop() {
            for pred in graph.predecessors(1, &node) {
                if depths[0].contains_key(&pred) {
                    continue;
                }
                let degree = degrees[0].get_mut(&pred).expect("graph degree exists");
                *degree -= 1;
                if sink_kind(0, &pred, type_map, &degrees).is_some() {
                    depths[0].insert(pred.clone(), depths[1][&node]);
                    lose_stack[0].push(pred);
                }
            }
        }
        while let Some(node) = lose_stack[0].pop() {
            for pred in graph.predecessors(0, &node) {
                if depths[1].contains_key(&pred) {
                    continue;
                }
                depths[1].insert(pred.clone(), depths[0][&node] + 1);
                win_stack[1].push(pred);
            }
        }
        while let Some(node) = win_stack[1].pop() {
            for pred in graph.predecessors(1, &node) {
                if depths[0].contains_key(&pred) {
                    continue;
                }
                depths[0].insert(pred.clone(), depths[1][&node]);
                win_stack[0].push(pred);
            }
        }
        while let Some(node) = win_stack[0].pop() {
            for pred in graph.predecessors(0, &node) {
                if depths[1].contains_key(&pred) {
                    continue;
                }
                let degree = degrees[1].get_mut(&pred).expect("graph degree exists");
                *degree -= 1;
                if let Some(kind) = sink_kind(1, &pred, type_map, &degrees) {
                    let depth = depths[0][&node] + if kind == NodeType::LoopWin { 2 } else { 1 };
                    depths[1].insert(pred.clone(), depth);
                    if kind == NodeType::LoopWin {
                        win_stack[1].push(pred);
                    } else {
                        lose_stack[1].push(pred);
                    }
                }
            }
        }
    }
    depths
}

pub fn classify(graph: BipartiteDiGraph, flow: usize) -> Classification {
    let mut result = Classification {
        graphs: GraphPartitions::new(graph),
        even_loops: Vec::new(),
        two_cycles: Vec::new(),
        type_map: [IndexMap::new(), IndexMap::new()],
        loop_map: IndexMap::new(),
    };
    for _ in 0..200 {
        let mut changed = false;
        // Every operation runs even if an earlier operation changed the graph.
        let operations = if flow == 0 { [0, 1, 2] } else { [1, 2, 0] };
        for operation in operations {
            match operation {
                0 => {
                    let pruned = prune_win_lose_nodes(&mut result.graphs);
                    changed |= pruned.type_map.iter().any(|map| !map.is_empty())
                        || !pruned.even_loops.is_empty()
                        || !pruned.loop_map.is_empty();
                    for (pos, map) in pruned.type_map.into_iter().enumerate() {
                        result.type_map[pos].extend(map);
                    }
                    result.even_loops.extend(pruned.even_loops);
                    result.loop_map.extend(pruned.loop_map);
                }
                1 => {
                    let loops = result.graphs.get_graph("route").get_even_loops();
                    changed |= !loops.is_empty();
                    for (start, end, count) in &loops {
                        result
                            .graphs
                            .transfer_edge("route", "removed", start, end, *count)
                            .expect("even-loop reduction cannot exceed its edge count");
                    }
                    result.even_loops.extend(loops);
                }
                _ => {
                    let cycles = result.graphs.get_graph("route").get_two_cycles();
                    changed |= !cycles.is_empty();
                    for ((start1, end1), (start2, end2), count) in &cycles {
                        result
                            .graphs
                            .transfer_edge("route", "removed", start1, end1, *count)
                            .expect("cycle reduction cannot exceed its edge count");
                        result
                            .graphs
                            .transfer_edge("route", "removed", start2, end2, *count)
                            .expect("cycle reduction cannot exceed its edge count");
                    }
                    result.two_cycles.extend(cycles);
                }
            }
        }
        if !changed {
            break;
        }
    }
    result
}
