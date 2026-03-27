use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub fn opposite(&self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn is_valid(&self) -> bool {
        self.is_valid_for_size(19)
    }

    pub fn is_valid_for_size(&self, size: usize) -> bool {
        self.x < size && self.y < size
    }

    /// Get adjacent positions (up, down, left, right)
    pub fn adjacent(&self) -> Vec<Position> {
        self.adjacent_for_size(19)
    }

    pub fn adjacent_for_size(&self, size: usize) -> Vec<Position> {
        let mut positions = Vec::new();

        if self.x > 0 {
            positions.push(Position::new(self.x - 1, self.y));
        }
        if self.x < size - 1 {
            positions.push(Position::new(self.x + 1, self.y));
        }
        if self.y > 0 {
            positions.push(Position::new(self.x, self.y - 1));
        }
        if self.y < size - 1 {
            positions.push(Position::new(self.x, self.y + 1));
        }

        positions
    }
}

/// Represents a group of connected stones of the same color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub color: Color,
    pub stones: Vec<(usize, usize)>, // Positions as (x, y) tuples for JSON serialization
    pub size: usize,
}
