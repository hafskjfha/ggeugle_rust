use crate::{edge_map::EdgeMap, graph::BipartiteDiGraph, rules::ChangeRule, types::*};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GraphPartitions {
    pub keys: Vec<String>,
    pub content: IndexMap<String, BipartiteDiGraph>,
}
impl GraphPartitions {
    pub fn new(graph: BipartiteDiGraph) -> Self {
        let content = IndexMap::from([
            ("route".into(), graph),
            ("winlose".into(), BipartiteDiGraph::new()),
            ("removed".into(), BipartiteDiGraph::new()),
        ]);
        Self {
            keys: vec!["removed".into(), "winlose".into(), "route".into()],
            content,
        }
    }
    pub fn from_graphs(content: IndexMap<String, BipartiteDiGraph>, keys: Vec<String>) -> Self {
        Self { content, keys }
    }
    pub fn get_graph(&self, key: &str) -> &BipartiteDiGraph {
        &self.content[key]
    }
    pub fn get_graph_mut(&mut self, key: &str) -> &mut BipartiteDiGraph {
        self.content
            .get_mut(key)
            .expect("key not in graph partition")
    }
    pub fn get_keys(&self) -> &[String] {
        &self.keys
    }
    pub fn transfer_node(&mut self, from: &str, to: &str, pos: NodePos, node: &str) {
        let edges = self.get_graph_mut(from).remove_node(pos, node);
        for (pos, edges) in edges.into_iter().enumerate() {
            for (start, end, num) in edges {
                self.get_graph_mut(to).set_edge(pos, &start, &end, num);
            }
        }
    }
    pub fn transfer_edge(
        &mut self,
        from: &str,
        to: &str,
        start: &str,
        end: &str,
        num: usize,
    ) -> crate::Result<()> {
        if !self.content.contains_key(from) || !self.content.contains_key(to) {
            return Err(crate::Error::InvalidInput(
                "key not in graph partition".into(),
            ));
        }
        self.get_graph_mut(from).decrease_edge(start, end, num)?;
        self.get_graph_mut(to).increase_edge(start, end, num);
        Ok(())
    }
    pub fn successors(&self, pos: NodePos, node: &str) -> Vec<String> {
        self.content
            .values()
            .filter(|graph| graph.has_node(pos, node))
            .flat_map(|graph| graph.successors(pos, node))
            .collect()
    }
    pub fn predecessors(&self, pos: NodePos, node: &str) -> Vec<String> {
        self.content
            .values()
            .filter(|graph| graph.has_node(pos, node))
            .flat_map(|graph| graph.predecessors(pos, node))
            .collect()
    }
    fn selected_keys<'a>(&'a self, keys: Option<&'a [&'a str]>) -> Vec<&'a str> {
        keys.map(<[&str]>::to_vec)
            .unwrap_or_else(|| self.keys.iter().map(String::as_str).collect())
    }
    pub fn get_move_view_nodes(
        &self,
        pos: NodePos,
        node: &str,
        direction: NodePos,
        rule: Option<ChangeRule>,
        keys: Option<&[&str]>,
    ) -> Vec<String> {
        let nodes: IndexSet<String> = self
            .selected_keys(keys)
            .into_iter()
            .flat_map(|key| {
                self.get_graph(key)
                    .get_move_view_nodes(pos, node, direction, rule)
            })
            .collect();
        nodes.into_iter().collect()
    }
    pub fn get_moves_from_node(
        &self,
        node: &str,
        view: NodePos,
        direction: NodePos,
        rule: Option<ChangeRule>,
        keys: Option<&[&str]>,
    ) -> Vec<SingleMove> {
        let move_nodes = self.get_move_view_nodes(view, node, direction, rule, keys);
        let mut counter = EdgeMap::<usize>::new();
        for key in self.selected_keys(keys) {
            for node in &move_nodes {
                for (start, end) in self
                    .get_graph(key)
                    .get_moves_from_node(node, 1, direction, rule)
                {
                    counter.increase(&start, &end, 1);
                }
            }
        }
        counter
            .to_array()
            .into_iter()
            .map(|(start, end, _)| (start, end))
            .collect()
    }
    pub fn get_edge_num_and_offset(
        &self,
        start: &str,
        end: &str,
        key: Option<&str>,
    ) -> (usize, usize) {
        let mut offset = 0;
        for current in &self.keys {
            let num = self.get_graph(current).get_edge_num(start, end);
            if key == Some(current.as_str()) {
                return (num, offset);
            }
            offset += num;
        }
        if key.is_none() {
            (offset, 0)
        } else {
            panic!("key not in graph partition")
        }
    }
    pub fn get_edge_idx_range(&self, start: &str, end: &str, key: Option<&str>) -> (usize, usize) {
        let (num, offset) = self.get_edge_num_and_offset(start, end, key);
        (offset, offset + num)
    }
    pub fn union(&self) -> BipartiteDiGraph {
        let mut result = BipartiteDiGraph::new();
        for graph in self.content.values() {
            for (start, end, _) in graph.edges(0) {
                result.set_edge(0, &start, &end, 1);
            }
            for (start, end, num) in graph.edges(1) {
                result.increase_edge(&start, &end, num);
            }
        }
        result
    }
    pub fn update_union(&mut self, to: &str) -> &BipartiteDiGraph {
        let keys = self.keys.clone();
        for key in keys {
            if key == to {
                continue;
            }
            let transformations = self.get_graph(&key).edges(0);
            let words = self.get_graph(&key).edges(1);
            for (start, end, _) in transformations {
                self.get_graph_mut(to).set_edge(0, &start, &end, 1);
            }
            for (start, end, num) in words {
                self.get_graph_mut(to).increase_edge(&start, &end, num);
            }
        }
        self.get_graph(to)
    }
    pub fn is_empty(&self) -> bool {
        self.content.values().all(BipartiteDiGraph::is_empty)
    }
}
