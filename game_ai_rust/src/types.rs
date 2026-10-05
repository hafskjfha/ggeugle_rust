use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub type NodeName = String;
pub type NodePos = usize;
pub type NodeMap<T> = [IndexMap<String, T>; 2];
pub type SingleMove = (String, String);
pub type Edge = (String, String, usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Win,
    #[serde(rename = "loopwin")]
    LoopWin,
    Lose,
    Route,
}

impl NodeType {
    pub fn normalized(self) -> Self {
        if self == Self::LoopWin {
            Self::Win
        } else {
            self
        }
    }
}

pub fn normalize_node_type(value: NodeType) -> NodeType {
    value.normalized()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PrecedenceMaps {
    pub edge: IndexMap<String, IndexMap<String, f64>>,
    pub node: IndexMap<String, f64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PrecInfo {
    pub rule: usize,
    pub maps: PrecedenceMaps,
}
