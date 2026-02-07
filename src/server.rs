use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    response::sse::{Event, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::Stream;
use tokio::sync::RwLock;
use tokio::time::{self, Duration};
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
        .route("/api/match/:id/stream", get(stream_match))
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

async fn stream_match(
    State(state): State<AppState>,
    Path(match_id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, Json<ErrorResponse>)> {
    {
        let matches = state.matches.read().await;
        if !matches.contains_key(&match_id) {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "match not found".to_string(),
                }),
            ));
        }
    }

    let match_id_clone = match_id.clone();
    let state_clone = state.clone();

    let stream = async_stream::stream! {
        let started_payload = serde_json::json!({
            "match_id": match_id_clone,
            "start_fen": START_FEN,
        });
        let started_json = serde_json::to_string(&started_payload).unwrap_or_default();
        yield Ok(Event::default().event("match_started").data(started_json));

        let mut ticker = time::interval(Duration::from_millis(200));
        loop {
            ticker.tick().await;

            let snapshot = {
                let matches = state_clone.matches.read().await;
                matches.get(&match_id).cloned()
            };

            let Some(snapshot) = snapshot else {
                break;
            };

            let clock_payload = serde_json::json!({
                "white_ms": snapshot.clocks.white_ms,
                "black_ms": snapshot.clocks.black_ms,
            });
            let clock_json = serde_json::to_string(&clock_payload).unwrap_or_default();
            yield Ok(Event::default().event("clock").data(clock_json));

            if snapshot.status != MatchStatus::Running {
                if let Some(result) = snapshot.result {
                    let result_json = serde_json::to_string(&result).unwrap_or_default();
                    yield Ok(Event::default().event("result").data(result_json));
                }
                break;
            }
        }
    };

    Ok(Sse::new(stream))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{EngineConfig, EngineConfigFile};
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode as HttpStatus};
    use tower::ServiceExt;

    fn sample_config() -> EngineConfigFile {
        EngineConfigFile {
            engine: vec![
                EngineConfig {
                    id: "stockfish-16".to_string(),
                    path: "/opt/stockfish".into(),
                    args: vec!["-threads".to_string(), "4".to_string()],
                    working_dir: None,
                },
                EngineConfig {
                    id: "lc0-0.30".to_string(),
                    path: "/opt/lc0".into(),
                    args: Vec::new(),
                    working_dir: None,
                },
            ],
        }
    }

    #[tokio::test]
    async fn get_engines_returns_configured_engines() {
        let app = build_router(sample_config());

        let response = app
            .oneshot(Request::builder().uri("/api/engines").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), HttpStatus::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: EnginesResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload.engines.len(), 2);
        assert_eq!(payload.engines[0].id, "stockfish-16");
        assert_eq!(payload.engines[1].id, "lc0-0.30");
    }

    #[tokio::test]
    async fn post_match_creates_match() {
        let app = build_router(sample_config());

        let request_body = serde_json::json!({
            "white_engine_id": "stockfish-16",
            "black_engine_id": "lc0-0.30",
            "time_control": { "initial_ms": 300000 }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/match")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), HttpStatus::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: MatchCreateResponse = serde_json::from_slice(&body).unwrap();

        assert!(!payload.match_id.is_empty());
    }

    #[tokio::test]
    async fn post_match_rejects_unknown_engine() {
        let app = build_router(sample_config());

        let request_body = serde_json::json!({
            "white_engine_id": "unknown",
            "black_engine_id": "lc0-0.30",
            "time_control": { "initial_ms": 300000 }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/match")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), HttpStatus::BAD_REQUEST);
    }
}
