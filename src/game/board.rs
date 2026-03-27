use super::types::{Color, Position};
use std::collections::HashSet;

pub(crate) struct Board {
    pub(crate) size: usize,
    pub(crate) grid: Vec<Vec<Option<Color>>>,
}

impl Board {
    pub fn new() -> Self {
        Self::with_size(19)
    }

    pub fn with_size(size: usize) -> Self {
        Self {
            size,
            grid: vec![vec![None; size]; size],
        }
    }

    pub fn get(&self, pos: Position) -> Option<Color> {
        if pos.is_valid_for_size(self.size) {
            self.grid[pos.y][pos.x]
        } else {
            None
        }
    }

    pub fn set(&mut self, pos: Position, color: Option<Color>) {
        if pos.is_valid_for_size(self.size) {
            self.grid[pos.y][pos.x] = color;
        }
    }

    pub fn is_empty(&self, pos: Position) -> bool {
        self.get(pos).is_none()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    /// Find all stones in the same group as the stone at the given position
    /// Uses flood fill to find connected stones of the same color
    pub(crate) fn find_group(&self, pos: Position) -> HashSet<Position> {
        let mut group = HashSet::new();
        let stone_color = match self.get(pos) {
            Some(color) => color,
            None => return group, // No stone at position
        };

        let mut to_visit = vec![pos];
        group.insert(pos);

        while let Some(current) = to_visit.pop() {
            for adjacent in current.adjacent_for_size(self.size) {
                if group.contains(&adjacent) {
                    continue; // Already visited
                }
                if self.get(adjacent) == Some(stone_color) {
                    group.insert(adjacent);
                    to_visit.push(adjacent);
                }
            }
        }

        group
    }

    /// Count liberties for a stone at the given position
    pub(crate) fn count_liberties(&self, pos: Position) -> usize {
        let group = self.find_group(pos);
        let mut liberties = HashSet::new();

        for &stone_pos in &group {
            for adjacent in stone_pos.adjacent_for_size(self.size) {
                if self.is_empty(adjacent) {
                    liberties.insert(adjacent);
                }
            }
        }

        liberties.len()
    }
}
