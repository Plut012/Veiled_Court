mod board;
mod rules;
mod types;

pub use types::{Color, Position, Group};

/// Main game state
pub struct Game {
    board: board::Board,
    turn: Color,
    prisoners: (u32, u32), // (black_captured, white_captured)
    history: Vec<u64>,     // Board hashes for ko detection
}

impl Game {
    pub fn new() -> Self {
        Self::with_size(19)
    }

    pub fn with_size(size: usize) -> Self {
        Self {
            board: board::Board::with_size(size),
            turn: Color::Black,
            prisoners: (0, 0),
            history: Vec::new(),
        }
    }

    pub fn reset_with_size(&mut self, size: usize) {
        *self = Self::with_size(size);
    }

    /// Attempt to place a stone at the given position
    pub fn place_stone(&mut self, pos: Position, color: Color) -> Result<(), String> {
        // 1. Check if it's the right player's turn
        if color != self.turn {
            return Err("Not your turn".to_string());
        }

        // 2. Check if position is valid
        if !pos.is_valid() {
            return Err("Invalid position".to_string());
        }

        // 3. Check if intersection is empty
        if !self.board.is_empty(pos) {
            return Err("Intersection occupied".to_string());
        }

        // 4. Check suicide rule
        if rules::is_suicide(&self.board, pos, color) {
            return Err("Suicide move not allowed".to_string());
        }

        // 5. Save board state for potential rollback
        let saved_board = self.board.grid.clone();

        // 6. Place the stone
        self.board.set(pos, Some(color));

        // 7. Remove captured opponent groups
        let opponent_color = color.opposite();
        let captures = rules::find_captures(&self.board, opponent_color);
        let num_captures = captures.len() as u32;

        for capture_pos in &captures {
            self.board.set(*capture_pos, None);
        }

        // 8. Check for ko violation
        let board_hash = rules::hash_board(&self.board);
        if rules::is_ko_violation(board_hash, &self.history) {
            // Rollback the board
            self.board.grid = saved_board;
            return Err("Ko rule violation".to_string());
        }

        // 9. Update prisoner count
        match color {
            Color::Black => self.prisoners.1 += num_captures, // Black captured white stones
            Color::White => self.prisoners.0 += num_captures, // White captured black stones
        }

        // 10. Update history
        self.history.push(board_hash);

        // 11. Switch turn
        self.turn = self.turn.opposite();

        Ok(())
    }

    /// Pass turn
    pub fn pass(&mut self) {
        self.turn = self.turn.opposite();
    }

    /// Reset game to initial state
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Get the current board state as a 2D vector for serialization
    pub fn get_board(&self) -> Vec<Vec<Option<Color>>> {
        let size = self.board.size();
        let mut result = Vec::with_capacity(size);
        for y in 0..size {
            let mut row = Vec::with_capacity(size);
            for x in 0..size {
                row.push(self.board.get(Position::new(x, y)));
            }
            result.push(row);
        }
        result
    }

    /// Get the board size
    pub fn get_board_size(&self) -> usize {
        self.board.size()
    }

    /// Get the current turn
    pub fn get_turn(&self) -> Color {
        self.turn
    }

    /// Get prisoner counts (black_captured, white_captured)
    pub fn get_prisoners(&self) -> (u32, u32) {
        self.prisoners
    }

    /// Detect all groups on the board
    /// Returns a vector of all groups (connected stones of the same color)
    pub fn detect_groups(&self) -> Vec<types::Group> {
        use std::collections::HashSet;

        let mut groups = Vec::new();
        let mut visited: HashSet<Position> = HashSet::new();
        let size = self.board.size();

        // Scan the entire board
        for y in 0..size {
            for x in 0..size {
                let pos = Position::new(x, y);

                // Skip if already visited or empty
                if visited.contains(&pos) {
                    continue;
                }

                // Check if there's a stone at this position
                if let Some(color) = self.board.get(pos) {
                    // Find the group using existing flood-fill logic
                    let group_positions = self.board.find_group(pos);

                    // Mark all positions in this group as visited
                    for &group_pos in &group_positions {
                        visited.insert(group_pos);
                    }

                    // Convert HashSet<Position> to Vec<(usize, usize)> for serialization
                    let mut stones: Vec<(usize, usize)> = group_positions
                        .iter()
                        .map(|p| (p.x, p.y))
                        .collect();

                    // Sort stones for consistent output (useful for testing)
                    stones.sort();

                    // Create and add the group
                    groups.push(types::Group {
                        color,
                        size: stones.len(),
                        stones,
                    });
                }
            }
        }

        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_move_placement() {
        let mut game = Game::new();

        // Black plays first
        assert!(game.place_stone(Position::new(3, 3), Color::Black).is_ok());

        // White plays next
        assert!(game.place_stone(Position::new(3, 4), Color::White).is_ok());

        // Black plays again
        assert!(game.place_stone(Position::new(4, 3), Color::Black).is_ok());
    }

    #[test]
    fn test_turn_enforcement() {
        let mut game = Game::new();

        // Black plays first
        assert!(game.place_stone(Position::new(3, 3), Color::Black).is_ok());

        // Black tries to play again - should fail
        let result = game.place_stone(Position::new(4, 4), Color::Black);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not your turn");

        // White plays - should succeed
        assert!(game.place_stone(Position::new(4, 4), Color::White).is_ok());
    }

    #[test]
    fn test_occupied_intersection() {
        let mut game = Game::new();

        game.place_stone(Position::new(5, 5), Color::Black).unwrap();

        // Try to place white stone on same spot
        let result = game.place_stone(Position::new(5, 5), Color::White);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Intersection occupied");
    }

    #[test]
    fn test_capture_updates_prisoners() {
        let mut game = Game::new();

        // Set up a capture situation - surround a single white stone
        // White stone at (5,5), surrounded by black
        game.place_stone(Position::new(5, 4), Color::Black).unwrap();
        game.place_stone(Position::new(5, 5), Color::White).unwrap();
        game.place_stone(Position::new(4, 5), Color::Black).unwrap();
        game.place_stone(Position::new(10, 10), Color::White).unwrap(); // White plays elsewhere
        game.place_stone(Position::new(6, 5), Color::Black).unwrap();
        game.place_stone(Position::new(11, 11), Color::White).unwrap(); // White plays elsewhere

        // Black captures white stone at (5,5) by playing at (5,6)
        game.place_stone(Position::new(5, 6), Color::Black).unwrap();

        // Check prisoner count - black captured 1 white stone
        assert_eq!(game.prisoners, (0, 1));
    }

    #[test]
    fn test_ko_rule() {
        let mut game = Game::new();

        // Set up a proper ko situation
        //   0 1 2 3
        // 0 . B W .
        // 1 B W . W
        // 2 . B W .

        game.place_stone(Position::new(1, 0), Color::Black).unwrap();
        game.place_stone(Position::new(2, 0), Color::White).unwrap();
        game.place_stone(Position::new(0, 1), Color::Black).unwrap();
        game.place_stone(Position::new(1, 1), Color::White).unwrap();
        game.place_stone(Position::new(1, 2), Color::Black).unwrap();
        game.place_stone(Position::new(2, 2), Color::White).unwrap();
        game.place_stone(Position::new(10, 10), Color::Black).unwrap(); // Black plays elsewhere
        game.place_stone(Position::new(3, 1), Color::White).unwrap();

        // Black captures White at (1,1) by playing at (2,1)
        game.place_stone(Position::new(2, 1), Color::Black).unwrap();

        // Now (1,1) is empty, and White at (1,1) was captured
        // If White plays at (1,1), it recaptures Black at (2,1)
        // This would recreate the board position before Black's last move - ko!
        let result = game.place_stone(Position::new(1, 1), Color::White);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Ko rule violation");
    }

    #[test]
    fn test_suicide_blocked() {
        let mut game = Game::new();

        // Create a surrounded position (alternating turns correctly)
        game.place_stone(Position::new(0, 1), Color::Black).unwrap();
        game.place_stone(Position::new(5, 5), Color::White).unwrap(); // White plays elsewhere
        game.place_stone(Position::new(2, 1), Color::Black).unwrap();
        game.place_stone(Position::new(6, 6), Color::White).unwrap(); // White plays elsewhere
        game.place_stone(Position::new(1, 0), Color::Black).unwrap();
        game.place_stone(Position::new(7, 7), Color::White).unwrap(); // White plays elsewhere
        game.place_stone(Position::new(1, 2), Color::Black).unwrap();

        // White tries suicide at (1,1)
        let result = game.place_stone(Position::new(1, 1), Color::White);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Suicide move not allowed");
    }

    #[test]
    fn test_pass() {
        let mut game = Game::new();

        assert_eq!(game.turn, Color::Black);
        game.pass();
        assert_eq!(game.turn, Color::White);
        game.pass();
        assert_eq!(game.turn, Color::Black);
    }

    #[test]
    fn test_reset() {
        let mut game = Game::new();

        game.place_stone(Position::new(3, 3), Color::Black).unwrap();
        game.place_stone(Position::new(4, 4), Color::White).unwrap();

        game.reset();

        // Board should be empty, turn should be Black
        assert_eq!(game.turn, Color::Black);
        assert_eq!(game.prisoners, (0, 0));
        assert!(game.board.is_empty(Position::new(3, 3)));
        assert!(game.board.is_empty(Position::new(4, 4)));
    }

    #[test]
    fn test_group_detection_single_stone() {
        let mut game = Game::new();

        // Place a single black stone
        game.place_stone(Position::new(5, 5), Color::Black).unwrap();
        game.pass(); // White passes

        let groups = game.detect_groups();

        // Should have exactly 1 group
        assert_eq!(groups.len(), 1);

        let group = &groups[0];
        assert_eq!(group.color, Color::Black);
        assert_eq!(group.size, 1);
        assert_eq!(group.stones.len(), 1);
        assert!(group.stones.contains(&(5, 5)));
    }

    #[test]
    fn test_group_detection_connected_stones() {
        let mut game = Game::new();

        // Place connected black stones in an L-shape
        game.place_stone(Position::new(5, 5), Color::Black).unwrap();
        game.pass();
        game.place_stone(Position::new(5, 6), Color::Black).unwrap();
        game.pass();
        game.place_stone(Position::new(5, 7), Color::Black).unwrap();
        game.pass();
        game.place_stone(Position::new(6, 7), Color::Black).unwrap();

        let groups = game.detect_groups();

        // Should have exactly 1 group (all stones connected)
        assert_eq!(groups.len(), 1);

        let group = &groups[0];
        assert_eq!(group.color, Color::Black);
        assert_eq!(group.size, 4);
        assert_eq!(group.stones.len(), 4);
        assert!(group.stones.contains(&(5, 5)));
        assert!(group.stones.contains(&(5, 6)));
        assert!(group.stones.contains(&(5, 7)));
        assert!(group.stones.contains(&(6, 7)));
    }

    #[test]
    fn test_group_detection_multiple_groups() {
        let mut game = Game::new();

        // Place two separate black groups
        game.place_stone(Position::new(3, 3), Color::Black).unwrap();
        game.place_stone(Position::new(10, 10), Color::White).unwrap();
        game.place_stone(Position::new(3, 4), Color::Black).unwrap();
        game.place_stone(Position::new(10, 11), Color::White).unwrap();

        // Group 1: (3,3) and (3,4) - connected
        // Group 2: (10,10) and (10,11) - connected

        let groups = game.detect_groups();

        // Should have 2 groups (one black, one white)
        assert_eq!(groups.len(), 2);

        // Find black and white groups
        let black_groups: Vec<_> = groups.iter().filter(|g| g.color == Color::Black).collect();
        let white_groups: Vec<_> = groups.iter().filter(|g| g.color == Color::White).collect();

        assert_eq!(black_groups.len(), 1);
        assert_eq!(white_groups.len(), 1);

        let black_group = black_groups[0];
        assert_eq!(black_group.size, 2);
        assert!(black_group.stones.contains(&(3, 3)));
        assert!(black_group.stones.contains(&(3, 4)));

        let white_group = white_groups[0];
        assert_eq!(white_group.size, 2);
        assert!(white_group.stones.contains(&(10, 10)));
        assert!(white_group.stones.contains(&(10, 11)));
    }

    #[test]
    fn test_group_detection_after_capture() {
        let mut game = Game::new();

        // Set up a capture situation - surround a single white stone
        // After capture, black stones should be disconnected (they were only connected through the captured stone)
        //   4 5 6
        // 4 . B .
        // 5 B W B   <- After white is captured, the black stones form 4 separate groups
        // 6 . B .

        // First create the surrounding structure
        game.place_stone(Position::new(5, 4), Color::Black).unwrap();
        game.pass();
        game.place_stone(Position::new(4, 5), Color::Black).unwrap();
        game.pass();
        game.place_stone(Position::new(6, 5), Color::Black).unwrap();
        game.place_stone(Position::new(5, 5), Color::White).unwrap(); // White places in surrounded spot

        // Black captures white stone at (5,5) by playing at (5,6)
        game.place_stone(Position::new(5, 6), Color::Black).unwrap();

        let groups = game.detect_groups();

        // After capture, white stone at (5,5) should be removed
        // Black stones form 4 separate groups (they were only connected through the captured stone)
        let black_groups: Vec<_> = groups.iter().filter(|g| g.color == Color::Black).collect();
        assert_eq!(black_groups.len(), 4);

        // Verify all black stones are present in groups
        let all_black_stones: Vec<(usize, usize)> = black_groups
            .iter()
            .flat_map(|g| g.stones.clone())
            .collect();

        assert!(all_black_stones.contains(&(5, 4)));
        assert!(all_black_stones.contains(&(4, 5)));
        assert!(all_black_stones.contains(&(6, 5)));
        assert!(all_black_stones.contains(&(5, 6)));

        // Verify white stone at (5,5) was captured (not in any group)
        for group in &groups {
            assert!(!group.stones.contains(&(5, 5)));
        }
    }

    #[test]
    fn test_group_detection_merging_groups() {
        let mut game = Game::new();

        // Create two separate black groups
        game.place_stone(Position::new(5, 5), Color::Black).unwrap();
        game.pass();
        game.place_stone(Position::new(7, 5), Color::Black).unwrap();
        game.pass();

        let groups_before = game.detect_groups();
        assert_eq!(groups_before.len(), 2); // Two separate groups

        // Place a stone that connects them
        game.place_stone(Position::new(6, 5), Color::Black).unwrap();

        let groups_after = game.detect_groups();
        assert_eq!(groups_after.len(), 1); // Now merged into one group

        let merged_group = &groups_after[0];
        assert_eq!(merged_group.size, 3);
        assert!(merged_group.stones.contains(&(5, 5)));
        assert!(merged_group.stones.contains(&(6, 5)));
        assert!(merged_group.stones.contains(&(7, 5)));
    }
}
