use crate::{Error, Result, rules::ChangeRule, types::*, words::WordSolver};
use serde::{Deserialize, Serialize};

pub enum StrategyRoot<M> {
    Winning(M),
    Losing(Vec<M>),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LosingBranch<M> {
    pub moves: Vec<M>,
    pub selected_index: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum StrategyState<M> {
    Winning {
        r#move: M,
        depth: usize,
    },
    Losing {
        moves: Vec<M>,
        selected_index: usize,
        depth: usize,
    },
}
type WinningMove<'a, M> = Box<dyn Fn(&M) -> Option<M> + 'a>;
type LosingMoves<'a, M> = Box<dyn Fn(&M) -> Vec<M> + 'a>;

pub struct StrategyTree<'a, M> {
    pub winning_path: Vec<M>,
    pub losing_moves: Vec<LosingBranch<M>>,
    pub is_win: bool,
    get_winning_move: WinningMove<'a, M>,
    get_losing_moves: LosingMoves<'a, M>,
}
impl<'a, M: Clone> StrategyTree<'a, M> {
    pub fn new(
        root: StrategyRoot<M>,
        winning: impl Fn(&M) -> Option<M> + 'a,
        losing: impl Fn(&M) -> Vec<M> + 'a,
    ) -> Self {
        let mut tree = Self {
            winning_path: vec![],
            losing_moves: vec![],
            is_win: matches!(root, StrategyRoot::Winning(_)),
            get_winning_move: Box::new(winning),
            get_losing_moves: Box::new(losing),
        };
        match root {
            StrategyRoot::Winning(movement) => tree.expand(Some(movement), 0),
            StrategyRoot::Losing(moves) => tree.expand_losing(0, moves, 0),
        }
        tree
    }
    fn expand(&mut self, mut movement: Option<M>, mut depth: usize) {
        // User-supplied callbacks must describe a finite strategy; iteration avoids stack growth.
        while let Some(current) = movement {
            self.winning_path.truncate(depth);
            self.winning_path.push(current.clone());
            let moves = (self.get_losing_moves)(&current);
            let losing_depth = depth + 1 - usize::from(self.is_win);
            self.losing_moves.truncate(losing_depth);
            if moves.is_empty() {
                break;
            }
            movement = (self.get_winning_move)(&moves[0]);
            self.losing_moves.push(LosingBranch {
                moves,
                selected_index: 0,
            });
            depth = losing_depth + usize::from(self.is_win);
        }
    }
    fn expand_losing(&mut self, depth: usize, moves: Vec<M>, index: usize) {
        self.losing_moves.truncate(depth);
        self.winning_path.truncate(depth + usize::from(self.is_win));
        if let Some(movement) = moves.get(index).and_then(&self.get_winning_move) {
            self.losing_moves.push(LosingBranch {
                moves,
                selected_index: index,
            });
            self.expand(Some(movement), depth + usize::from(self.is_win));
        }
    }
    pub fn select_index(&mut self, depth: usize, index: usize) -> Result<()> {
        let branch = self
            .losing_moves
            .get(depth)
            .ok_or_else(|| Error::InvalidInput("strategy depth out of bounds".into()))?;
        if index >= branch.moves.len() {
            return Err(Error::InvalidInput(
                "strategy selection out of bounds".into(),
            ));
        }
        self.expand_losing(depth, branch.moves.clone(), index);
        Ok(())
    }
    pub fn get_state(&self) -> Vec<StrategyState<M>> {
        let mut result = vec![];
        for (depth, movement) in self.winning_path.iter().enumerate() {
            let losing_depth = depth.saturating_sub(usize::from(self.is_win));
            if (!self.is_win || depth > 0)
                && let Some(branch) = self.losing_moves.get(losing_depth)
            {
                result.push(StrategyState::Losing {
                    moves: branch.moves.clone(),
                    selected_index: branch.selected_index,
                    depth: losing_depth,
                });
            }
            result.push(StrategyState::Winning {
                r#move: movement.clone(),
                depth,
            });
        }
        result
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeData {
    pub is_win: bool,
    /// One entry for a winning move, one entry per alternative for a losing turn.
    pub words: Vec<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_index: Option<usize>,
    pub depth: usize,
}
pub struct WcStrategyTree<'a> {
    pub tree: StrategyTree<'a, SingleMove>,
    pub solver: &'a WordSolver,
}
impl<'a> WcStrategyTree<'a> {
    pub fn new(
        solver: &'a WordSolver,
        pos: usize,
        start: &str,
        change: Option<ChangeRule>,
    ) -> Result<Self> {
        if pos > 1 {
            return Err(Error::InvalidInput("position must be 0 or 1".into()));
        }
        let graph_solver = &solver.graph_solver;
        let sorted_losing = move |node: &str| {
            let mut moves = graph_solver
                .graphs
                .get_graph("winlose")
                .get_moves_from_node(node, 0, 0, change);
            moves.retain(|(s, e)| graph_solver.loop_map.get(s) != Some(e));
            moves.sort_by_key(|(_, end)| {
                std::cmp::Reverse(graph_solver.depth_map[0].get(end).copied().unwrap_or(0))
            });
            moves
        };
        let root = match graph_solver.get_node_type(start, pos, change).normalized() {
            NodeType::Route => {
                return Err(Error::InvalidInput(
                    "route node cannot make strategy tree".into(),
                ));
            }
            NodeType::Win => StrategyRoot::Winning(
                graph_solver
                    .get_winning_optimal_move(pos, start, change)
                    .ok_or_else(|| {
                        Error::InvalidInput("winning node has no optimal move".into())
                    })?,
            ),
            _ => {
                let mut moves = graph_solver
                    .graphs
                    .get_graph("winlose")
                    .get_moves_from_node(start, pos, 0, change);
                moves.sort_by_key(|(_, end)| {
                    std::cmp::Reverse(graph_solver.depth_map[0].get(end).copied().unwrap_or(0))
                });
                StrategyRoot::Losing(moves)
            }
        };
        let tree = StrategyTree::new(
            root,
            move |movement: &SingleMove| {
                graph_solver.get_winning_optimal_move(0, &movement.1, change)
            },
            move |movement: &SingleMove| sorted_losing(&movement.1),
        );
        Ok(Self { tree, solver })
    }
    pub fn select_index(&mut self, depth: usize, index: usize) -> Result<()> {
        self.tree.select_index(depth, index)
    }
    pub fn get_tree_data(&self) -> Vec<TreeData> {
        let words = |movement: &SingleMove| {
            let (a, b) = self.solver.graph_solver.graphs.get_edge_idx_range(
                &movement.0,
                &movement.1,
                Some("winlose"),
            );
            (a..b)
                .filter_map(|idx| {
                    self.solver
                        .word_map
                        .get_word(&movement.0, &movement.1, idx)
                        .map(str::to_owned)
                })
                .collect()
        };
        self.tree
            .get_state()
            .into_iter()
            .map(|state| match state {
                StrategyState::Winning { r#move, depth } => TreeData {
                    is_win: true,
                    words: vec![words(&r#move)],
                    selected_index: None,
                    depth,
                },
                StrategyState::Losing {
                    moves,
                    selected_index,
                    depth,
                } => TreeData {
                    is_win: false,
                    words: moves.iter().map(words).collect(),
                    selected_index: Some(selected_index),
                    depth,
                },
            })
            .collect()
    }
}
