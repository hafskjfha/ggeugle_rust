use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

use crate::{
    Edge, NodeMap, NodeType, SingleMove,
    classify::{classify, get_depth_map},
    edge_map::EdgeMap,
    graph::BipartiteDiGraph,
    pairs::PairManager,
    partitions::GraphPartitions,
    rules::ChangeRule,
};

pub type MoveType = usize;
pub type MoveClass = [IndexMap<usize, EdgeMap<MoveInfoData>>; 6];
pub type ComparisonMap = NodeMap<(NodeType, NodeType)>;
pub type ComparisonData = [Vec<(NodeType, NodeType, Vec<String>)>; 2];

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveInfo {
    #[serde(rename = "type")]
    pub move_type: MoveType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pair: Option<Edge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveInfoData {
    pub node_types: [NodeType; 2],
    pub word_idx: Vec<usize>,
    pub pairs: Vec<Edge>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteInfo {
    pub char_num: usize,
    pub move_num: usize,
    pub average_num: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordTypeCounts {
    pub type_num: [usize; 6],
    pub subtype_num: Vec<Vec<(NodeType, NodeType, usize)>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepthNodes {
    pub depth: usize,
    pub nodes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeTypeCount {
    pub node_type: NodeType,
    pub num: usize,
    pub fill: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepthCount {
    pub depth: usize,
    pub num: usize,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub fill: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SccSuccessor {
    pub nodes: Vec<String>,
    pub by: Vec<(String, String, (usize, usize))>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SccData {
    pub nodes: Vec<String>,
    pub succ: Vec<SccSuccessor>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeableCharsData {
    pub from: Vec<(String, NodeType)>,
    pub to: Vec<(String, NodeType)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DistributionKey {
    Total,
    Move(MoveType),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DistributionDisplay {
    Number,
    Fraction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DistributionCalculation {
    Ratio,
    Difference,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistributionSort {
    pub key: DistributionKey,
    pub desc: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DistributionRow {
    pub char: String,
    pub num: [f64; 6],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DistributionCalculationRow {
    pub char: String,
    pub num: [f64; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphSolver {
    pub graphs: GraphPartitions,
    pub type_map: NodeMap<NodeType>,
    pub depth_map: NodeMap<usize>,
    pub loop_map: IndexMap<String, String>,
    pub pair_manager: PairManager,
    pub scc_map: NodeMap<usize>,
    pub flow: usize,
}

impl GraphSolver {
    pub fn new(graph: BipartiteDiGraph, flow: usize) -> Self {
        let classified = classify(graph, flow);
        let mut type_map = classified.type_map;
        let depth_map = get_depth_map(classified.graphs.get_graph("winlose"), &type_map);
        for (pos, map) in type_map.iter_mut().enumerate() {
            for node in classified.graphs.get_graph("route").nodes(pos) {
                assert!(!map.contains_key(&node), "route node already classified");
                map.insert(node, NodeType::Route);
            }
        }
        let scc_map = classified.graphs.get_graph("route").get_scc_map();
        Self {
            graphs: classified.graphs,
            type_map,
            depth_map,
            loop_map: classified.loop_map,
            pair_manager: PairManager::from_data(&classified.even_loops, &classified.two_cycles),
            scc_map,
            flow,
        }
    }

    pub fn get_node_type(&self, node: &str, pos: usize, rule: Option<ChangeRule>) -> NodeType {
        if let Some(kind) = self.type_map[pos].get(node) {
            return *kind;
        }
        let nodes = self.graphs.get_move_view_nodes(pos, node, 0, rule, None);
        if nodes.iter().any(|name| {
            self.type_map[1]
                .get(name)
                .is_some_and(|kind| kind.normalized() == NodeType::Win)
        }) {
            NodeType::Win
        } else if nodes
            .iter()
            .any(|name| self.type_map[1].get(name) == Some(&NodeType::Route))
        {
            NodeType::Route
        } else {
            NodeType::Lose
        }
    }

    pub fn get_winlose_nodes(&self, kind: NodeType, view: usize) -> Vec<DepthNodes> {
        let mut groups = std::collections::BTreeMap::<usize, Vec<String>>::new();
        for (node, node_kind) in &self.type_map[view] {
            if *node_kind == kind
                && let Some(depth) = self.depth_map[view].get(node)
            {
                groups.entry(*depth).or_default().push(node.clone());
            }
        }
        groups
            .into_iter()
            .map(|(depth, mut nodes)| {
                nodes.sort();
                DepthNodes { depth, nodes }
            })
            .collect()
    }

    pub fn get_route_nodes(&self, pos: usize) -> [Vec<String>; 2] {
        let mut groups: IndexMap<usize, Vec<String>> = IndexMap::new();
        for (node, component) in &self.scc_map[pos] {
            groups.entry(*component).or_default().push(node.clone());
        }
        let mut result: [Vec<String>; 2] = [Vec::new(), Vec::new()];
        // The original threshold counts the selected side only.
        for group in groups.into_values() {
            result[usize::from(group.len() < 3)].extend(group);
        }
        for nodes in &mut result {
            nodes.sort();
        }
        result
    }

    pub fn get_node_type_num(&self, pos: usize) -> Vec<NodeTypeCount> {
        [
            NodeType::Win,
            NodeType::Lose,
            NodeType::LoopWin,
            NodeType::Route,
        ]
        .into_iter()
        .map(|kind| NodeTypeCount {
            node_type: kind,
            num: self.type_map[pos]
                .values()
                .filter(|value| **value == kind)
                .count(),
            fill: type_fill(kind),
        })
        .collect()
    }

    pub fn get_winlose_detail_num_data(&self, pos: usize) -> Vec<DepthCount> {
        let mut groups = std::collections::BTreeMap::<usize, usize>::new();
        for (node, kind) in &self.type_map[pos] {
            if kind.normalized() != NodeType::Route
                && let Some(depth) = self.depth_map[pos].get(node)
            {
                *groups.entry(*depth).or_default() += 1;
            }
        }
        groups
            .into_iter()
            .map(|(depth, num)| {
                let kind = if depth % 2 == 1 {
                    NodeType::Win
                } else {
                    NodeType::Lose
                };
                DepthCount {
                    depth,
                    num,
                    node_type: kind,
                    fill: type_fill(kind),
                }
            })
            .collect()
    }

    pub fn get_winning_optimal_move(
        &self,
        pos: usize,
        node: &str,
        rule: Option<ChangeRule>,
    ) -> Option<SingleMove> {
        if self.get_node_type(node, pos, rule).normalized() != NodeType::Win {
            return None;
        }
        let graph = self.graphs.get_graph("winlose");
        if pos == 1 {
            if self.type_map[1].get(node) == Some(&NodeType::LoopWin) {
                return self
                    .loop_map
                    .get(node)
                    .map(|tail| (node.to_owned(), tail.clone()));
            }
            let successors: Vec<_> = graph
                .successors(1, node)
                .into_iter()
                .filter(|name| {
                    self.type_map[0]
                        .get(name)
                        .is_some_and(|kind| kind.normalized() == NodeType::Lose)
                })
                .collect();
            return choose_depth(&successors, &self.depth_map[0], true)
                .map(|tail| (node.to_owned(), tail));
        }
        let successors = if graph.has_node(0, node) {
            graph
                .successors(0, node)
                .into_iter()
                .filter(|name| {
                    self.type_map[1]
                        .get(name)
                        .is_some_and(|kind| kind.normalized() == NodeType::Win)
                })
                .collect()
        } else {
            self.graphs.get_move_view_nodes(pos, node, 0, rule, None)
        };
        let head = choose_depth(&successors, &self.depth_map[1], true)?;
        self.get_winning_optimal_move(1, &head, rule)
    }

    pub fn get_losing_optimal_move(
        &self,
        pos: usize,
        node: &str,
        rule: Option<ChangeRule>,
    ) -> Option<SingleMove> {
        let graph = self.graphs.get_graph("winlose");
        if pos == 1 {
            if !self.type_map[1].contains_key(node)
                || self.depth_map[1].get(node).is_some_and(|depth| *depth <= 1)
            {
                return None;
            }
            let successors: Vec<_> = graph
                .successors(1, node)
                .into_iter()
                .filter(|name| self.loop_map.get(node) != Some(name))
                .collect();
            return choose_depth(&successors, &self.depth_map[0], false)
                .map(|tail| (node.to_owned(), tail));
        }
        let (successors, depths, minimum) = if self.type_map[0].contains_key(node) {
            // Preserve the source reducer's use of the position-0 depth map
            // for these head nodes, including its last-on-missing behavior.
            (graph.successors(0, node), &self.depth_map[0], false)
        } else {
            (
                self.graphs.get_move_view_nodes(pos, node, 0, rule, None),
                &self.depth_map[1],
                true,
            )
        };
        let head = choose_depth(&successors, depths, minimum)?;
        self.get_losing_optimal_move(1, &head, rule)
    }

    pub fn get_unremoved_move_type(&self, head: &str, tail: &str) -> MoveInfo {
        let head_type = self.get_node_type(head, 1, None);
        let tail_type = self.get_node_type(tail, 0, None);
        if head_type == NodeType::LoopWin && tail_type == NodeType::LoopWin {
            let designated = self.loop_map.get(head).is_some_and(|name| name == tail);
            return MoveInfo {
                move_type: if designated { 0 } else { 5 },
                depth: self.depth_map[0]
                    .get(tail)
                    .map(|depth| depth + usize::from(!designated)),
                ..MoveInfo::default()
            };
        }
        let move_type = node_types_to_move_type(head_type, tail_type)
            .expect("unremoved word connects an impossible pair of node types");
        let mut info = MoveInfo {
            move_type,
            ..MoveInfo::default()
        };
        if has_depth_map(move_type) {
            info.depth = self.depth_map[0].get(tail).map(|depth| depth + 1);
        } else if move_type == 1 {
            info.connected = Some(
                self.scc_map[1]
                    .get(head)
                    .zip(self.scc_map[0].get(tail))
                    .is_some_and(|(head_id, tail_id)| head_id == tail_id),
            );
        }
        info
    }

    pub fn get_move_type(&self, head: &str, tail: &str, index: usize) -> MoveInfo {
        if index < self.graphs.get_graph("removed").get_edge_num(head, tail) {
            MoveInfo {
                move_type: 2,
                pair: self.pair_manager.get_pair_idx(head, tail, index),
                ..MoveInfo::default()
            }
        } else {
            self.get_unremoved_move_type(head, tail)
        }
    }

    pub fn classify_moves(&self, moves: &EdgeMap<Vec<usize>>) -> MoveClass {
        let mut result: MoveClass = std::array::from_fn(|_| IndexMap::new());
        for (head, tail, indices) in moves.to_array() {
            let node_types = [
                self.get_node_type(&head, 1, None),
                self.get_node_type(&tail, 0, None),
            ];
            for index in indices {
                let info = self.get_move_type(&head, &tail, index);
                // The source groups connected routes under key 0 and
                // disconnected routes under key 1. Card conversion retains
                // the source's inverse interpretation of this key.
                let key = info.depth.unwrap_or_else(|| {
                    info.connected
                        .map_or(0, |connected| usize::from(!connected))
                });
                let map = result[info.move_type].entry(key).or_default();
                let data = map.get_or_insert(
                    &head,
                    &tail,
                    MoveInfoData {
                        node_types,
                        word_idx: Vec::new(),
                        pairs: Vec::new(),
                    },
                );
                data.word_idx.push(index);
                if let Some(pair) = info.pair {
                    data.pairs.push(pair);
                }
            }
        }
        result
    }

    pub fn get_word_type_num(&self) -> WordTypeCounts {
        let mut result = WordTypeCounts {
            type_num: [0; 6],
            subtype_num: (0..6)
                .map(|kind| {
                    move_type_to_node_types(kind)
                        .iter()
                        .map(|&(head, tail)| (head, tail, 0))
                        .collect()
                })
                .collect(),
        };
        result.type_num[2] = self.graphs.get_graph("removed").get_all_move_num();
        for (head, tail, count) in self
            .graphs
            .get_graph("route")
            .edges(1)
            .into_iter()
            .chain(self.graphs.get_graph("winlose").edges(1))
        {
            let kind = self.get_unremoved_move_type(&head, &tail).move_type;
            result.type_num[kind] += count;
            let head_kind = self.get_node_type(&head, 1, None);
            let tail_kind = self.get_node_type(&tail, 0, None);
            if let Some(subtype) = result.subtype_num[kind]
                .iter_mut()
                .find(|pair| pair.0 == head_kind && pair.1 == tail_kind)
            {
                subtype.2 += count;
            }
        }
        result
    }

    pub fn get_comparison_map(&self, other: &Self, rule: ChangeRule) -> ComparisonMap {
        let mut result = [IndexMap::new(), IndexMap::new()];
        for (pos, map) in result.iter_mut().enumerate() {
            let nodes: IndexSet<_> = self.type_map[pos]
                .keys()
                .chain(other.type_map[pos].keys())
                .collect();
            for node in nodes {
                let before = self.get_node_type(node, pos, Some(rule));
                let after = other.get_node_type(node, pos, Some(rule));
                if before != after {
                    map.insert(node.clone(), (before, after));
                }
            }
        }
        result
    }

    pub fn get_changeables_chars_data(&self, node: &str, rule: ChangeRule) -> ChangeableCharsData {
        ChangeableCharsData {
            from: rule
                .backward(node)
                .into_iter()
                .map(|name| {
                    let kind = self.get_node_type(&name, 0, Some(rule));
                    (name, kind)
                })
                .collect(),
            to: rule
                .forward(node)
                .into_iter()
                .map(|name| {
                    let kind = self.get_node_type(&name, 1, Some(rule));
                    (name, kind)
                })
                .collect(),
        }
    }

    pub fn get_max_route_info(&self, view: usize) -> RouteInfo {
        let mut totals: IndexMap<usize, usize> = IndexMap::new();
        for component in self.scc_map.iter().flat_map(|map| map.values()) {
            *totals.entry(*component).or_default() += 1;
        }
        // This threshold counts both graph sides, unlike get_route_nodes.
        let nodes: [IndexSet<String>; 2] = std::array::from_fn(|pos| {
            self.scc_map[pos]
                .iter()
                .filter(|(_, component)| totals[*component] >= 3)
                .map(|(name, _)| name.clone())
                .collect()
        });
        let graph = self.graphs.get_graph("route").get_induced_subgraph(&nodes);
        let char_num = graph.nodes(view).len();
        let move_num = graph.get_all_move_num();
        let average_total: usize = graph
            .nodes(1)
            .iter()
            .map(|head| {
                let out_degree: usize = graph
                    .successors(1, head)
                    .iter()
                    .map(|tail| graph.get_edge_num(head, tail))
                    .sum();
                if view == 0 {
                    graph.in_degree(1, head) * out_degree
                } else {
                    out_degree
                }
            })
            .sum();
        let average_num = if char_num == 0 {
            0.0
        } else {
            ((average_total as f64 / char_num as f64) * 1000.0).round() / 1000.0
        };
        RouteInfo {
            char_num,
            move_num,
            average_num,
        }
    }

    pub fn get_distribution_map(
        &self,
        kind: NodeType,
        view: usize,
        direction: usize,
    ) -> IndexMap<String, [usize; 6]> {
        let mut result: IndexMap<_, _> = self.type_map[view]
            .iter()
            .filter(|(_, value)| **value == kind)
            .map(|(node, _)| (node.clone(), [0; 6]))
            .collect();
        let removed = self
            .graphs
            .get_graph("removed")
            .edges(1)
            .into_iter()
            .map(|edge| (edge, 2));
        let unremoved = self
            .graphs
            .get_graph("route")
            .edges(1)
            .into_iter()
            .chain(self.graphs.get_graph("winlose").edges(1))
            .map(|edge| {
                let kind = self.get_unremoved_move_type(&edge.0, &edge.1).move_type;
                (edge, kind)
            });
        for ((head, tail, count), move_type) in removed.chain(unremoved) {
            let node = if direction == 0 { head } else { tail };
            let targets = if direction ^ view == 1 {
                vec![node]
            } else if direction == 0 {
                self.graphs.predecessors(1 - view, &node)
            } else {
                self.graphs.successors(1 - view, &node)
            };
            // Do not deduplicate partition neighbors: the original uses a
            // concatenation, and its distribution counts include repetitions.
            for target in targets {
                if let Some(row) = result.get_mut(&target) {
                    row[move_type] += count;
                }
            }
        }
        result
    }

    pub fn get_distribution(
        &self,
        kind: NodeType,
        view: usize,
        direction: usize,
        sort: DistributionSort,
        display: DistributionDisplay,
    ) -> Vec<DistributionRow> {
        let mut result: Vec<_> = self
            .get_distribution_map(kind, view, direction)
            .into_iter()
            .map(|(char, counts)| {
                let total = counts.iter().sum::<usize>() as f64;
                let num = counts.map(|count| {
                    if display == DistributionDisplay::Fraction && total > 0.0 {
                        count as f64 / total
                    } else {
                        count as f64
                    }
                });
                DistributionRow { char, num }
            })
            .collect();
        result.sort_by(|left, right| {
            distribution_value(&left.num, sort.key)
                .partial_cmp(&distribution_value(&right.num, sort.key))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if sort.desc {
            result.reverse();
        }
        result
    }

    pub fn get_distribution_with_calc(
        &self,
        kind: NodeType,
        view: usize,
        word_types: [DistributionKey; 2],
        desc: bool,
        calculation: DistributionCalculation,
    ) -> Vec<DistributionCalculationRow> {
        let maps: [IndexMap<String, f64>; 2] = std::array::from_fn(|direction| {
            self.get_distribution_map(kind, view, direction)
                .into_iter()
                .map(|(node, counts)| {
                    (
                        node,
                        distribution_value(
                            &counts.map(|value| value as f64),
                            word_types[direction],
                        ),
                    )
                })
                .collect()
        });
        let mut result: Vec<_> = maps[0]
            .iter()
            .map(|(char, forward)| {
                let backward = maps[1][char];
                let value = match calculation {
                    DistributionCalculation::Ratio => backward / forward,
                    DistributionCalculation::Difference => backward - forward,
                };
                DistributionCalculationRow {
                    char: char.clone(),
                    num: [backward, *forward, value],
                }
            })
            .collect();
        result.sort_by(|left, right| {
            left.num[2]
                .partial_cmp(&right.num[2])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if desc {
            result.reverse();
        }
        result
    }

    pub fn get_scc_data(&self, pos: usize, asc: bool) -> Vec<SccData> {
        let graph = self.graphs.get_graph("route");
        let (condensed, _, mut members) = graph.condensation(&self.scc_map, pos);
        let two_step_count = |node: &str, successor: bool| -> usize {
            let first = if successor {
                graph.successors(pos, node)
            } else {
                graph.predecessors(pos, node)
            };
            first
                .into_iter()
                .map(|next| {
                    if successor {
                        graph.successors(1 - pos, &next).len()
                    } else {
                        graph.predecessors(1 - pos, &next).len()
                    }
                })
                .sum()
        };
        for group in &mut members {
            // The TS comparator returns +1 for equal scores. Stable sorting
            // retains input order on ties, without an invalid Rust comparator.
            group.sort_by_key(|name| {
                std::cmp::Reverse(two_step_count(name, true) + two_step_count(name, false))
            });
        }
        let mut nodes = condensed.sort_by_distance_from_sink();
        if asc {
            nodes.reverse();
        }
        nodes
            .into_iter()
            .map(|component| SccData {
                nodes: members[component].clone(),
                succ: condensed
                    .successors(component)
                    .into_iter()
                    .map(|next| SccSuccessor {
                        nodes: members[next].clone(),
                        by: condensed
                            .get_prop(component, next)
                            .expect("condensation edge property exists")
                            .edges
                            .to_array()
                            .into_iter()
                            .map(|(head, tail, _)| {
                                let range =
                                    self.graphs.get_edge_idx_range(&head, &tail, Some("route"));
                                (head, tail, range)
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect()
    }
}

fn choose_depth(
    nodes: &[String],
    depths: &IndexMap<String, usize>,
    minimum: bool,
) -> Option<String> {
    let mut selected = nodes.first()?.clone();
    for node in &nodes[1..] {
        let keep = depths
            .get(&selected)
            .zip(depths.get(node))
            .is_some_and(|(left, right)| if minimum { left < right } else { left > right });
        if !keep {
            selected = node.clone();
        }
    }
    Some(selected)
}

fn type_fill(kind: NodeType) -> String {
    let name = match kind {
        NodeType::Win => "win",
        NodeType::Lose => "lose",
        NodeType::LoopWin => "loopwin",
        NodeType::Route => "route",
    };
    format!("var(--color-{name})")
}

fn distribution_value(values: &[f64; 6], key: DistributionKey) -> f64 {
    match key {
        DistributionKey::Total => values.iter().sum(),
        DistributionKey::Move(index) => values.get(index).copied().unwrap_or(f64::NAN),
    }
}

pub fn has_depth_map(kind: MoveType) -> bool {
    matches!(kind, 0 | 3 | 5)
}

pub fn node_types_to_move_type(head: NodeType, tail: NodeType) -> Option<MoveType> {
    use NodeType::*;
    match (head, tail) {
        (Win, Lose) | (LoopWin, LoopWin) => Some(0),
        (Route, Route) => Some(1),
        (Lose, Win | LoopWin) => Some(3),
        (Win, Route) => Some(4),
        (Win | Route | LoopWin, Win | LoopWin) => Some(5),
        _ => None,
    }
}

pub fn move_type_to_node_types(kind: MoveType) -> &'static [(NodeType, NodeType)] {
    use NodeType::*;
    match kind {
        0 => &[(Win, Lose), (LoopWin, LoopWin)],
        1 => &[(Route, Route)],
        2 => &[],
        3 => &[(Lose, Win), (Lose, LoopWin)],
        4 => &[(Win, Route)],
        5 => &[
            (Route, Win),
            (Route, LoopWin),
            (Win, Win),
            (Win, LoopWin),
            (LoopWin, Win),
            (LoopWin, LoopWin),
        ],
        _ => &[],
    }
}

pub fn get_comparison_data(mapping: &ComparisonMap) -> ComparisonData {
    let mut result: ComparisonData = [Vec::new(), Vec::new()];
    let kinds = [
        NodeType::Win,
        NodeType::Lose,
        NodeType::LoopWin,
        NodeType::Route,
    ];
    for pos in 0..2 {
        for before in kinds {
            for after in kinds {
                if before == after {
                    continue;
                }
                let nodes = mapping[pos]
                    .iter()
                    .filter(|(_, pair)| **pair == (before, after))
                    .map(|(node, _)| node.clone())
                    .collect();
                result[pos].push((before, after, nodes));
            }
        }
    }
    result
}

pub fn sort_edges(edges: &mut [Edge]) {
    edges.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
}
