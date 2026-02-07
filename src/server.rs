use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::api::{
    EngineInfo, EnginesResponse, MatchCreateRequest, MatchCreateResponse, MatchStatusResponse,
};
use crate::config::EngineConfigFile;
use crate::domain::{Clock, MatchState, MatchStatus, Side};

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Clone)]
pub struct AppState {
    engines: Arc<Vec<EngineInfo>>,
    engine_ids: Arc<Vec<String>>,
    matches: Arc<RwLock<HashMap<String, MatchState>>>,
}

#[derive(Debug, serde::Serialize)]
struct ErrorResponse {
    error: String,
}

pub fn build_router(config: EngineConfigFile) -> Router {
    let engines: Vec<EngineInfo> = config
        .engine
        .into_iter()
        .map(|entry| EngineInfo {
            id: entry.id.clone(),
            name: entry.id,
            author: String::new(),
        })
        .collect();

    let engine_ids = engines.iter().map(|engine| engine.id.clone()).collect();

    let state = AppState {
        engines: Arc::new(engines),
        engine_ids: Arc::new(engine_ids),
        matches: Arc::new(RwLock::new(HashMap::new())),
    };

    Router::new()
        .route("/api/engines", get(get_engines))
        .route("/api/match", post(create_match))
        .route("/api/match/:id", get(get_match))
        .with_state(state)
}

async fn get_engines(State(state): State<AppState>) -> impl IntoResponse {
    let response = EnginesResponse {
        engines: state.engines.as_ref().clone(),
    };

    Json(response)
}

async fn create_match(
    State(state): State<AppState>,
    Json(payload): Json<MatchCreateRequest>,
) -> Result<Json<MatchCreateResponse>, (StatusCode, Json<ErrorResponse>)> {
    if payload.time_control.initial_ms == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "initial_ms must be greater than zero".to_string(),
            }),
        ));
    }

    if !state.engine_ids.contains(&payload.white_engine_id)
        || !state.engine_ids.contains(&payload.black_engine_id)
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "unknown engine id".to_string(),
            }),
        ));
    }

    let match_id = Uuid::new_v4().to_string();
    let state_entry = MatchState {
        match_id: match_id.clone(),
        status: MatchStatus::Running,
        current_fen: START_FEN.to_string(),
        pgn: String::new(),
        clocks: Clock {
            white_ms: payload.time_control.initial_ms,
            black_ms: payload.time_control.initial_ms,
        },
        result: None,
        side_to_move: Side::White,
        ply: 0,
        start_fen: START_FEN.to_string(),
    };

    let mut matches = state.matches.write().await;
    matches.insert(match_id.clone(), state_entry);

    Ok(Json(MatchCreateResponse { match_id }))
}

async fn get_match(
    State(state): State<AppState>,
    Path(match_id): Path<String>,
) -> Result<Json<MatchStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    let matches = state.matches.read().await;
    let Some(entry) = matches.get(&match_id) else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "match not found".to_string(),
            }),
        ));
    };

    let response = MatchStatusResponse {
        match_id: entry.match_id.clone(),
        status: entry.status,
        current_fen: entry.current_fen.clone(),
        pgn: entry.pgn.clone(),
        clocks: entry.clocks.clone(),
        result: entry.result.clone(),
    };

    Ok(Json(response))
}
