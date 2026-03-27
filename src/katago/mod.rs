use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::game::{Color, Position};

/// Configuration for KataGo service
#[derive(Debug, Clone)]
pub struct KataGoConfig {
    pub binary_path: PathBuf,
    pub model_path: PathBuf,
    pub config_path: PathBuf,
    pub max_visits: u32,
    pub enable_ownership: bool,
}

impl Default for KataGoConfig {
    fn default() -> Self {
        Self {
            binary_path: PathBuf::from("assets/katago/katago"),
            model_path: PathBuf::from("assets/katago/model.bin.gz"),
            config_path: PathBuf::from("assets/katago/analysis.cfg"),
            max_visits: 50, // Optimized for fast territory estimation
            enable_ownership: true,
        }
    }
}

/// Territory ownership data for each intersection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipData {
    /// Ownership values: -1.0 (black) to +1.0 (white) for each intersection
    pub ownership: Vec<Vec<f32>>,
    /// Standard deviation/confidence for each intersection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ownership_stdev: Option<Vec<Vec<f32>>>,
}

/// KataGo service that manages the analysis process
pub struct KataGoService {
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<BufReader<ChildStdout>>,
    cache: Arc<Mutex<HashMap<u64, OwnershipData>>>,
    config: KataGoConfig,
}

impl KataGoService {
    /// Create a new KataGo service and spawn the analysis process
    pub fn new(config: KataGoConfig) -> Result<Self, String> {
        // Verify binary exists
        if !config.binary_path.exists() {
            return Err(format!(
                "KataGo binary not found at {:?}",
                config.binary_path
            ));
        }

        // Verify model exists
        if !config.model_path.exists() {
            return Err(format!(
                "KataGo model not found at {:?}",
                config.model_path
            ));
        }

        // Verify config exists
        if !config.config_path.exists() {
            return Err(format!(
                "KataGo config not found at {:?}",
                config.config_path
            ));
        }

        Ok(Self {
            process: None,
            stdin: None,
            stdout: None,
            cache: Arc::new(Mutex::new(HashMap::new())),
            config,
        })
    }

    /// Spawn the KataGo analysis process
    fn spawn_process(&mut self) -> Result<(), String> {
        let mut child = Command::new(&self.config.binary_path)
            .arg("analysis")
            .arg("-model")
            .arg(&self.config.model_path)
            .arg("-config")
            .arg(&self.config.config_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()) // Suppress stderr for now
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

        self.process = Some(child);
        self.stdin = Some(stdin);
        self.stdout = Some(BufReader::new(stdout));

        Ok(())
    }

    /// Get ownership data for the current board position
    pub fn get_ownership(&mut self, board: &[Vec<Option<Color>>], board_size: usize) -> Result<OwnershipData, String> {
        // Check cache
        let hash = Self::position_hash(board, board_size);
        if let Some(cached) = self.cache.lock().unwrap().get(&hash) {
            return Ok(cached.clone());
        }

        // Convert board to initial stones
        let initial_stones = Self::board_to_initial_stones(board, board_size);

        // Create query
        let query = AnalysisQuery {
            id: format!("ownership_{}", hash),
            moves: Vec::new(),
            initial_stones,
            rules: "tromp-taylor".to_string(),
            komi: 7.5,
            board_x_size: board_size,
            board_y_size: board_size,
            max_visits: self.config.max_visits,
            include_ownership: true,
        };

        // Send query and get response
        let response = self.send_query(query)?;

        // Extract ownership data
        let ownership_flat = response.ownership.ok_or("No ownership data in response")?;

        // Convert flat array to 2D (KataGo returns flattened row-major)
        let ownership = Self::unflatten_ownership(&ownership_flat, board_size);

        let ownership_data = OwnershipData {
            ownership,
            ownership_stdev: None,
        };

        // Cache result
        self.cache.lock().unwrap().insert(hash, ownership_data.clone());

        Ok(ownership_data)
    }


    /// Send a query to KataGo and receive the response
    fn send_query(&mut self, query: AnalysisQuery) -> Result<AnalysisResponse, String> {
        // Ensure process is running
        if self.process.is_none() {
            self.spawn_process()?;
        }

        let stdin = self.stdin.as_mut().ok_or("KataGo stdin not available")?;
        let stdout = self.stdout.as_mut().ok_or("KataGo stdout not available")?;

        // Serialize and send query
        let query_json = serde_json::to_string(&query)
            .map_err(|e| format!("Failed to serialize query: {}", e))?;

        writeln!(stdin, "{}", query_json)
            .map_err(|e| format!("Failed to write to KataGo: {}", e))?;

        stdin.flush()
            .map_err(|e| format!("Failed to flush KataGo stdin: {}", e))?;

        // Read response
        let mut response_line = String::new();
        stdout.read_line(&mut response_line)
            .map_err(|e| format!("Failed to read from KataGo: {}", e))?;

        // Deserialize response
        let response: AnalysisResponse = serde_json::from_str(&response_line)
            .map_err(|e| format!("Failed to parse KataGo response: {}", e))?;

        Ok(response)
    }

    /// Convert board state to initial stones for KataGo
    fn board_to_initial_stones(board: &[Vec<Option<Color>>], board_size: usize) -> Vec<(String, String)> {
        let mut stones = Vec::new();

        for (y, row) in board.iter().enumerate().take(board_size) {
            for (x, &cell) in row.iter().enumerate().take(board_size) {
                if let Some(color) = cell {
                    let pos = Position::new(x, y);
                    let gtp_coord = Self::position_to_gtp(pos, board_size);
                    let color_str = match color {
                        Color::Black => "B",
                        Color::White => "W",
                    };
                    stones.push((color_str.to_string(), gtp_coord));
                }
            }
        }

        stones
    }

    /// Convert position to GTP coordinate (e.g., (3, 3) -> "D4")
    fn position_to_gtp(pos: Position, board_size: usize) -> String {
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

    /// Parse GTP coordinate to Position (e.g., "D4" -> (3, 3))
    fn parse_gtp_move(gtp: &str, board_size: usize) -> Result<Position, String> {
        if gtp == "pass" {
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

        Ok(Position::new(x, y))
    }

    /// Convert flattened ownership array to 2D
    fn unflatten_ownership(flat: &[f32], board_size: usize) -> Vec<Vec<f32>> {
        flat.chunks(board_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    /// Hash the board position for caching
    fn position_hash(board: &[Vec<Option<Color>>], board_size: usize) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        board_size.hash(&mut hasher);

        for row in board.iter().take(board_size) {
            for cell in row.iter().take(board_size) {
                match cell {
                    None => 0u8.hash(&mut hasher),
                    Some(Color::Black) => 1u8.hash(&mut hasher),
                    Some(Color::White) => 2u8.hash(&mut hasher),
                }
            }
        }
        hasher.finish()
    }
}

impl Drop for KataGoService {
    fn drop(&mut self) {
        // Clean up: kill the KataGo process
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}

/// JSON query sent to KataGo analysis engine
#[derive(Debug, Serialize, Deserialize)]
struct AnalysisQuery {
    id: String,
    moves: Vec<(String, String)>, // e.g. [["B", "Q4"], ["W", "D16"]] - must always be present
    #[serde(rename = "initialStones", skip_serializing_if = "Vec::is_empty")]
    initial_stones: Vec<(String, String)>, // e.g. [["B", "Q4"], ["W", "D16"]]
    rules: String,
    komi: f32,
    #[serde(rename = "boardXSize")]
    board_x_size: usize,
    #[serde(rename = "boardYSize")]
    board_y_size: usize,
    #[serde(rename = "maxVisits")]
    max_visits: u32,
    #[serde(rename = "includeOwnership")]
    include_ownership: bool,
}

/// JSON response from KataGo analysis engine
#[derive(Debug, Serialize, Deserialize)]
struct AnalysisResponse {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ownership: Option<Vec<f32>>, // Flattened array
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_hash() {
        let empty_board: Vec<Vec<Option<Color>>> = vec![vec![None; 19]; 19];
        let hash1 = KataGoService::position_hash(&empty_board, 19);
        let hash2 = KataGoService::position_hash(&empty_board, 19);
        assert_eq!(hash1, hash2, "Same position should have same hash");

        let mut board_with_stone: Vec<Vec<Option<Color>>> = vec![vec![None; 19]; 19];
        board_with_stone[0][0] = Some(Color::Black);
        let hash3 = KataGoService::position_hash(&board_with_stone, 19);
        assert_ne!(hash1, hash3, "Different positions should have different hashes");
    }

    #[test]
    fn test_config_default() {
        let config = KataGoConfig::default();
        assert_eq!(config.max_visits, 100);
        assert!(config.enable_ownership);
    }

    #[test]
    fn test_gtp_conversion() {
        // Test position to GTP
        assert_eq!(KataGoService::position_to_gtp(Position::new(0, 0), 19), "A19");
        assert_eq!(KataGoService::position_to_gtp(Position::new(3, 3), 19), "D16");
        assert_eq!(KataGoService::position_to_gtp(Position::new(8, 8), 19), "J11"); // Skip I
        assert_eq!(KataGoService::position_to_gtp(Position::new(18, 18), 19), "T1");

        // Test GTP to position
        assert_eq!(KataGoService::parse_gtp_move("A19", 19).unwrap(), Position::new(0, 0));
        assert_eq!(KataGoService::parse_gtp_move("D16", 19).unwrap(), Position::new(3, 3));
        assert_eq!(KataGoService::parse_gtp_move("Q4", 19).unwrap(), Position::new(15, 15));

        // Test pass
        assert!(KataGoService::parse_gtp_move("pass", 19).is_err());
    }

    #[test]
    fn test_unflatten_ownership() {
        let flat: Vec<f32> = (0..81).map(|i| i as f32).collect();
        let unflat = KataGoService::unflatten_ownership(&flat, 9);

        assert_eq!(unflat.len(), 9);
        assert_eq!(unflat[0].len(), 9);
        assert_eq!(unflat[0][0], 0.0);
        assert_eq!(unflat[0][8], 8.0);
        assert_eq!(unflat[8][8], 80.0);
    }

    #[test]
    #[ignore] // Run with --ignored to test actual KataGo communication
    fn test_katago_ownership_query() {
        let config = KataGoConfig::default();
        let mut service = KataGoService::new(config).expect("Failed to create service");

        // Create empty 19x19 board
        let empty_board: Vec<Vec<Option<Color>>> = vec![vec![None; 19]; 19];

        // Query ownership
        let result = service.get_ownership(&empty_board, 19);

        assert!(result.is_ok(), "Failed to get ownership: {:?}", result.err());
        let ownership = result.unwrap();

        // Verify ownership data structure
        assert_eq!(ownership.ownership.len(), 19);
        assert_eq!(ownership.ownership[0].len(), 19);

        // For empty board, ownership should be close to 0 everywhere
        for row in &ownership.ownership {
            for &value in row {
                assert!(value.abs() < 1.0, "Ownership value out of range: {}", value);
            }
        }
    }
}
