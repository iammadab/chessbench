use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchStatus {
    Running,
    Finished,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResultReason {
    Checkmate,
    Stalemate,
    Timeout,
    Illegal,
    Resignation,
    Draw,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    White,
    Black,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clock {
    pub white_ms: u64,
    pub black_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub result: String,
    pub reason: ResultReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchState {
    pub match_id: String,
    pub status: MatchStatus,
    pub current_fen: String,
    pub pgn: String,
    pub clocks: Clock,
    pub result: Option<MatchResult>,
    pub side_to_move: Side,
    pub ply: u32,
    pub start_fen: String,
    pub last_move: Option<MoveSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveSnapshot {
    pub ply: u32,
    pub uci: String,
    pub san: String,
    pub fen: String,
    pub pgn: String,
}
