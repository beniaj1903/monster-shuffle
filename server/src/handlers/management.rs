use axum::{extract::State, http::StatusCode, response::Json};
use core::factory::compute_stats;
use core::game::GameSession;
use serde::Deserialize;

use crate::state::AppState;

/// Payload para reordenar el equipo
#[derive(Deserialize, Debug)]
pub struct ReorderTeamRequest {
    pub session_id: String,
    /// Nuevo orden de los índices (ej: [2, 0, 1, 3, 4, 5])
    pub new_order: Vec<usize>,
}

/// Payload para reordenar los movimientos de un Pokémon
#[derive(Deserialize, Debug)]
pub struct ReorderMovesRequest {
    pub session_id: String,
    /// Índice del Pokémon en el equipo activo
    pub pokemon_index: usize,
    /// Nuevo orden de los índices de movimientos (ej: [3, 0, 1, 2])
    pub move_indices: Vec<usize>,
}

/// Payload para evolucionar un Pokémon
#[derive(Deserialize, Debug)]
pub struct EvolvePokemonRequest {
    pub session_id: String,
    /// Índice del Pokémon en el equipo activo
    pub pokemon_index: usize,
    /// ID de la especie objetivo de la evolución
    pub target_species_id: String,
}

/// Handler para reordenar el equipo
/// 
/// POST /api/game/team/reorder
pub async fn reorder_team(
    State(state): State<AppState>,
    Json(payload): Json<ReorderTeamRequest>,
) -> Result<Json<GameSession>, StatusCode> {
    // Buscar la sesión
    let mut session = state
        .sessions
        .get(&payload.session_id)
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();

    let team_size = session.team.active_members.len();

    // Validar que new_order tenga la misma longitud que el equipo
    if payload.new_order.len() != team_size {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validar que todos los índices sean válidos y únicos
    let mut seen = vec![false; team_size];
    for &index in &payload.new_order {
        if index >= team_size {
            return Err(StatusCode::BAD_REQUEST);
        }
        if seen[index] {
            return Err(StatusCode::BAD_REQUEST); // Índice duplicado
        }
        seen[index] = true;
    }

    // Reordenar el equipo según los índices recibidos
    let mut reordered_team = Vec::with_capacity(team_size);
    for &index in &payload.new_order {
        reordered_team.push(session.team.active_members[index].clone());
    }
    session.team.active_members = reordered_team;

    // Actualizar el índice activo si es necesario (ajustar si el Pokémon activo cambió de posición)
    // Por ahora, simplemente mantenemos el índice 0 como activo si hay un battle_state
    if let Some(ref mut battle_state) = session.battle {
        // Buscar el nuevo índice del Pokémon que estaba activo
        // Por simplicidad, asumimos que el primer Pokémon en el nuevo orden es el activo
        // Actualizar el primer slot (en Single solo hay uno)
        if let Some(first) = battle_state.player_active_indices.first_mut() {
            *first = 0;
        } else {
            battle_state.player_active_indices.push(0);
        }
    }

    // Guardar la sesión actualizada
    state.sessions.insert(payload.session_id.clone(), session.clone());

    Ok(Json(session))
}

/// Handler para reordenar los movimientos de un Pokémon
/// 
/// POST /api/game/pokemon/move-reorder
pub async fn reorder_moves(
    State(state): State<AppState>,
    Json(payload): Json<ReorderMovesRequest>,
) -> Result<Json<GameSession>, StatusCode> {
    // Buscar la sesión
    let mut session = state
        .sessions
        .get(&payload.session_id)
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();

    // Validar que el índice del Pokémon sea válido
    if payload.pokemon_index >= session.team.active_members.len() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let pokemon = &mut session.team.active_members[payload.pokemon_index];
    let moves_count = pokemon.randomized_profile.moves.len();

    // Validar que move_indices tenga la misma longitud que los movimientos
    if payload.move_indices.len() != moves_count {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validar que todos los índices sean válidos y únicos
    let mut seen = vec![false; moves_count];
    for &index in &payload.move_indices {
        if index >= moves_count {
            return Err(StatusCode::BAD_REQUEST);
        }
        if seen[index] {
            return Err(StatusCode::BAD_REQUEST); // Índice duplicado
        }
        seen[index] = true;
    }

    // Reordenar los movimientos según los índices recibidos
    let mut reordered_moves = Vec::with_capacity(moves_count);
    for &index in &payload.move_indices {
        reordered_moves.push(pokemon.randomized_profile.moves[index].clone());
    }
    pokemon.randomized_profile.moves = reordered_moves;

    // Guardar la sesión actualizada
    state.sessions.insert(payload.session_id.clone(), session.clone());

    Ok(Json(session))
}

/// Handler para evolucionar un Pokémon
/// 
/// POST /api/game/pokemon/evolve
pub async fn evolve_pokemon(
    State(state): State<AppState>,
    Json(payload): Json<EvolvePokemonRequest>,
) -> Result<Json<GameSession>, StatusCode> {
    // Buscar la sesión
    let mut session = state
        .sessions
        .get(&payload.session_id)
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();

    // Validar que el índice del Pokémon sea válido
    if payload.pokemon_index >= session.team.active_members.len() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let pokemon = &session.team.active_members[payload.pokemon_index].clone();
    
    // Obtener la especie actual del Pokémon
    let current_species = state
        .pokedex
        .get(&pokemon.species.species_id)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Buscar la evolución en la lista de evoluciones
    let evolution = current_species
        .evolutions
        .iter()
        .find(|evo| evo.target_species_id == payload.target_species_id)
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Validar el nivel mínimo (si aplica)
    if let Some(min_level) = evolution.min_level {
        if pokemon.level < min_level {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // Obtener la nueva especie del pokedex
    let new_species = state
        .pokedex
        .get(&payload.target_species_id)
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Clonar el Pokémon para poder modificarlo
    let mut evolved_pokemon = pokemon.clone();

    // Calcular el ratio de HP actual antes de cambiar los stats
    let hp_ratio = if evolved_pokemon.base_computed_stats.hp > 0 {
        evolved_pokemon.current_hp as f32 / evolved_pokemon.base_computed_stats.hp as f32
    } else {
        1.0
    };

    // Actualizar la especie
    evolved_pokemon.species = new_species.clone();

    // Recalcular stats usando los mismos IVs, EVs y nivel, pero con las stats base de la nueva especie
    evolved_pokemon.base_computed_stats = compute_stats(
        &new_species.base_stats,
        &evolved_pokemon.individual_values,
        &evolved_pokemon.effort_values,
        evolved_pokemon.level,
    );

    // Ajustar HP proporcionalmente al nuevo max_hp
    let new_max_hp = evolved_pokemon.base_computed_stats.hp;
    evolved_pokemon.current_hp = (new_max_hp as f32 * hp_ratio) as u16;

    // Mantener los movimientos actuales (por ahora, sin aprender nuevos movimientos)

    // Actualizar el Pokémon en el equipo
    session.team.active_members[payload.pokemon_index] = evolved_pokemon;

    // Guardar la sesión actualizada
    state.sessions.insert(payload.session_id.clone(), session.clone());

    Ok(Json(session))
}

