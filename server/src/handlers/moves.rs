use axum::{extract::State, http::StatusCode, response::Json};
use core::models::MoveData;
use serde::Serialize;
use std::collections::HashMap;

use crate::state::AppState;

/// Respuesta del endpoint de movimientos
#[derive(Serialize, Debug)]
pub struct MovesResponse {
    pub moves: HashMap<String, MoveData>,
}

/// Handler para obtener todos los movimientos
/// 
/// GET /api/moves
/// 
/// Devuelve todos los movimientos disponibles en el sistema.
pub async fn get_all_moves(
    State(state): State<AppState>,
) -> Result<Json<MovesResponse>, StatusCode> {
    // Clonar el HashMap interno del Arc
    let moves: HashMap<String, MoveData> = state.moves.as_ref().clone();
    Ok(Json(MovesResponse {
        moves,
    }))
}

