use crate::{edge_map::EdgeMap, rules::ChangeRule, types::*};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

pub type Condensation = (
    DiGraph<usize, CondensationProp>,
    IndexMap<String, usize>,
    Vec<Vec<String>>,
);
use std::{cmp::Ordering, collections::VecDeque, hash::Hash};

/// Alternating transformation (position 0) and word (position 1) edges.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BipartiteDiGraph {
    pub _nodes: [IndexSet<String>; 2],
    pub _succ: [EdgeMap<usize>; 2],
    pub _pred: [EdgeMap<usize>; 2],
}

pub fn get_oppos(pos: NodePos) -> NodePos {
    1 - pos
}

impl BipartiteDiGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_word_map(map: &EdgeMap<Vec<String>>, rule: ChangeRule) -> Self {
        let mut graph = Self::new();
        for (head, tail, words) in map.to_array() {
            if !words.is_empty() {
                graph.set_edge(1, &head, &tail, words.len());
            }
        }
        for node in graph.nodes(0) {
            for next in rule.forward(&node) {
                if graph.has_node(1, &next) {
                    graph.set_edge(0, &node, &next, 1);
                }
            }
        }
        graph
    }
    pub fn nodes(&self, pos: NodePos) -> Vec<String> {
        self._nodes[pos].iter().cloned().collect()
    }
    pub fn edges(&self, pos: NodePos) -> Vec<Edge> {
        self._succ[pos].to_array()
    }
    pub fn has_node(&self, pos: NodePos, node: &str) -> bool {
        self._nodes[pos].contains(node)
    }
    pub fn add_node(&mut self, pos: NodePos, node: &str) {
        self._nodes[pos].insert(node.into());
    }
    pub fn has_edge(&self, pos: NodePos, start: &str, end: &str) -> bool {
        self._succ[pos].get_num(start, end) != 0
    }
    pub fn get_edge_num(&self, start: &str, end: &str) -> usize {
        self._succ[1].get_num(start, end)
    }

    pub fn set_edge(&mut self, pos: NodePos, start: &str, end: &str, num: usize) {
        let oppos = get_oppos(pos);
        self.add_node(pos, start);
        self.add_node(oppos, end);
        self._succ[pos].set(start, end, num);
        self._pred[oppos].set(end, start, num);
    }
    pub fn increase_edge(&mut self, start: &str, end: &str, num: usize) {
        self.add_node(1, start);
        self.add_node(0, end);
        if num > 0 {
            self._succ[1].increase(start, end, num);
            self._pred[0].increase(end, start, num);
        }
    }
    pub fn decrease_edge(&mut self, start: &str, end: &str, num: usize) -> crate::Result<()> {
        if start == "__none" {
            return Ok(());
        }
        if self._succ[1].get_num(start, end) < num || self._pred[0].get_num(end, start) < num {
            return Err(crate::Error::InvalidInput(
                "cannot decrease edge to less than zero".into(),
            ));
        }
        self._succ[1].decrease(start, end, num)?;
        self._pred[0].decrease(end, start, num)?;
        Ok(())
    }
    pub fn remove_edge(&mut self, pos: NodePos, start: &str, end: &str) {
        self._succ[pos].remove(start, Some(end));
        self._pred[get_oppos(pos)].remove(end, Some(start));
    }
    /// Returns removed edges grouped by their source position.
    pub fn remove_node(&mut self, pos: NodePos, node: &str) -> [Vec<Edge>; 2] {
        let mut edges = [Vec::new(), Vec::new()];
        if !self.has_node(pos, node) {
            return edges;
        }
        let oppos = get_oppos(pos);
        for (succ, count) in self._succ[pos].get_succ(node) {
            edges[pos].push((node.into(), succ.clone(), count));
            self._pred[oppos].remove(&succ, Some(node));
        }
        self._succ[pos].remove(node, None);
        for (pred, count) in self._pred[pos].get_succ(node) {
            edges[oppos].push((pred.clone(), node.into(), count));
            self._succ[oppos].remove(&pred, Some(node));
        }
        self._pred[pos].remove(node, None);
        self._nodes[pos].shift_remove(node);
        edges
    }
    pub fn successors(&self, pos: NodePos, node: &str) -> Vec<String> {
        self._succ[pos]
            .content
            .get(node)
            .map(|inner| inner.keys().cloned().collect())
            .unwrap_or_default()
    }
    pub fn predecessors(&self, pos: NodePos, node: &str) -> Vec<String> {
        self._pred[pos]
            .content
            .get(node)
            .map(|inner| inner.keys().cloned().collect())
            .unwrap_or_default()
    }
    /// Degree counts neighbors; multiplicity is available through get_edge_num.
    pub fn out_degree(&self, pos: NodePos, node: &str) -> usize {
        self._succ[pos].content.get(node).map_or(0, IndexMap::len)
    }
    pub fn in_degree(&self, pos: NodePos, node: &str) -> usize {
        self._pred[pos].content.get(node).map_or(0, IndexMap::len)
    }
    pub fn get_out_degree_map(&self) -> NodeMap<usize> {
        std::array::from_fn(|pos| {
            self._nodes[pos]
                .iter()
                .map(|node| (node.clone(), self.out_degree(pos, node)))
                .collect()
        })
    }
    pub fn get_single_out_edges(&self) -> Vec<SingleMove> {
        self.nodes(0)
            .into_iter()
            .filter_map(|node| {
                let succ = self.successors(0, &node);
                (succ.len() == 1).then(|| (node, succ[0].clone()))
            })
            .collect()
    }
    pub fn get_single_in_edges(&self) -> Vec<SingleMove> {
        self.nodes(1)
            .into_iter()
            .filter_map(|node| {
                let pred = self.predecessors(1, &node);
                (pred.len() == 1).then(|| (node, pred[0].clone()))
            })
            .collect()
    }
    pub fn get_single_in_out_edges(&self) -> Vec<SingleMove> {
        let mut edges = EdgeMap::new();
        for (start, end) in self
            .get_single_in_edges()
            .into_iter()
            .chain(self.get_single_out_edges())
        {
            edges.set(&start, &end, ());
        }
        edges
            .to_array()
            .into_iter()
            .map(|(start, end, ())| (start, end))
            .collect()
    }
    pub fn get_move_view_nodes(
        &self,
        pos: NodePos,
        node: &str,
        direction: NodePos,
        rule: Option<ChangeRule>,
    ) -> Vec<String> {
        if self.has_node(pos, node) {
            if pos + direction == 1 {
                vec![node.into()]
            } else if direction == 0 {
                self.successors(0, node)
            } else {
                self.predecessors(1, node)
            }
        } else if pos ^ direction == 1 {
            vec![]
        } else if let Some(rule) = rule {
            if direction == 0 {
                rule.forward(node)
                    .into_iter()
                    .filter(|next| self.has_node(1, next))
                    .collect()
            } else {
                rule.backward(node)
                    .into_iter()
                    .filter(|next| self.has_node(0, next))
                    .collect()
            }
        } else {
            vec![]
        }
    }
    pub fn get_moves_from_node(
        &self,
        node: &str,
        view: NodePos,
        direction: NodePos,
        rule: Option<ChangeRule>,
    ) -> Vec<SingleMove> {
        self.get_move_view_nodes(direction ^ view, node, direction, rule)
            .into_iter()
            .flat_map(|node| {
                if direction == 0 {
                    self.successors(1, &node)
                        .into_iter()
                        .map(|end| (node.clone(), end))
                        .collect::<Vec<_>>()
                } else {
                    self.predecessors(0, &node)
                        .into_iter()
                        .map(|start| (start, node.clone()))
                        .collect()
                }
            })
            .collect()
    }
    pub fn get_reachable_nodes(
        &self,
        pos: NodePos,
        node: &str,
        rule: Option<ChangeRule>,
    ) -> [IndexSet<String>; 2] {
        let mut visited: [IndexSet<String>; 2] = Default::default();
        visited[pos].insert(node.into());
        let mut stack = vec![(pos, node.to_owned())];
        while let Some((pos, node)) = stack.pop() {
            let oppos = get_oppos(pos);
            let successors = if self.has_node(pos, &node) {
                self.successors(pos, &node)
            } else {
                rule.map(|rule| {
                    rule.forward(&node)
                        .into_iter()
                        .filter(|next| self.has_node(oppos, next))
                        .collect()
                })
                .unwrap_or_default()
            };
            for next in successors {
                if visited[oppos].insert(next.clone()) {
                    stack.push((oppos, next));
                }
            }
        }
        visited
    }
    pub fn get_induced_subgraph(&self, nodes: &[IndexSet<String>; 2]) -> Self {
        let mut graph = Self::new();
        for pos in 0..2 {
            for node in &nodes[pos] {
                graph.add_node(pos, node);
                for end in self.successors(pos, node) {
                    if nodes[get_oppos(pos)].contains(&end) {
                        graph.set_edge(
                            pos,
                            node,
                            &end,
                            if pos == 0 {
                                1
                            } else {
                                self.get_edge_num(node, &end)
                            },
                        );
                    }
                }
            }
        }
        graph
    }
    pub fn get_reachable_graph(&self, pos: NodePos, node: &str, rule: Option<ChangeRule>) -> Self {
        self.get_induced_subgraph(&self.get_reachable_nodes(pos, node, rule))
    }
    /// Tarjan traversal uses explicit frames to handle large dictionaries.
    pub fn get_scc(&self) -> Vec<[Vec<String>; 2]> {
        struct Frame {
            pos: usize,
            node: String,
            successors: Vec<String>,
            next: usize,
        }
        let mut discovery: NodeMap<usize> = Default::default();
        let mut low: NodeMap<usize> = Default::default();
        let mut active: [IndexSet<String>; 2] = Default::default();
        let mut stack: Vec<(usize, String)> = Vec::new();
        let mut components = Vec::new();
        let mut id = 0;
        for pos in 0..2 {
            for node in self.nodes(pos) {
                if discovery[pos].contains_key(&node) {
                    continue;
                }
                id += 1;
                discovery[pos].insert(node.clone(), id);
                low[pos].insert(node.clone(), id);
                active[pos].insert(node.clone());
                stack.push((pos, node.clone()));
                let mut frames = vec![Frame {
                    pos,
                    successors: self.successors(pos, &node),
                    node,
                    next: 0,
                }];
                while let Some(frame) = frames.last_mut() {
                    let current_pos = frame.pos;
                    let current_node = frame.node.clone();
                    if let Some(next) = frame.successors.get(frame.next).cloned() {
                        frame.next += 1;
                        let next_pos = get_oppos(current_pos);
                        if !discovery[next_pos].contains_key(&next) {
                            id += 1;
                            discovery[next_pos].insert(next.clone(), id);
                            low[next_pos].insert(next.clone(), id);
                            active[next_pos].insert(next.clone());
                            stack.push((next_pos, next.clone()));
                            frames.push(Frame {
                                pos: next_pos,
                                successors: self.successors(next_pos, &next),
                                node: next,
                                next: 0,
                            });
                        } else if active[next_pos].contains(&next) {
                            low[current_pos][&current_node] =
                                low[current_pos][&current_node].min(discovery[next_pos][&next]);
                        }
                    } else {
                        let frame = frames.pop().unwrap();
                        let value = low[frame.pos][&frame.node];
                        if value == discovery[frame.pos][&frame.node] {
                            let mut component: [Vec<String>; 2] = Default::default();
                            while let Some((member_pos, member)) = stack.pop() {
                                active[member_pos].shift_remove(&member);
                                let done = member_pos == frame.pos && member == frame.node;
                                component[member_pos].push(member);
                                if done {
                                    break;
                                }
                            }
                            components.push(component);
                        }
                        if let Some(parent) = frames.last() {
                            low[parent.pos][&parent.node] =
                                low[parent.pos][&parent.node].min(value);
                        }
                    }
                }
            }
        }
        components
    }
    pub fn get_scc_map(&self) -> NodeMap<usize> {
        let mut map: NodeMap<usize> = Default::default();
        for (id, component) in self.get_scc().into_iter().enumerate() {
            for (pos, members) in component.into_iter().enumerate() {
                for member in members {
                    map[pos].insert(member, id);
                }
            }
        }
        map
    }
    pub fn get_two_paths(&self, pos: NodePos) -> Vec<(String, String, String)> {
        let mut paths = Vec::new();
        for start in self.nodes(pos) {
            for middle in self.successors(pos, &start) {
                for end in self.successors(get_oppos(pos), &middle) {
                    paths.push((start.clone(), middle.clone(), end));
                }
            }
        }
        paths
    }
    pub fn condensation(&self, scc: &NodeMap<usize>, pos: NodePos) -> Condensation {
        let mut mapping = IndexMap::new();
        let mut members: Vec<Vec<String>> = Vec::new();
        let mut compact = IndexMap::new();
        for (node, id) in &scc[pos] {
            let next = compact.len();
            let index = *compact.entry(*id).or_insert(next);
            if index == members.len() {
                members.push(Vec::new());
            }
            members[index].push(node.clone());
            mapping.insert(node.clone(), index);
        }
        let mut graph = DiGraph::new();
        for (start, middle, end) in self.get_two_paths(pos) {
            if let (Some(&from), Some(&to)) = (mapping.get(&start), mapping.get(&end)) {
                if from == to {
                    continue;
                }
                if !graph.has_edge(from, to) {
                    graph.add_edge(from, to, CondensationProp::default());
                }
                let prop = graph.get_prop_mut(from, to).unwrap();
                if pos == 0 {
                    prop.edges.increase(&middle, &end, 1);
                } else {
                    prop.edges.increase(&start, &middle, 1);
                }
            }
        }
        (graph, mapping, members)
    }
    pub fn get_even_loops(&self) -> Vec<Edge> {
        self.get_single_in_out_edges()
            .into_iter()
            .filter_map(|(start, end)| {
                let num = self.get_edge_num(&end, &start);
                (num >= 2).then(|| (end, start, num - num % 2))
            })
            .collect()
    }
    pub fn get_two_cycles(&self) -> Vec<(SingleMove, SingleMove, usize)> {
        let singles = self.get_single_in_out_edges();
        let mut consumed = EdgeMap::<usize>::new();
        let mut cycles = Vec::new();
        for (i, (head1, tail1)) in singles.iter().enumerate() {
            for (head2, tail2) in singles.iter().skip(i + 1) {
                let num1 = self
                    .get_edge_num(tail1, head2)
                    .saturating_sub(consumed.get_num(tail1, head2));
                let num2 = self
                    .get_edge_num(tail2, head1)
                    .saturating_sub(consumed.get_num(tail2, head1));
                if num1 > 0 && num2 > 0 {
                    let amount = num1.min(num2);
                    consumed.increase(tail1, head2, amount);
                    consumed.increase(tail2, head1, amount);
                    cycles.push((
                        (tail1.clone(), head2.clone()),
                        (tail2.clone(), head1.clone()),
                        amount,
                    ));
                }
            }
        }
        cycles
    }
    pub fn get_critical_edges(&self) -> Vec<SingleMove> {
        let chars: IndexSet<String> = self
            .get_single_out_edges()
            .into_iter()
            .map(|(_, end)| end)
            .collect();
        let mut result = Vec::new();
        for node in chars {
            let succ = self.successors(1, &node);
            if succ.len() == 1 && self.get_edge_num(&node, &succ[0]) == 1 {
                result.push((node, succ[0].clone()));
            } else if succ.len() == 2
                && self.get_edge_num(&node, &succ[0]) == 1
                && self.get_edge_num(&node, &succ[1]) == 1
            {
                if self.has_edge(0, &succ[0], &node) {
                    result.push((node.clone(), succ[1].clone()));
                }
                if self.has_edge(0, &succ[1], &node) {
                    result.push((node.clone(), succ[0].clone()));
                }
            }
        }
        result
    }
    pub fn get_sinks(&self, pos: NodePos) -> Vec<String> {
        self.nodes(pos)
            .into_iter()
            .filter(|node| self.out_degree(pos, node) == 0)
            .collect()
    }
    pub fn get_sources(&self, pos: NodePos) -> Vec<String> {
        self.nodes(pos)
            .into_iter()
            .filter(|node| self.in_degree(pos, node) == 0)
            .collect()
    }
    pub fn get_hanbang_nodes(&self) -> Vec<String> {
        self.get_sinks(0)
    }
    pub fn get_hanbang_moves(&self) -> Vec<Edge> {
        self.get_hanbang_nodes()
            .into_iter()
            .flat_map(|end| {
                self.predecessors(0, &end)
                    .into_iter()
                    .map(|start| (start.clone(), end.clone(), self.get_edge_num(&start, &end)))
                    .collect::<Vec<_>>()
            })
            .collect()
    }
    pub fn get_all_move_num(&self) -> usize {
        self.edges(1).iter().map(|edge| edge.2).sum()
    }
    pub fn is_empty(&self) -> bool {
        self._succ[1].content.is_empty()
    }
    pub fn shortest_distance_to_any_target(
        &self,
        start_pos: NodePos,
        start_node: &str,
        target_pos: NodePos,
        targets: &IndexSet<String>,
    ) -> f64 {
        let mut visited: [IndexSet<String>; 2] = Default::default();
        let mut queue = VecDeque::from([(start_pos, start_node.to_owned(), 0usize)]);
        visited[start_pos].insert(start_node.into());
        while let Some((pos, node, distance)) = queue.pop_front() {
            if pos == target_pos && targets.contains(&node) {
                return distance as f64;
            }
            let oppos = get_oppos(pos);
            for next in self.successors(pos, &node) {
                if visited[oppos].insert(next.clone()) {
                    queue.push_back((oppos, next, distance + 1));
                }
            }
        }
        f64::INFINITY
    }
    pub fn compare_next_move_num(
        &self,
        first: &SingleMove,
        second: &SingleMove,
        prec: &PrecInfo,
    ) -> Ordering {
        let edge_score = |value: &SingleMove| {
            prec.maps
                .edge
                .get(&value.0)
                .and_then(|inner| inner.get(&value.1))
                .copied()
                .unwrap_or(0.0)
        };
        let left = edge_score(first);
        let right = edge_score(second);
        if left != right {
            return left.partial_cmp(&right).unwrap_or(Ordering::Equal);
        }
        let node_score = |value: &SingleMove| prec.maps.node.get(&value.1).copied().unwrap_or(0.0);
        let left = node_score(first);
        let right = node_score(second);
        if left != right {
            return left.partial_cmp(&right).unwrap_or(Ordering::Equal);
        }
        self.move_score(first, prec.rule)
            .partial_cmp(&self.move_score(second, prec.rule))
            .unwrap_or(Ordering::Equal)
    }
    fn move_score(&self, move_: &SingleMove, rule: usize) -> f64 {
        let next = self.get_moves_from_node(&move_.1, 0, 0, None);
        let prev = self.get_moves_from_node(&move_.1, 0, 1, None);
        let next_num = || {
            next.iter()
                .map(|(start, end)| self.get_edge_num(start, end) as f64)
                .sum::<f64>()
        };
        let prev_num = || {
            prev.iter()
                .map(|(start, end)| self.get_edge_num(start, end) as f64)
                .sum::<f64>()
        };
        match rule {
            0 => {
                next_num() * 1000.0
                    - prev.len() as f64
                    - if self.is_trap_situation(&move_.0, &move_.1) {
                        1_000_000.0
                    } else {
                        0.0
                    }
            }
            1 => prev_num(),
            2 => next_num() - prev_num(),
            3 => next_num() / prev_num(),
            4 => {
                let targets: IndexSet<String> = self
                    .get_critical_edges()
                    .into_iter()
                    .map(|edge| edge.0)
                    .collect();
                if targets.is_empty() {
                    next_num() * 1000.0 - prev.len() as f64
                } else {
                    self.shortest_distance_to_any_target(0, &move_.1, 1, &targets)
                }
            }
            5 => next.len() as f64 * 1000.0 - prev.len() as f64,
            _ => next.len() as f64 / prev.len() as f64,
        }
    }
    fn is_trap_situation(&self, first: &str, last: &str) -> bool {
        if first == "__none" {
            return false;
        }
        let succ = self.successors(1, first);
        succ.len() == 2
            && self.get_edge_num(first, &succ[0]) == 1
            && self.get_edge_num(first, &succ[1]) == 1
            && self.has_edge(0, last, first)
    }
    pub fn next_words_limit_nodes(&self, pos: NodePos, limit: usize) -> Vec<String> {
        self.nodes(pos)
            .into_iter()
            .filter(|node| {
                self.get_move_view_nodes(pos, node, 0, None)
                    .iter()
                    .map(|head| {
                        self.successors(1, head)
                            .iter()
                            .map(|tail| self.get_edge_num(head, tail))
                            .sum::<usize>()
                    })
                    .sum::<usize>()
                    < limit
            })
            .collect()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CondensationProp {
    pub edges: EdgeMap<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiGraph<K: Eq + Hash, P> {
    pub _succ: IndexMap<K, IndexSet<K>>,
    pub _pred: IndexMap<K, IndexSet<K>>,
    pub _prop: IndexMap<K, IndexMap<K, P>>,
}
impl<K: Eq + Hash, P> Default for DiGraph<K, P> {
    fn default() -> Self {
        Self {
            _succ: IndexMap::new(),
            _pred: IndexMap::new(),
            _prop: IndexMap::new(),
        }
    }
}
impl<K: Clone + Eq + Hash, P> DiGraph<K, P> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn nodes(&self) -> Vec<K> {
        self._succ.keys().cloned().collect()
    }
    pub fn edges(&self) -> Vec<(K, K)> {
        self._succ
            .iter()
            .flat_map(|(start, ends)| ends.iter().map(move |end| (start.clone(), end.clone())))
            .collect()
    }
    pub fn has_node(&self, node: K) -> bool {
        self._succ.contains_key(&node)
    }
    pub fn add_node(&mut self, node: K) {
        if !self._succ.contains_key(&node) {
            self._succ.insert(node.clone(), IndexSet::new());
            self._pred.insert(node.clone(), IndexSet::new());
            self._prop.insert(node, IndexMap::new());
        }
    }
    pub fn has_edge(&self, start: K, end: K) -> bool {
        self._succ
            .get(&start)
            .is_some_and(|ends| ends.contains(&end))
    }
    pub fn add_edge(&mut self, start: K, end: K, prop: P) {
        self.add_node(start.clone());
        self.add_node(end.clone());
        self._succ[&start].insert(end.clone());
        self._pred[&end].insert(start.clone());
        self._prop[&start].insert(end, prop);
    }
    pub fn get_prop(&self, start: K, end: K) -> Option<&P> {
        self._prop.get(&start)?.get(&end)
    }
    pub fn get_prop_mut(&mut self, start: K, end: K) -> Option<&mut P> {
        self._prop.get_mut(&start)?.get_mut(&end)
    }
    pub fn successors(&self, node: K) -> Vec<K> {
        self._succ
            .get(&node)
            .map(|nodes| nodes.iter().cloned().collect())
            .unwrap_or_default()
    }
    pub fn predecessor(&self, node: K) -> Vec<K> {
        self._pred
            .get(&node)
            .map(|nodes| nodes.iter().cloned().collect())
            .unwrap_or_default()
    }
    pub fn sort_by_distance_from_sink(&self) -> Vec<K> {
        let mut distance: IndexMap<K, usize> =
            self.nodes().into_iter().map(|node| (node, 0)).collect();
        let mut remaining: IndexMap<K, usize> = self
            ._succ
            .iter()
            .filter(|(_, edges)| !edges.is_empty())
            .map(|(node, edges)| (node.clone(), edges.len()))
            .collect();
        let mut queue: VecDeque<K> = self
            .nodes()
            .into_iter()
            .filter(|node| !remaining.contains_key(node))
            .collect();
        while let Some(node) = queue.pop_front() {
            let d = distance[&node];
            for pred in self.predecessor(node) {
                distance[&pred] = distance[&pred].max(d + 1);
                remaining[&pred] -= 1;
                if remaining[&pred] == 0 {
                    queue.push_back(pred);
                }
            }
        }
        let mut nodes = self.nodes();
        nodes.sort_by(|a, b| distance[b].cmp(&distance[a]));
        nodes
    }
}
