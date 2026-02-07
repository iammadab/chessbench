use serde::{Deserialize, Serialize};

use crate::domain::{Clock, MatchResult, MatchStatus, ResultReason};

#[derive(Debug, Serialize)]
pub struct EnginesResponse {
    pub engines: Vec<EngineInfo>,
}

#[derive(Debug, Serialize)]
pub struct EngineInfo {
    pub id: String,
    pub name: String,
    pub author: String,
}

#[derive(Debug, Deserialize)]
pub struct MatchCreateRequest {
    pub white_engine_id: String,
    pub black_engine_id: String,
    pub time_control: TimeControlRequest,
}

#[derive(Debug, Deserialize)]
pub struct TimeControlRequest {
    pub initial_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct MatchCreateResponse {
    pub match_id: String,
}

#[derive(Debug, Serialize)]
pub struct MatchStatusResponse {
    pub match_id: String,
    pub status: MatchStatus,
    pub current_fen: String,
    pub pgn: String,
    pub clocks: Clock,
    pub result: Option<MatchResult>,
}

#[derive(Debug, Serialize)]
pub struct MatchStartedEvent {
    pub match_id: String,
    pub start_fen: String,
}

#[derive(Debug, Serialize)]
pub struct ClockEvent {
    pub white_ms: u64,
    pub black_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct MoveEvent {
    pub ply: u32,
    pub uci: String,
    pub san: String,
    pub fen: String,
    pub pgn: String,
}

#[derive(Debug, Serialize)]
pub struct ResultEvent {
    pub result: String,
    pub reason: ResultReason,
}
