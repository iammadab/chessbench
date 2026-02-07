use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;

use shakmaty::fen::Fen;
use shakmaty::san::San;
use shakmaty::uci::UciMove;
use shakmaty::{Chess, Color, EnPassantMode, Outcome, Position};

use crate::domain::{Clock, MatchResult, MatchState, MatchStatus, ResultReason, Side};
use crate::engine::EngineSpec;
use crate::uci::{UciError, UciProcess};

pub async fn run_match(
    match_id: String,
    white: EngineSpec,
    black: EngineSpec,
    initial_ms: u64,
    matches: Arc<RwLock<HashMap<String, MatchState>>>,
) {
    let match_id_clone = match_id.clone();
    if let Err(err) = run_match_inner(match_id_clone, white, black, initial_ms, matches.clone()).await {
        let mut guard = matches.write().await;
        if let Some(entry) = guard.get_mut(&match_id) {
            entry.status = MatchStatus::Error;
            entry.result = Some(MatchResult {
                result: "*".to_string(),
                reason: ResultReason::Error,
            });
        }
        eprintln!("match runner error: {err}");
    }
}

async fn run_match_inner(
    match_id: String,
    white: EngineSpec,
    black: EngineSpec,
    initial_ms: u64,
    matches: Arc<RwLock<HashMap<String, MatchState>>>,
) -> Result<(), UciError> {
    let mut white_engine = UciProcess::spawn(&white.path, &white.args, white.working_dir.as_ref()).await?;
    let mut black_engine = UciProcess::spawn(&black.path, &black.args, black.working_dir.as_ref()).await?;

    let _ = white_engine.handshake().await;
    let _ = black_engine.handshake().await;
    let _ = white_engine.is_ready().await;
    let _ = black_engine.is_ready().await;
    let _ = white_engine.ucinewgame().await;
    let _ = black_engine.ucinewgame().await;

    let mut pos = Chess::default();
    let mut white_ms = initial_ms;
    let mut black_ms = initial_ms;
    let mut ply: u32 = 0;
    let mut moves: Vec<String> = Vec::new();

    loop {
        let side = if pos.turn() == Color::White { Side::White } else { Side::Black };
        let (engine, remaining_ms) = match side {
            Side::White => (&mut white_engine, white_ms),
            Side::Black => (&mut black_engine, black_ms),
        };

        if remaining_ms == 0 {
            finish_match(&match_id, side, ResultReason::Timeout, &matches).await;
            break;
        }

        let fen = Fen::from_position(pos.clone(), EnPassantMode::Legal).to_string();
        let position_cmd = format!("position fen {fen}");
        engine.send_line(&position_cmd).await?;

        let start = Instant::now();
        let bestmove = match engine.bestmove(white_ms, black_ms, remaining_ms).await {
            Ok(bestmove) => bestmove,
            Err(UciError::Timeout(_)) => {
                finish_match(&match_id, side, ResultReason::Timeout, &matches).await;
                break;
            }
            Err(err) => return Err(err),
        };

        let elapsed_ms = start.elapsed().as_millis() as u64;
        match side {
            Side::White => white_ms = white_ms.saturating_sub(elapsed_ms),
            Side::Black => black_ms = black_ms.saturating_sub(elapsed_ms),
        }

        if bestmove == "(none)" {
            if let Some(outcome) = pos.outcome() {
                finish_with_outcome(&match_id, outcome, &pos, &matches).await;
            } else {
                finish_match(&match_id, side, ResultReason::Error, &matches).await;
            }
            break;
        }

        let uci_move = bestmove.parse::<UciMove>().map_err(|_| UciError::InvalidResponse(bestmove.clone()))?;
        let mv = match uci_move.to_move(&pos) {
            Ok(mv) => mv,
            Err(_) => {
                finish_match(&match_id, side, ResultReason::Illegal, &matches).await;
                break;
            }
        };

        let san = San::from_move(&pos, &mv).to_string();
        let pos_next = match pos.play(&mv) {
            Ok(pos_next) => pos_next,
            Err(_) => {
                finish_match(&match_id, side, ResultReason::Illegal, &matches).await;
                break;
            }
        };

        ply += 1;
        moves.push(san.clone());
        let pgn = format_pgn(&moves);
        let fen_next = Fen::from_position(pos_next.clone(), EnPassantMode::Legal).to_string();

        update_match_state(
            &matches,
            &match_id,
            ply,
            &fen_next,
            &pgn,
            white_ms,
            black_ms,
            bestmove.clone(),
            san,
        )
        .await;

        pos = pos_next;

        if let Some(outcome) = pos.outcome() {
            finish_with_outcome(&match_id, outcome, &pos, &matches).await;
            break;
        }
    }

    let _ = white_engine.quit().await;
    let _ = black_engine.quit().await;

    Ok(())
}

async fn update_match_state(
    matches: &Arc<RwLock<HashMap<String, MatchState>>>,
    match_id: &str,
    ply: u32,
    fen: &str,
    pgn: &str,
    white_ms: u64,
    black_ms: u64,
    uci: String,
    san: String,
) {
    let mut guard = matches.write().await;
    if let Some(entry) = guard.get_mut(match_id) {
        entry.ply = ply;
        entry.current_fen = fen.to_string();
        entry.pgn = pgn.to_string();
        entry.clocks = Clock { white_ms, black_ms };
        entry.last_move = Some(crate::domain::MoveSnapshot {
            ply,
            uci,
            san,
            fen: fen.to_string(),
            pgn: pgn.to_string(),
        });
    }
}

async fn finish_match(
    match_id: &str,
    offender: Side,
    reason: ResultReason,
    matches: &Arc<RwLock<HashMap<String, MatchState>>>,
) {
    let result = match offender {
        Side::White => "0-1",
        Side::Black => "1-0",
    };

    let mut guard = matches.write().await;
    if let Some(entry) = guard.get_mut(match_id) {
        entry.status = MatchStatus::Finished;
        entry.result = Some(MatchResult {
            result: result.to_string(),
            reason,
        });
    }
}

async fn finish_with_outcome(
    match_id: &str,
    outcome: Outcome,
    pos: &Chess,
    matches: &Arc<RwLock<HashMap<String, MatchState>>>,
) {
    let reason = if pos.is_checkmate() {
        ResultReason::Checkmate
    } else if pos.is_stalemate() {
        ResultReason::Stalemate
    } else {
        ResultReason::Draw
    };

    let result = outcome.as_str().to_string();
    let mut guard = matches.write().await;
    if let Some(entry) = guard.get_mut(match_id) {
        entry.status = MatchStatus::Finished;
        entry.result = Some(MatchResult { result, reason });
    }
}

fn format_pgn(moves: &[String]) -> String {
    let mut pgn = String::new();
    for (idx, san) in moves.iter().enumerate() {
        if idx % 2 == 0 {
            if !pgn.is_empty() {
                pgn.push(' ');
            }
            let move_no = idx / 2 + 1;
            pgn.push_str(&format!("{move_no}. {san}"));
        } else {
            pgn.push(' ');
            pgn.push_str(san);
        }
    }
    pgn
}
