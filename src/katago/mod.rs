use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use crate::game::{Color, Position};

pub mod jaguar;
pub mod crow;

/// KataGo process for GTP (Go Text Protocol) mode
pub struct KataGoProcess {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl KataGoProcess {
    /// Spawn a new KataGo process in GTP mode with the specified config file
    pub fn spawn(config_path: &str) -> Result<Self, String> {
        let binary_path = PathBuf::from(
            std::env::var("KATAGO_BINARY").unwrap_or_else(|_| "assets/katago/katago".to_string()),
        );
        let model_path = PathBuf::from(
            std::env::var("KATAGO_MODEL").unwrap_or_else(|_| "assets/katago/model.bin.gz".to_string()),
        );

        // Verify binary exists
        if !binary_path.exists() {
            return Err(format!(
                "KataGo binary not found at {:?}",
                binary_path
            ));
        }

        // Verify model exists
        if !model_path.exists() {
            return Err(format!(
                "KataGo model not found at {:?}",
                model_path
            ));
        }

        // Verify config exists
        let config_path = PathBuf::from(config_path);
        if !config_path.exists() {
            return Err(format!(
                "KataGo config not found at {:?}",
                config_path
            ));
        }

        let human_model_path = std::env::var("KATAGO_HUMAN_MODEL").ok().map(PathBuf::from);

        let mut cmd = Command::new(&binary_path);
        cmd.arg("gtp")
            .arg("-model")
            .arg(&model_path)
            .arg("-config")
            .arg(&config_path);

        if let Some(ref hm) = human_model_path {
            if hm.exists() {
                cmd.arg("-human-model").arg(hm);
            }
        }

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn KataGo process: {}", e))?;

        let stdin = child
            .stdin
            .take()
            .ok_or("Failed to open KataGo stdin")?;

        let stdout = child
            .stdout
            .take()
            .ok_or("Failed to open KataGo stdout")?;

        Ok(Self {
            process: child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    /// Send a GTP command and receive the response
    fn send_command(&mut self, command: &str) -> Result<String, String> {
        // Send command
        writeln!(self.stdin, "{}", command)
            .map_err(|e| format!("Failed to write to KataGo: {}", e))?;

        self.stdin.flush()
            .map_err(|e| format!("Failed to flush KataGo stdin: {}", e))?;

        // Read response lines until we get "=" or "?" (success/failure indicator)
        let mut response_lines = Vec::new();
        let mut first_line = String::new();

        self.stdout.read_line(&mut first_line)
            .map_err(|e| format!("Failed to read from KataGo: {}", e))?;

        if first_line.is_empty() {
            return Err("KataGo process closed unexpectedly".to_string());
        }

        // Check for success (=) or failure (?)
        if first_line.starts_with('?') {
            return Err(format!("KataGo command failed: {}", first_line));
        }

        if !first_line.starts_with('=') {
            return Err(format!("Unexpected KataGo response: {}", first_line));
        }

        // The first line after '=' contains the actual response
        let response_content = first_line[1..].trim().to_string();

        // Read until we get an empty line (GTP protocol)
        loop {
            let mut line = String::new();
            self.stdout.read_line(&mut line)
                .map_err(|e| format!("Failed to read from KataGo: {}", e))?;

            if line.trim().is_empty() {
                break;
            }
            response_lines.push(line.trim().to_string());
        }

        Ok(response_content)
    }

    /// Set the board size
    pub fn set_boardsize(&mut self, size: usize) -> Result<(), String> {
        self.send_command(&format!("boardsize {}", size))?;
        Ok(())
    }

    /// Clear the board
    pub fn clear_board(&mut self) -> Result<(), String> {
        self.send_command("clear_board")?;
        Ok(())
    }

    /// Play a move on the board
    pub fn play(&mut self, color: Color, position: Position, board_size: usize) -> Result<(), String> {
        let color_str = match color {
            Color::Black => "black",
            Color::White => "white",
        };
        let gtp_coord = Self::position_to_gtp(position, board_size);
        self.send_command(&format!("play {} {}", color_str, gtp_coord))?;
        Ok(())
    }

    /// Generate a move for the specified color
    pub fn genmove(&mut self, color: Color, board_size: usize) -> Result<Position, String> {
        let color_str = match color {
            Color::Black => "black",
            Color::White => "white",
        };
        let response = self.send_command(&format!("genmove {}", color_str))?;

        if response.to_lowercase() == "pass" {
            return Err("KataGo passed".to_string());
        }

        Self::parse_gtp_move(&response, board_size)
    }

    /// Generate a move for the specified color with a custom visit count
    pub fn genmove_with_visits(&mut self, color: Color, visits: u32, board_size: usize) -> Result<Position, String> {
        // First, set the visit count using kata-set-param
        self.send_command(&format!("kata-set-param maxVisits {}", visits))?;

        // Then generate the move
        self.genmove(color, board_size)
    }

    /// Convert position to GTP coordinate (e.g., (3, 3) -> "D16" for 19x19 board)
    pub fn position_to_gtp(pos: Position, board_size: usize) -> String {
        // GTP uses letters A-T (skipping I) for columns, numbers 1-19 for rows
        // x=0 -> A, x=1 -> B, ..., x=7 -> H, x=8 -> J (skip I), ...
        // y=0 -> bottom row (19 for 19x19), y=18 -> top row (1)

        let col_char = if pos.x < 8 {
            (b'A' + pos.x as u8) as char
        } else {
            (b'A' + pos.x as u8 + 1) as char // Skip 'I'
        };

        let row_num = board_size - pos.y;

        format!("{}{}", col_char, row_num)
    }

    /// Parse GTP coordinate to Position (e.g., "D16" -> (3, 3) for 19x19 board)
    pub fn parse_gtp_move(gtp: &str, board_size: usize) -> Result<Position, String> {
        if gtp.to_lowercase() == "pass" {
            return Err("Pass move".to_string());
        }

        let gtp = gtp.to_uppercase();
        let mut chars = gtp.chars();

        let col_char = chars.next().ok_or("Invalid GTP coordinate")?;
        let row_str: String = chars.collect();
        let row_num: usize = row_str.parse().map_err(|_| "Invalid row number")?;

        // Convert column letter to x coordinate
        let x = if col_char < 'I' {
            (col_char as u8 - b'A') as usize
        } else {
            (col_char as u8 - b'A' - 1) as usize // Account for skipped 'I'
        };

        // Convert row number to y coordinate
        let y = board_size - row_num;

        if x >= board_size || y >= board_size {
            return Err(format!("GTP coordinate out of bounds: {}", gtp));
        }

        Ok(Position::new(x, y))
    }
}

impl Drop for KataGoProcess {
    fn drop(&mut self) {
        // Clean up: kill the KataGo process
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gtp_conversion() {
        // Test position to GTP
        assert_eq!(KataGoProcess::position_to_gtp(Position::new(0, 0), 19), "A19");
        assert_eq!(KataGoProcess::position_to_gtp(Position::new(3, 3), 19), "D16");
        assert_eq!(KataGoProcess::position_to_gtp(Position::new(8, 8), 19), "J11"); // Skip I
        assert_eq!(KataGoProcess::position_to_gtp(Position::new(18, 18), 19), "T1");

        // Test GTP to position
        assert_eq!(KataGoProcess::parse_gtp_move("A19", 19).unwrap(), Position::new(0, 0));
        assert_eq!(KataGoProcess::parse_gtp_move("D16", 19).unwrap(), Position::new(3, 3));
        assert_eq!(KataGoProcess::parse_gtp_move("Q4", 19).unwrap(), Position::new(15, 15));

        // Test pass
        assert!(KataGoProcess::parse_gtp_move("pass", 19).is_err());
    }

    #[test]
    #[ignore] // Run with --ignored to test actual KataGo communication
    fn test_katago_gtp_spawn() {
        let mut process = KataGoProcess::spawn("configs/dragon.cfg").expect("Failed to spawn KataGo");

        // Test basic commands
        process.set_boardsize(19).expect("Failed to set board size");
        process.clear_board().expect("Failed to clear board");

        // Test playing a move
        process.play(Color::Black, Position::new(3, 3), 19).expect("Failed to play move");

        // Test generating a move
        let bot_move = process.genmove(Color::White, 19);
        assert!(bot_move.is_ok(), "Failed to generate move: {:?}", bot_move.err());
    }
}
