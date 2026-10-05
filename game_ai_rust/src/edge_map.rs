use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeMap<T> {
    pub content: IndexMap<String, IndexMap<String, T>>,
}

pub type EdgeCounter = EdgeMap<usize>;

impl<T> Default for EdgeMap<T> {
    fn default() -> Self {
        Self {
            content: IndexMap::new(),
        }
    }
}

impl<T> EdgeMap<T> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_array(values: impl IntoIterator<Item = (String, String, T)>) -> Self {
        let mut result = Self::new();
        for (start, end, value) in values {
            result.set(&start, &end, value);
        }
        result
    }
    pub fn set(&mut self, start: &str, end: &str, value: T) {
        self.content
            .entry(start.into())
            .or_default()
            .insert(end.into(), value);
    }
    pub fn get(&self, start: &str, end: &str) -> Option<&T> {
        self.content.get(start)?.get(end)
    }
    pub fn get_mut(&mut self, start: &str, end: &str) -> Option<&mut T> {
        self.content.get_mut(start)?.get_mut(end)
    }
    pub fn get_or_insert(&mut self, start: &str, end: &str, value: T) -> &mut T {
        self.content
            .entry(start.into())
            .or_default()
            .entry(end.into())
            .or_insert(value)
    }
    pub fn remove(&mut self, start: &str, end: Option<&str>) {
        if let Some(end) = end {
            if let Some(inner) = self.content.get_mut(start) {
                inner.shift_remove(end);
                if inner.is_empty() {
                    self.content.shift_remove(start);
                }
            }
        } else {
            self.content.shift_remove(start);
        }
    }
}

impl<T: Clone> EdgeMap<T> {
    pub fn to_array(&self) -> Vec<(String, String, T)> {
        self.content
            .iter()
            .flat_map(|(start, inner)| {
                inner
                    .iter()
                    .map(move |(end, value)| (start.clone(), end.clone(), value.clone()))
            })
            .collect()
    }
    pub fn get_succ(&self, node: &str) -> Vec<(String, T)> {
        self.content
            .get(node)
            .map(|inner| {
                inner
                    .iter()
                    .map(|(end, value)| (end.clone(), value.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn to_object(&self) -> IndexMap<String, IndexMap<String, T>> {
        self.content.clone()
    }
}

impl EdgeMap<usize> {
    pub fn get_num(&self, start: &str, end: &str) -> usize {
        self.get(start, end).copied().unwrap_or(0)
    }
    pub fn increase(&mut self, start: &str, end: &str, amount: usize) {
        let next = self.get_num(start, end) + amount;
        self.set(start, end, next);
    }
    pub fn decrease(&mut self, start: &str, end: &str, amount: usize) -> crate::Result<()> {
        let next = self
            .get_num(start, end)
            .checked_sub(amount)
            .ok_or_else(|| {
                crate::Error::InvalidInput("cannot decrease edge to less than zero".into())
            })?;
        if next == 0 {
            self.remove(start, Some(end));
        } else {
            self.set(start, end, next);
        }
        Ok(())
    }
}
