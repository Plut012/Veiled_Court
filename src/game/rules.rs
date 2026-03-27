use super::board::Board;
use super::types::{Color, Position};

/// Check if placing a stone would be suicide (illegal unless it captures)
pub fn is_suicide(board: &Board, pos: Position, color: Color) -> bool {
    // Create a temporary board to test the move
    let mut test_board = Board {
        size: board.size(),
        grid: board.grid.clone(),
    };

    // Place the stone
    test_board.set(pos, Some(color));

    // Check if this move captures any opponent stones
    let opponent_color = color.opposite();
    let captures = find_captures(&test_board, opponent_color);

    // If we capture opponent stones, remove them
    if !captures.is_empty() {
        for capture_pos in captures {
            test_board.set(capture_pos, None);
        }
    }

    // Now check if our group has liberties
    // If it has zero liberties, it's suicide
    test_board.count_liberties(pos) == 0
}

/// Detect and return positions of captured stones (groups with 0 liberties)
pub fn find_captures(board: &Board, opponent_color: Color) -> Vec<Position> {
    use std::collections::HashSet;

    let mut captured = Vec::new();
    let mut checked_groups: HashSet<Position> = HashSet::new();

    let size = board.size();

    // Scan board for opponent stones
    for y in 0..size {
        for x in 0..size {
            let pos = Position::new(x, y);

            // Skip if already checked as part of another group
            if checked_groups.contains(&pos) {
                continue;
            }

            // Check if this is an opponent stone
            if board.get(pos) == Some(opponent_color) {
                let group = board.find_group(pos);

                // Mark all positions in this group as checked
                for &group_pos in &group {
                    checked_groups.insert(group_pos);
                }

                // If group has no liberties, it's captured
                if board.count_liberties(pos) == 0 {
                    captured.extend(group);
                }
            }
        }
    }

    captured
}

/// Check if a move violates the ko rule
pub fn is_ko_violation(board_hash: u64, history: &[u64]) -> bool {
    // Simple ko: check if the position matches any recent position
    // For proper ko detection, we check the last few moves
    // Typically, ko occurs when recapturing immediately (2 moves back)
    if history.is_empty() {
        return false;
    }

    // Check if this board state appeared in recent history
    // For simple ko, checking last 2-3 moves is sufficient
    let check_depth = history.len().min(3);
    let recent_history = &history[history.len() - check_depth..];

    recent_history.contains(&board_hash)
}

/// Calculate a hash of the board position (for ko detection)
pub fn hash_board(board: &Board) -> u64 {
    let mut hash: u64 = 0;

    let size = board.size();

    for y in 0..size {
        for x in 0..size {
            let pos = Position::new(x, y);
            if let Some(color) = board.get(pos) {
                // XOR with a unique value for each (position, color) pair
                // Use prime numbers to minimize collisions
                let position_hash = (x as u64 * 19 + y as u64) * 31;
                let color_hash = match color {
                    Color::Black => 1,
                    Color::White => 2,
                };
                hash ^= position_hash.wrapping_mul(color_hash);
            }
        }
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_capture_single_stone() {
        let mut board = Board::new();

        // Surround a black stone at (1,1) with white stones
        board.set(Position::new(1, 1), Some(Color::Black));
        board.set(Position::new(0, 1), Some(Color::White));
        board.set(Position::new(2, 1), Some(Color::White));
        board.set(Position::new(1, 0), Some(Color::White));
        board.set(Position::new(1, 2), Some(Color::White));

        let captures = find_captures(&board, Color::Black);
        assert_eq!(captures.len(), 1);
        assert!(captures.contains(&Position::new(1, 1)));
    }

    #[test]
    fn test_capture_group() {
        let mut board = Board::new();

        // Create a group of two black stones and surround them
        board.set(Position::new(1, 1), Some(Color::Black));
        board.set(Position::new(2, 1), Some(Color::Black));

        // Surround the group
        board.set(Position::new(0, 1), Some(Color::White));
        board.set(Position::new(1, 0), Some(Color::White));
        board.set(Position::new(2, 0), Some(Color::White));
        board.set(Position::new(3, 1), Some(Color::White));
        board.set(Position::new(1, 2), Some(Color::White));
        board.set(Position::new(2, 2), Some(Color::White));

        let captures = find_captures(&board, Color::Black);
        assert_eq!(captures.len(), 2);
        assert!(captures.contains(&Position::new(1, 1)));
        assert!(captures.contains(&Position::new(2, 1)));
    }

    #[test]
    fn test_liberty_counting_single_stone() {
        let mut board = Board::new();

        // Corner stone has 2 liberties
        board.set(Position::new(0, 0), Some(Color::Black));
        assert_eq!(board.count_liberties(Position::new(0, 0)), 2);

        // Side stone has 3 liberties
        board = Board::new();
        board.set(Position::new(0, 5), Some(Color::Black));
        assert_eq!(board.count_liberties(Position::new(0, 5)), 3);

        // Center stone has 4 liberties
        board = Board::new();
        board.set(Position::new(5, 5), Some(Color::Black));
        assert_eq!(board.count_liberties(Position::new(5, 5)), 4);
    }

    #[test]
    fn test_liberty_counting_group() {
        let mut board = Board::new();

        // Two connected stones
        board.set(Position::new(5, 5), Some(Color::Black));
        board.set(Position::new(5, 6), Some(Color::Black));

        // The group should have 6 liberties (3 + 3, sharing 2)
        assert_eq!(board.count_liberties(Position::new(5, 5)), 6);
        assert_eq!(board.count_liberties(Position::new(5, 6)), 6);
    }

    #[test]
    fn test_suicide_illegal() {
        let mut board = Board::new();

        // Surround a position with white stones
        board.set(Position::new(0, 1), Some(Color::White));
        board.set(Position::new(2, 1), Some(Color::White));
        board.set(Position::new(1, 0), Some(Color::White));
        board.set(Position::new(1, 2), Some(Color::White));

        // Placing a black stone at (1,1) would be suicide
        assert!(is_suicide(&board, Position::new(1, 1), Color::Black));
    }

    #[test]
    fn test_suicide_legal_when_capturing() {
        let mut board = Board::new();

        // Create a white stone with one liberty at (2,1)
        board.set(Position::new(2, 1), Some(Color::White));
        board.set(Position::new(1, 1), Some(Color::Black));
        board.set(Position::new(3, 1), Some(Color::Black));
        board.set(Position::new(2, 0), Some(Color::Black));

        // Black at (2,2) would normally be suicide, but it captures white
        // So it should be legal
        board.set(Position::new(1, 2), Some(Color::Black));
        board.set(Position::new(3, 2), Some(Color::Black));

        assert!(!is_suicide(&board, Position::new(2, 2), Color::Black));
    }

    #[test]
    fn test_ko_detection() {
        let mut board = Board::new();

        // Set up a ko situation
        board.set(Position::new(1, 0), Some(Color::Black));
        board.set(Position::new(2, 1), Some(Color::Black));
        board.set(Position::new(1, 2), Some(Color::Black));
        board.set(Position::new(0, 1), Some(Color::White));

        let hash1 = hash_board(&board);

        // White captures at (1,1)
        board.set(Position::new(1, 1), Some(Color::White));
        let hash2 = hash_board(&board);

        // History contains the position before white's move
        let history = vec![hash1];

        // Now black tries to recapture immediately
        // This would create the same board position as before
        // So it should be detected as ko violation
        assert!(is_ko_violation(hash1, &history));
        assert!(!is_ko_violation(hash2, &history));
    }

    #[test]
    fn test_group_finding() {
        let mut board = Board::new();

        // Create an L-shaped group
        board.set(Position::new(5, 5), Some(Color::Black));
        board.set(Position::new(5, 6), Some(Color::Black));
        board.set(Position::new(5, 7), Some(Color::Black));
        board.set(Position::new(6, 7), Some(Color::Black));

        let group = board.find_group(Position::new(5, 5));
        assert_eq!(group.len(), 4);
        assert!(group.contains(&Position::new(5, 5)));
        assert!(group.contains(&Position::new(5, 6)));
        assert!(group.contains(&Position::new(5, 7)));
        assert!(group.contains(&Position::new(6, 7)));
    }

    #[test]
    fn test_snapback() {
        let mut board = Board::new();

        // Set up a connected white group that will be captured
        //   0 1 2 3
        // 0 . B W .
        // 1 B W W W
        // 2 . B W .

        board.set(Position::new(1, 0), Some(Color::Black));
        board.set(Position::new(2, 0), Some(Color::White));
        board.set(Position::new(0, 1), Some(Color::Black));
        board.set(Position::new(1, 1), Some(Color::White));
        board.set(Position::new(2, 1), Some(Color::White));
        board.set(Position::new(3, 1), Some(Color::White));
        board.set(Position::new(1, 2), Some(Color::Black));
        board.set(Position::new(2, 2), Some(Color::White));

        // White group is connected: (2,0)-(1,1)-(2,1)-(3,1) and (2,1)-(2,2)
        // Verify white has liberties before capture
        let captures = find_captures(&board, Color::White);
        assert!(captures.is_empty());

        // Black surrounds completely
        board.set(Position::new(3, 0), Some(Color::Black));
        board.set(Position::new(4, 1), Some(Color::Black));
        board.set(Position::new(3, 2), Some(Color::Black));
        board.set(Position::new(2, 3), Some(Color::Black)); // Close the last liberty

        // Now the entire white group should be captured
        let captures_after = find_captures(&board, Color::White);
        assert_eq!(captures_after.len(), 5); // All 5 white stones captured
    }
}
