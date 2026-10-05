use crate::{
    Result,
    edge_map::EdgeMap,
    graph::BipartiteDiGraph,
    rules::{ChangeRule, get_head_tail},
    solver::GraphSolver,
    types::*,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub type WordMap = EdgeMap<Vec<String>>;

impl EdgeMap<Vec<String>> {
    pub fn from_words(words: &[String], head: isize, tail: isize) -> Result<Self> {
        let mut result = Self::new();
        for word in words {
            result.add_word(word, head, tail)?;
        }
        Ok(result)
    }
    pub fn add_word(&mut self, word: &str, head: isize, tail: isize) -> Result<()> {
        let (start, end) = get_head_tail(word, head, tail)?;
        self.get_or_insert(&start, &end, vec![]).push(word.into());
        Ok(())
    }
    pub fn remove_word(&mut self, head: &str, tail: &str, word: &str) {
        if let Some(words) = self.content.get_mut(head).and_then(|m| m.get_mut(tail)) {
            if let Some(idx) = words.iter().position(|w| w == word) {
                words.remove(idx);
            }
            if words.is_empty() {
                self.remove(head, Some(tail));
            }
        }
    }
    pub fn get_move(&self, word: &str, head: isize, tail: isize) -> Result<Option<Edge>> {
        let (start, end) = get_head_tail(word, head, tail)?;
        Ok(self
            .get(&start, &end)
            .and_then(|words| words.iter().position(|w| w == word))
            .map(|idx| (start, end, idx)))
    }
    pub fn has_word(&self, word: &str, head: isize, tail: isize) -> Result<bool> {
        Ok(self.get_move(word, head, tail)?.is_some())
    }
    pub fn get_word(&self, head: &str, tail: &str, idx: usize) -> Option<&str> {
        self.get(head, tail)?.get(idx).map(String::as_str)
    }
    pub fn get_all_words(&self) -> Vec<String> {
        self.content
            .values()
            .flat_map(|m| m.values())
            .flatten()
            .cloned()
            .collect()
    }
    pub fn get_size(&self) -> usize {
        self.content
            .values()
            .flat_map(|m| m.values())
            .map(Vec::len)
            .sum()
    }
    pub fn get_moves(&self, mut filter: impl FnMut(&str) -> bool) -> EdgeMap<Vec<usize>> {
        let mut result = EdgeMap::new();
        for (start, inner) in &self.content {
            for (end, words) in inner {
                for (i, word) in words.iter().enumerate() {
                    if filter(word) {
                        result.get_or_insert(start, end, vec![]).push(i);
                    }
                }
            }
        }
        result
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordSolver {
    pub graph_solver: GraphSolver,
    pub word_map: WordMap,
    pub head_idx: isize,
    pub tail_idx: isize,
    pub flow: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveRow {
    pub r#move: SingleMove,
    pub node_types: [NodeType; 2],
    pub words: Vec<String>,
    pub pairs: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordsCard {
    pub move_type: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected: Option<bool>,
    pub move_rows: Vec<MoveRow>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharWords<T> {
    pub r#char: String,
    pub words: T,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepthWords {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,
    pub words: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharInfo {
    pub r#char: String,
    pub info: Vec<DepthWords>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EssentialWord {
    pub r#char: String,
    pub word: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordSccData {
    pub nodes: Vec<String>,
    pub succ: Vec<WordSccSuccessor>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordSccSuccessor {
    pub nodes: Vec<String>,
    pub by: Vec<String>,
}

impl WordSolver {
    pub fn new(
        graph: BipartiteDiGraph,
        word_map: WordMap,
        head_idx: isize,
        tail_idx: isize,
        flow: usize,
    ) -> Self {
        Self {
            graph_solver: GraphSolver::new(graph, flow),
            word_map,
            head_idx,
            tail_idx,
            flow,
        }
    }
    pub fn get_next_words(&self, history: &[String]) -> Result<Vec<String>> {
        if history.is_empty() {
            return Ok(self.word_map.get_all_words());
        }
        let (_, tail) = get_head_tail(history.last().unwrap(), self.head_idx, self.tail_idx)?;
        let moves = self
            .graph_solver
            .graphs
            .get_moves_from_node(&tail, 0, 0, None, None);
        Ok(moves
            .iter()
            .flat_map(|(start, end)| self.word_map.get(start, end).into_iter().flatten())
            .filter(|word| !history.contains(word))
            .cloned()
            .collect())
    }
    pub fn after_history(&self, history: &[String], flow: usize) -> Result<GraphSolver> {
        let mut partitions = self.graph_solver.graphs.clone();
        let mut graph = partitions.update_union("winlose").clone();
        let mut seen = std::collections::HashSet::new();
        for word in history {
            if seen.insert(word) && self.word_map.has_word(word, self.head_idx, self.tail_idx)? {
                let (start, end) = get_head_tail(word, self.head_idx, self.tail_idx)?;
                graph.decrease_edge(&start, &end, 1)?;
            }
        }
        Ok(GraphSolver::new(graph, flow))
    }
    fn words_in_partition(&self, start: &str, end: &str, key: &str) -> Vec<String> {
        let (begin, end_idx) = self
            .graph_solver
            .graphs
            .get_edge_idx_range(start, end, Some(key));
        (begin..end_idx)
            .filter_map(|idx| self.word_map.get_word(start, end, idx).map(str::to_owned))
            .collect()
    }
    pub fn move_class_to_words_cards(&self, class: crate::solver::MoveClass) -> Vec<WordsCard> {
        let mut result = vec![];
        for (move_type, groups) in class.into_iter().enumerate() {
            let has_depth = [0, 3, 5].contains(&move_type);
            let mut cards = vec![];
            for (group, map) in groups {
                let mut rows = vec![];
                for (start, end, info) in map.to_array() {
                    let words = info
                        .word_idx
                        .iter()
                        .filter_map(|&idx| {
                            self.word_map.get_word(&start, &end, idx).map(str::to_owned)
                        })
                        .collect();
                    let pairs = info
                        .pairs
                        .iter()
                        .filter_map(|(s, e, idx)| {
                            self.word_map.get_word(s, e, *idx).map(str::to_owned)
                        })
                        .collect();
                    rows.push(MoveRow {
                        r#move: (start, end),
                        node_types: info.node_types,
                        words,
                        pairs,
                    });
                }
                rows.sort_by(|a, b| a.r#move.cmp(&b.r#move));
                cards.push(WordsCard {
                    move_type,
                    depth: has_depth.then_some(group),
                    // Preserve the original card presentation flag, whose grouping is 1 - connected.
                    connected: (move_type == 1).then_some(group == 1),
                    move_rows: rows,
                });
            }
            cards.sort_by_key(|c| c.depth.unwrap_or(0));
            if move_type == 3 || move_type == 5 {
                cards.reverse();
            }
            result.extend(cards);
        }
        result
    }
    pub fn get_words_cards_from_char(
        &self,
        node: &str,
        view: usize,
        direction: usize,
        change: Option<ChangeRule>,
    ) -> Vec<WordsCard> {
        let moves = self
            .graph_solver
            .graphs
            .get_moves_from_node(node, view, direction, change, None);
        let mut indices = EdgeMap::new();
        for (start, end) in moves {
            let (begin, limit) = self
                .graph_solver
                .graphs
                .get_edge_idx_range(&start, &end, None);
            indices.set(&start, &end, (begin..limit).collect());
        }
        self.move_class_to_words_cards(self.graph_solver.classify_moves(&indices))
    }
    pub fn get_scc_data(&self, view: usize, asc: bool) -> Vec<WordSccData> {
        self.graph_solver
            .get_scc_data(view, asc)
            .into_iter()
            .map(|data| WordSccData {
                nodes: data.nodes,
                succ: data
                    .succ
                    .into_iter()
                    .map(|s| WordSccSuccessor {
                        nodes: s.nodes,
                        by: s
                            .by
                            .into_iter()
                            .flat_map(|(start, end, (a, b))| {
                                (a..b)
                                    .filter_map(|i| {
                                        self.word_map.get_word(&start, &end, i).map(str::to_owned)
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect()
    }
    pub fn get_distinct_route_words(&self, moves: &[SingleMove]) -> Result<Vec<String>> {
        let mut counter: IndexMap<SingleMove, usize> = IndexMap::new();
        let mut words = vec![];
        for (start, end) in moves {
            if start == "__none" {
                words.push(end.clone());
                continue;
            }
            let (_, offset) =
                self.graph_solver
                    .graphs
                    .get_edge_num_and_offset(start, end, Some("route"));
            let used = counter.entry((start.clone(), end.clone())).or_default();
            let word = self
                .word_map
                .get_word(start, end, offset + *used)
                .ok_or_else(|| {
                    crate::Error::InvalidInput(format!("route word exhausted: ({start},{end})"))
                })?;
            words.push(word.into());
            *used += 1;
        }
        Ok(words)
    }
    fn nodes_of_type(&self, node_type: NodeType, view: usize) -> Vec<String> {
        let mut nodes: Vec<_> = self.graph_solver.type_map[view]
            .iter()
            .filter(|(_, t)| **t == node_type)
            .map(|(n, _)| n.clone())
            .collect();
        nodes.sort();
        nodes
    }
    pub fn get_lose_words_file(&self, view: usize) -> Vec<CharWords<Vec<WordsCard>>> {
        self.nodes_of_type(NodeType::Lose, view)
            .into_iter()
            .map(|node| CharWords {
                words: self.get_words_cards_from_char(&node, view, 0, None),
                r#char: node,
            })
            .collect()
    }
    pub fn get_essential_win_word_file(&self, view: usize) -> Vec<CharWords<WordsCard>> {
        let mut nodes = self.nodes_of_type(NodeType::Win, view);
        nodes.extend(self.nodes_of_type(NodeType::LoopWin, view));
        nodes.sort();
        nodes
            .into_iter()
            .filter_map(|node| {
                self.get_words_cards_from_char(&node, view, 0, None)
                    .into_iter()
                    .find(|c| c.move_type == 0)
                    .map(|words| CharWords {
                        r#char: node,
                        words,
                    })
            })
            .collect()
    }
    pub fn get_win_words_file(&self, view: usize) -> Vec<CharInfo> {
        self.nodes_of_type(NodeType::Win, view)
            .into_iter()
            .map(|node| {
                let info = self
                    .get_words_cards_from_char(&node, view, 0, None)
                    .into_iter()
                    .take_while(|c| c.move_type == 0)
                    .map(|c| DepthWords {
                        depth: c.depth,
                        words: c.move_rows.into_iter().flat_map(|r| r.words).collect(),
                    })
                    .collect();
                CharInfo { r#char: node, info }
            })
            .collect()
    }
    pub fn get_bangdan_file(&self, view: usize) -> Vec<CharInfo> {
        self.nodes_of_type(NodeType::Lose, view)
            .into_iter()
            .filter_map(|node| {
                let info: Vec<_> = self
                    .get_words_cards_from_char(&node, view, 0, None)
                    .into_iter()
                    .map(|c| DepthWords {
                        depth: c.depth,
                        words: c.move_rows.into_iter().flat_map(|r| r.words).collect(),
                    })
                    .collect();
                (!info.is_empty()).then_some(CharInfo { r#char: node, info })
            })
            .collect()
    }
    pub fn get_essential_win_words_file(&self, view: usize) -> Vec<EssentialWord> {
        self.nodes_of_type(NodeType::Win, view)
            .into_iter()
            .filter_map(|node| {
                let (start, end) = self
                    .graph_solver
                    .get_winning_optimal_move(view, &node, None)?;
                self.words_in_partition(&start, &end, "winlose")
                    .first()
                    .map(|word| EssentialWord {
                        r#char: node,
                        word: word.clone(),
                    })
            })
            .collect()
    }
    pub fn get_route_words_file(&self, view: usize, rare: usize) -> Vec<CharWords<Vec<String>>> {
        self.graph_solver
            .get_route_nodes(view)
            .get(rare)
            .into_iter()
            .flatten()
            .map(|node| {
                let words = self
                    .graph_solver
                    .graphs
                    .get_moves_from_node(node, view, 0, None, Some(&["route"]))
                    .into_iter()
                    .flat_map(|(s, e)| self.words_in_partition(&s, &e, "route"))
                    .collect();
                CharWords {
                    r#char: node.clone(),
                    words,
                }
            })
            .collect()
    }
    pub fn get_removed_words_file(&self) -> Vec<(String, String)> {
        let mut pairs: Vec<(String, String)> = vec![];
        for (start, end, num) in self.graph_solver.graphs.get_graph("removed").edges(1) {
            for idx in 0..num {
                if let Some((s, e, i)) = self
                    .graph_solver
                    .pair_manager
                    .get_pair_idx(&start, &end, idx)
                    && let (Some(a), Some(b)) = (
                        self.word_map.get_word(&start, &end, idx),
                        self.word_map.get_word(&s, &e, i),
                    )
                {
                    pairs.push((a.into(), b.into()));
                }
            }
        }
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        pairs
    }
    pub fn get_ikki_words_file(&self, prec: &PrecInfo) -> String {
        let class = self
            .graph_solver
            .classify_moves(&self.word_map.get_moves(|_| true));
        let mut cards = self.move_class_to_words_cards(class);
        let mut rows = vec!["\"word\",\"sort_key\",\"rule\"".to_string()];
        for card in &mut cards {
            if card.move_type == 1 || card.move_type == 4 {
                card.move_rows.sort_by(|a, b| {
                    self.graph_solver
                        .graphs
                        .get_graph("route")
                        .compare_next_move_num(&a.r#move, &b.r#move, prec)
                });
            }
            for row in &card.move_rows {
                for word in &row.words {
                    rows.push(format!(
                        "\"{}\",\"{}\",\"2\"",
                        word.replace('"', "\"\""),
                        rows.len() - 1
                    ));
                }
            }
        }
        rows.join("\n")
    }
}
