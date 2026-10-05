use serde::{Deserialize, Serialize};

use crate::{Edge, classify::TwoCycle, edge_map::EdgeMap};

/// Tracks the response word paired with each removed word index.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PairManager {
    pub even_loop_map: EdgeMap<usize>,
    pub two_cycle_map: EdgeMap<Vec<Edge>>,
}

impl PairManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_data(even_loops: &[Edge], two_cycles: &[TwoCycle]) -> Self {
        let mut manager = Self::new();
        for edge in even_loops {
            manager.add_even_loop(edge.clone());
        }
        for cycle in two_cycles {
            manager.add_two_cycle(cycle.clone());
        }
        manager
    }

    pub fn add_even_loop(&mut self, (start, end, count): Edge) {
        if count > 0 {
            *self.even_loop_map.get_or_insert(&start, &end, 0) += count;
        }
    }

    pub fn add_two_cycle(&mut self, ((start1, end1), (start2, end2), count): TwoCycle) {
        if count == 0 {
            return;
        }
        self.two_cycle_map
            .get_or_insert(&start1, &end1, Vec::new())
            .push((start2.clone(), end2.clone(), count));
        self.two_cycle_map
            .get_or_insert(&start2, &end2, Vec::new())
            .push((start1, end1, count));
    }

    pub fn get_even_loop_num(&self, head: &str, tail: &str) -> usize {
        self.even_loop_map.get(head, tail).copied().unwrap_or(0)
    }

    pub fn get_two_cycles_num(&self, head: &str, tail: &str) -> usize {
        self.two_cycle_map
            .get(head, tail)
            .map(|pairs| pairs.iter().map(|edge| edge.2).sum())
            .unwrap_or(0)
    }

    pub fn get_removed_word_num(&self, head: &str, tail: &str) -> usize {
        self.get_even_loop_num(head, tail) + self.get_two_cycles_num(head, tail)
    }

    pub fn get_pair_idx(&self, head: &str, tail: &str, mut index: usize) -> Option<Edge> {
        let even_count = self.get_even_loop_num(head, tail);
        if index < even_count {
            return Some((
                head.to_owned(),
                tail.to_owned(),
                (index + even_count / 2) % even_count,
            ));
        }
        index -= even_count;
        if let Some(pairs) = self.two_cycle_map.get(head, tail) {
            for (pair_head, pair_tail, count) in pairs {
                if index < *count {
                    return Some((pair_head.clone(), pair_tail.clone(), index));
                }
                index -= count;
            }
        }
        None
    }

    pub fn is_removed(&self, head: &str, tail: &str, index: usize) -> bool {
        index < self.get_removed_word_num(head, tail)
    }
}
