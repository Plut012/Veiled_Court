/// Jaguar middleware for dynamic visit scaling
///
/// The Jaguar spirit transforms over the course of the game, starting casual
/// and approachable in the opening (500 visits) and ramping up dramatically
/// in the endgame (20000+ visits) to become a precision assassin.
///
/// This visit scaling creates the character arc that defines the Jaguar.

/// Get the number of visits for Jaguar based on the current move number
///
/// Scaling pattern:
/// - Moves 0-39:   500 visits   (Opening - casual, approachable)
/// - Moves 40-79:  2000 visits  (Midgame - slightly more present)
/// - Moves 80-119: 8000 visits  (Late midgame - the shift begins)
/// - Moves 120+:   20000 visits (Endgame - transformation complete)
pub fn get_jaguar_visits(move_number: usize) -> u32 {
    if move_number < 40 {
        500      // Opening — casual, approachable
    } else if move_number < 80 {
        2000     // Midgame — slightly more present
    } else if move_number < 120 {
        8000     // Late midgame — the shift begins
    } else {
        20000    // Endgame — transformation complete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaguar_visit_scaling() {
        // Opening phase
        assert_eq!(get_jaguar_visits(0), 500);
        assert_eq!(get_jaguar_visits(20), 500);
        assert_eq!(get_jaguar_visits(39), 500);

        // Midgame phase
        assert_eq!(get_jaguar_visits(40), 2000);
        assert_eq!(get_jaguar_visits(60), 2000);
        assert_eq!(get_jaguar_visits(79), 2000);

        // Late midgame phase
        assert_eq!(get_jaguar_visits(80), 8000);
        assert_eq!(get_jaguar_visits(100), 8000);
        assert_eq!(get_jaguar_visits(119), 8000);

        // Endgame phase
        assert_eq!(get_jaguar_visits(120), 20000);
        assert_eq!(get_jaguar_visits(150), 20000);
        assert_eq!(get_jaguar_visits(200), 20000);
    }
}
