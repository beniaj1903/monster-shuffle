use axum::{extract::State, http::StatusCode, response::Json};
use core::battle::{execute_enemy_attack, execute_turn, TurnResult};
use core::experience::apply_victory_level_up;
use core::game::{GameSession, GameState};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

/// Payload para enviar un movimiento en batalla
#[derive(Deserialize, Debug)]
pub struct SubmitMoveRequest {
    pub session_id: String,
    /// Índice del movimiento elegido (0-3)
    pub move_index: usize,
}

/// Respuesta del endpoint de batalla
#[derive(Serialize, Debug)]
pub struct SubmitMoveResponse {
    pub result: TurnResult,
    pub player_hp: u16,
    pub enemy_hp: u16,
    pub battle_over: bool,
    pub player_won: Option<bool>,
    /// Sesión actualizada (solo cuando la batalla termina)
    pub session: Option<GameSession>,
}

/// Handler para ejecutar un movimiento en batalla
/// 
/// POST /api/game/battle/move
/// 
/// Ejecuta un turno de batalla con el movimiento elegido por el jugador.
pub async fn submit_move(
    State(state): State<AppState>,
    Json(payload): Json<SubmitMoveRequest>,
) -> Result<Json<SubmitMoveResponse>, StatusCode> {
    // Buscar la sesión
    let mut session = state
        .sessions
        .get(&payload.session_id)
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();

    // Validar que esté en estado de batalla (normal o gimnasio)
    if session.state != GameState::Battle && session.state != GameState::GymBattle {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Obtener el estado de batalla (clonar para poder modificarlo)
    let mut battle_state = session
        .battle
        .clone()
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Obtener el Pokémon activo del jugador
    let player_active_index = battle_state.player_active_index;
    if player_active_index >= session.team.active_members.len() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validación: Verificar que el Pokémon activo no esté debilitado
    if session.team.active_members[player_active_index].current_hp == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut player_mon = session.team.active_members[player_active_index].clone();
    let mut enemy_mon = battle_state.get_opponent_active().clone();

    // Obtener el movimiento elegido por el jugador
    // Solo permitir seleccionar entre los primeros 4 movimientos activos
    let active_moves = player_mon.get_active_moves();
    if payload.move_index >= active_moves.len() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let player_move_id = &active_moves[payload.move_index].template_id;
    let player_move = state
        .moves
        .get(player_move_id)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // IA Enemiga: Elegir un movimiento aleatorio del enemigo
    // Solo usar los primeros 4 movimientos activos
    let enemy_active_moves = enemy_mon.get_active_moves();
    if enemy_active_moves.is_empty() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let mut rng = StdRng::from_entropy();
    
    // Si es batalla contra entrenador, usar el Pokémon activo del oponente
    let enemy_move_index = rng.gen_range(0..enemy_active_moves.len());
    let enemy_move_id = &enemy_active_moves[enemy_move_index].template_id;
    let enemy_move = state
        .moves
        .get(enemy_move_id)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Inicializar battle_stages si no están inicializados (al entrar en batalla)
    if player_mon.battle_stages.is_none() {
        player_mon.init_battle_stages();
    }
    if enemy_mon.battle_stages.is_none() {
        enemy_mon.init_battle_stages();
    }

    // Ejecutar el turno
    let turn_result = execute_turn(
        &mut player_mon,
        &mut enemy_mon,
        player_move,
        enemy_move,
        &session.team,
        player_active_index,
        &mut battle_state,
        &mut rng,
    );

    // Actualizar los Pokémon en la sesión
    session.team.active_members[player_active_index] = player_mon.clone();
    
    // IMPORTANTE: Solo actualizar el oponente si NO cambió de Pokémon
    // Si cambió (EnemySwitched), el battle_state ya tiene el nuevo Pokémon activo
    if turn_result.outcome != core::battle::BattleOutcome::EnemySwitched {
        *battle_state.get_opponent_active_mut() = enemy_mon.clone();
    }
    // Si es EnemySwitched, el battle_state ya tiene el nuevo oponente activo
    // y enemy_mon sigue siendo el Pokémon anterior (debilitado), así que NO lo actualizamos
    
    battle_state.turn_counter += 1;

    // Añadir logs al estado de batalla
    for log in &turn_result.logs {
        battle_state.add_log(log.clone());
    }

    // Manejar el resultado del turno
    match turn_result.outcome {
        core::battle::BattleOutcome::Continue => {
            // La batalla continúa normalmente
            session.battle = Some(battle_state.clone());
            // IMPORTANTE: Guardar la sesión antes de retornar
            state.sessions.insert(payload.session_id.clone(), session.clone());
            return Ok(Json(SubmitMoveResponse {
                result: turn_result,
                player_hp: player_mon.current_hp,
                enemy_hp: enemy_mon.current_hp,
                battle_over: false,
                player_won: None,
                session: None,
            }));
        }
        core::battle::BattleOutcome::EnemySwitched => {
            // El enemigo cambió de Pokémon, la batalla continúa
            // IMPORTANTE: El battle_state ya tiene el nuevo oponente activo (switch_to_next_opponent ya se ejecutó)
            // Necesitamos sincronizar opponent_instance para que el frontend lo vea
            battle_state.sync_opponent_instance();
            
            let next_opponent = battle_state.get_opponent_active().clone();
            let opponent_name = battle_state.opponent_name.clone().unwrap_or_else(|| "El entrenador".to_string());
            battle_state.add_log(format!(
                "{} envió a {}!",
                opponent_name,
                next_opponent.species.display_name
            ));
            session.battle = Some(battle_state.clone());
            // IMPORTANTE: Guardar la sesión antes de retornar
            state.sessions.insert(payload.session_id.clone(), session.clone());
            return Ok(Json(SubmitMoveResponse {
                result: turn_result,
                player_hp: player_mon.current_hp,
                enemy_hp: next_opponent.current_hp,
                battle_over: false,
                player_won: None,
                session: None,
            }));
        }
        core::battle::BattleOutcome::PlayerMustSwitch => {
            // El jugador debe cambiar de Pokémon
            // IMPORTANTE: NO cambiar el estado a Map, mantener Battle/GymBattle
            battle_state.add_log("¡Necesitas cambiar de Pokémon!".to_string());
            // Actualizar el estado de batalla con el Pokémon debilitado
            session.team.active_members[player_active_index] = player_mon.clone();
            *battle_state.get_opponent_active_mut() = enemy_mon.clone();
            session.battle = Some(battle_state.clone());
            
            // Guardar la sesión actualizada
            state.sessions.insert(payload.session_id.clone(), session.clone());
            
            return Ok(Json(SubmitMoveResponse {
                result: turn_result,
                player_hp: player_mon.current_hp, // Será 0
                enemy_hp: enemy_mon.current_hp,
                battle_over: false, // La batalla continúa, pero el jugador debe cambiar
                player_won: None,
                session: None,
            }));
        }
        core::battle::BattleOutcome::PlayerWon => {
            // El oponente no tiene más Pokémon (o era salvaje) - Jugador ganó la batalla completa
            battle_state.add_log("¡Has ganado la batalla!".to_string());

            // Determinar si fue una batalla de gimnasio
            let is_gym_victory = battle_state.is_trainer_battle;

            // EXPERIENCIA GLOBAL (Exp Share): Aplicar subida de nivel a TODO el equipo
            for team_member in &mut session.team.active_members {
                let species = state.pokedex
                    .get(&team_member.species.species_id)
                    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
                
                let levels_gained = apply_victory_level_up(
                    team_member,
                    species,
                    &session.config,
                    is_gym_victory,
                );
                
                if levels_gained > 0 {
                    let new_level = team_member.level;
                    battle_state.add_log(format!(
                        "¡{} subió al nivel {}!",
                        team_member.species.display_name,
                        new_level
                    ));
                }
            }

            // AUTO-CURACIÓN: Restaurar completamente a todo el equipo después de la victoria
            session.team.heal_all();
            battle_state.add_log("¡El equipo ha recuperado toda su energía!".to_string());

            // PROGRESIÓN DE MOVIMIENTOS: Aprender nuevos movimientos después de la victoria
            // Condición: Es victoria de Gym O cada 2 combates salvajes (encounters_won % 2 == 0)
            let should_learn_moves = is_gym_victory || (session.encounters_won % 2 == 0);
            
            if should_learn_moves {
                // Obtener el pool global de movimientos
                let global_move_pool: Vec<String> = state.moves.keys().cloned().collect();
                
                if !global_move_pool.is_empty() {
                    let mut rng = StdRng::from_entropy();
                    let mut moves_learned = false;
                    
                    // Iterar sobre el equipo activo del jugador
                    for pokemon in &mut session.team.active_members {
                        // Obtener los IDs de movimientos que el Pokémon ya tiene aprendidos
                        let learned_move_ids: std::collections::HashSet<String> = pokemon
                            .randomized_profile
                            .moves
                            .iter()
                            .map(|m| m.template_id.clone())
                            .collect();
                        
                        // Filtrar movimientos disponibles (que no tenga ya aprendidos)
                        let available_moves: Vec<String> = global_move_pool
                            .iter()
                            .filter(|move_id| !learned_move_ids.contains(*move_id))
                            .cloned()
                            .collect();
                        
                        // Si hay movimientos disponibles, seleccionar uno al azar
                        if !available_moves.is_empty() {
                            let random_move = available_moves
                                .choose(&mut rng)
                                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
                            
                            pokemon.learn_new_move(random_move.clone());
                            moves_learned = true;
                        }
                    }
                    
                    if moves_learned {
                        battle_state.add_log("¡El equipo aprendió nuevos movimientos!".to_string());
                    }
                }
            }

            // Si es una batalla salvaje (no entrenador), añadir el Pokémon al equipo
            if !is_gym_victory {
                // Obtener el Pokémon oponente (ya derrotado, pero lo clonamos para añadirlo)
                let mut defeated_pokemon = battle_state.get_opponent_active().clone();
                
                // Nivelar el Pokémon capturado al nivel del Pokémon activo del jugador
                let player_active_level = session.team.active_members[player_active_index].level;
                defeated_pokemon.set_level(player_active_level);
                
                // Asegurar que el Pokémon capturado tenga la misma cantidad de movimientos que el Pokémon activo
                let player_active_pokemon = &session.team.active_members[player_active_index];
                let target_move_count = player_active_pokemon.randomized_profile.moves.len();
                let current_move_count = defeated_pokemon.randomized_profile.moves.len();
                
                if current_move_count < target_move_count {
                    // Obtener el pool global de movimientos
                    let global_move_pool: Vec<String> = state.moves.keys().cloned().collect();
                    
                    if !global_move_pool.is_empty() {
                        let mut rng = StdRng::from_entropy();
                        
                        // Añadir movimientos aleatorios hasta igualar la cantidad
                        let moves_to_add = target_move_count - current_move_count;
                        let mut added_count = 0;
                        
                        for _ in 0..moves_to_add {
                            // Obtener los IDs de movimientos que el Pokémon ya tiene aprendidos (actualizado en cada iteración)
                            let learned_move_ids: std::collections::HashSet<String> = defeated_pokemon
                                .randomized_profile
                                .moves
                                .iter()
                                .map(|m| m.template_id.clone())
                                .collect();
                            
                            // Filtrar movimientos disponibles (que no tenga ya aprendidos)
                            let available_moves: Vec<String> = global_move_pool
                                .iter()
                                .filter(|move_id| !learned_move_ids.contains(*move_id))
                                .cloned()
                                .collect();
                            
                            if available_moves.is_empty() {
                                break; // No hay más movimientos únicos disponibles
                            }
                            
                            if let Some(random_move) = available_moves.choose(&mut rng) {
                                defeated_pokemon.learn_new_move(random_move.clone());
                                added_count += 1;
                            }
                        }
                        
                        if added_count > 0 {
                            battle_state.add_log(format!(
                                "{} aprendió {} movimientos nuevos al ser capturado!",
                                defeated_pokemon.species.display_name,
                                added_count
                            ));
                        }
                    }
                }
                
                // Curar al Pokémon recién capturado antes de añadirlo al equipo
                defeated_pokemon.full_restore();
                session.team.add_member(defeated_pokemon);
            }

            // Incrementar contador de encuentros ganados
            session.encounters_won += 1;

            // Verificar si se completó la partida
            if session.encounters_won >= session.config.total_encounters {
                session.state = GameState::Completed;
            } else {
                // Volver al mapa
                session.state = GameState::Map;
            }
            
            // Limpiar el estado de batalla
            session.battle = None;
            
            // IMPORTANTE: Guardar la sesión actualizada antes de retornar
            state.sessions.insert(payload.session_id.clone(), session.clone());
            
            return Ok(Json(SubmitMoveResponse {
                result: turn_result,
                player_hp: player_mon.current_hp,
                enemy_hp: enemy_mon.current_hp,
                battle_over: true,
                player_won: Some(true),
                session: Some(session),
            }));
        }
        core::battle::BattleOutcome::PlayerLost => {
            // Todos los Pokémon del jugador están debilitados
            battle_state.add_log("Has sido derrotado...".to_string());

            // IMPORTANTE: Revivir TODOS los Pokémon del equipo con 1 HP (no solo el activo)
            // Esto asegura que el jugador pueda continuar jugando
            for team_member in &mut session.team.active_members {
                if team_member.current_hp == 0 {
                    team_member.current_hp = 1;
                }
            }
            
            // Asegurar que el estado del juego se mantenga (no reiniciar)
            // El jugador ya eligió su starter, así que NO debe volver a StarterSelection
            if session.state == GameState::Battle || session.state == GameState::GymBattle {
                session.state = GameState::Map;
            }
            session.battle = None;
            
            // IMPORTANTE: Guardar la sesión actualizada antes de retornar
            state.sessions.insert(payload.session_id.clone(), session.clone());
            
            return Ok(Json(SubmitMoveResponse {
                result: turn_result,
                player_hp: 1, // El Pokémon activo ahora tiene 1 HP
                enemy_hp: enemy_mon.current_hp,
                battle_over: true,
                player_won: Some(false),
                session: Some(session),
            }));
        }
    }
}

/// Payload para cambiar de Pokémon durante la batalla
#[derive(Deserialize, Debug)]
pub struct SwitchPokemonRequest {
    pub session_id: String,
    /// Índice del Pokémon al que cambiar en session.team.active_members
    pub switch_to_index: usize,
}

/// Respuesta del endpoint de cambio de Pokémon
#[derive(Serialize, Debug)]
pub struct SwitchPokemonResponse {
    /// Resultado del turno (ataque del enemigo)
    pub result: TurnResult,
    /// HP del jugador después del cambio y ataque
    pub player_hp: u16,
    /// HP del enemigo
    pub enemy_hp: u16,
    /// Indica si la batalla terminó
    pub battle_over: bool,
    /// Indica si el jugador ganó (None si la batalla continúa)
    pub player_won: Option<bool>,
    /// Sesión actualizada
    pub session: GameSession,
}

/// Handler para cambiar de Pokémon durante la batalla
/// 
/// POST /api/game/battle/switch
/// 
/// Permite al jugador cambiar su Pokémon activo durante la batalla.
pub async fn switch_pokemon(
    State(state): State<AppState>,
    Json(payload): Json<SwitchPokemonRequest>,
) -> Result<Json<SwitchPokemonResponse>, StatusCode> {
    // Buscar la sesión
    let mut session = state
        .sessions
        .get(&payload.session_id)
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();

    // Validar que esté en estado de batalla (normal o gimnasio)
    if session.state != GameState::Battle && session.state != GameState::GymBattle {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Obtener el estado de batalla (clonar para poder modificarlo)
    let mut battle_state = session
        .battle
        .clone()
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Validar que el índice sea válido
    if payload.switch_to_index >= session.team.active_members.len() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validar que no sea el mismo Pokémon que ya está activo
    if payload.switch_to_index == battle_state.player_active_index {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validar que el Pokémon al que se quiere cambiar no esté debilitado
    let target_pokemon = &session.team.active_members[payload.switch_to_index];
    if target_pokemon.current_hp == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Obtener el Pokémon actual ANTES del cambio para determinar si es forzado
    let current_active = &session.team.active_members[battle_state.player_active_index];
    let is_forced_switch = current_active.current_hp == 0;

    // Obtener el nombre del Pokémon actual y del nuevo
    let current_pokemon_name = current_active.species.display_name.clone();
    let new_pokemon_name = target_pokemon.species.display_name.clone();

    // Cambiar el Pokémon activo
    battle_state.player_active_index = payload.switch_to_index;

    // Obtener el nuevo Pokémon activo (mutable)
    let new_active_index = battle_state.player_active_index;
    let mut new_active_pokemon = session.team.active_members[new_active_index].clone();
    let mut enemy_mon = battle_state.get_opponent_active().clone();

    // Añadir log del cambio
    if is_forced_switch {
        // Cambio forzado: solo mostrar el cambio
        battle_state.add_log(format!(
            "¡Ve, {}!",
            new_pokemon_name
        ));
    } else {
        // Cambio táctico: mostrar retiro y entrada
        battle_state.add_log(format!(
            "¡Jugador retiró a {}!",
            current_pokemon_name
        ));
        battle_state.add_log(format!(
            "¡Ve, {}!",
            new_pokemon_name
        ));
    }

    // Lógica condicional de ataque
    let turn_result = if is_forced_switch {
        // Cambio forzado: El enemigo NO ataca, espera
        // El jugador no gasta su turno cambiando
        TurnResult::new()
    } else {
        // Cambio táctico: El enemigo SÍ ataca (el jugador gasta su turno cambiando)
        // Seleccionar un movimiento aleatorio del enemigo
        let enemy_active_moves = enemy_mon.get_active_moves();
        if enemy_active_moves.is_empty() {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        let mut rng = StdRng::from_entropy();
        let enemy_move_index = rng.gen_range(0..enemy_active_moves.len());
        let enemy_move_id = &enemy_active_moves[enemy_move_index].template_id;
        let enemy_move = state
            .moves
            .get(enemy_move_id)
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // Inicializar battle_stages si no están inicializados
        if new_active_pokemon.battle_stages.is_none() {
            new_active_pokemon.init_battle_stages();
        }
        if enemy_mon.battle_stages.is_none() {
            enemy_mon.init_battle_stages();
        }

        // Ejecutar el ataque del enemigo
        execute_enemy_attack(
            &mut new_active_pokemon,
            &mut enemy_mon,
            enemy_move,
            &mut rng,
        )
    };

    // Actualizar los Pokémon en la sesión
    session.team.active_members[new_active_index] = new_active_pokemon.clone();
    
    // Actualizar el oponente en el battle_state (solo si hubo ataque)
    if !is_forced_switch {
        if battle_state.is_trainer_battle {
            battle_state.opponent_team[battle_state.opponent_active_index] = enemy_mon.clone();
        } else {
            battle_state.opponent_instance = enemy_mon.clone();
        }
        battle_state.turn_counter += 1;
    }

    // Añadir logs al estado de batalla (solo si hubo ataque)
    if !is_forced_switch {
        for log in &turn_result.logs {
            battle_state.add_log(log.clone());
        }
    }

    // Manejar el resultado del ataque del enemigo (solo si hubo ataque)
    let mut player_won = None;
    let battle_over = if is_forced_switch {
        false // Cambio forzado: la batalla continúa
    } else {
        matches!(
            turn_result.outcome,
            core::battle::BattleOutcome::PlayerWon | core::battle::BattleOutcome::PlayerLost
        )
    };

    if !is_forced_switch {
        match turn_result.outcome {
            core::battle::BattleOutcome::PlayerLost => {
                // Jugador perdió (todos los Pokémon debilitados)
                player_won = Some(false);
                battle_state.add_log("Has sido derrotado...".to_string());

                // IMPORTANTE: Revivir TODOS los Pokémon del equipo con 1 HP (no solo el activo)
                for team_member in &mut session.team.active_members {
                    if team_member.current_hp == 0 {
                        team_member.current_hp = 1;
                    }
                }
                
                // Asegurar que el estado del juego se mantenga (no reiniciar)
                if session.state == GameState::Battle || session.state == GameState::GymBattle {
                    session.state = GameState::Map;
                }
                session.battle = None;
            }
            core::battle::BattleOutcome::PlayerMustSwitch => {
                // El nuevo Pokémon se debilitó, pero el jugador tiene más disponibles
                battle_state.add_log("¡Necesitas cambiar de Pokémon!".to_string());
            }
            _ => {
                // La batalla continúa normalmente
            }
        }
    }

    // Sincronizar opponent_instance
    battle_state.sync_opponent_instance();

    // Actualizar el battle_state en la sesión
    if battle_over && player_won == Some(false) {
        // Si el jugador perdió, limpiar el estado de batalla
        session.battle = None;
    } else {
        // Actualizar el estado de batalla
        session.battle = Some(battle_state);
    }

    // Guardar la sesión actualizada
    state.sessions.insert(payload.session_id.clone(), session.clone());

    // Obtener el HP del enemigo (puede haber cambiado si hubo ataque)
    let enemy_hp = if is_forced_switch {
        // Si es cambio forzado, obtener el HP del enemigo desde la sesión actualizada
        session.battle.as_ref()
            .map(|b| b.get_opponent_active().current_hp)
            .unwrap_or(enemy_mon.current_hp)
    } else {
        enemy_mon.current_hp
    };

    Ok(Json(SwitchPokemonResponse {
        result: turn_result,
        player_hp: new_active_pokemon.current_hp,
        enemy_hp,
        battle_over,
        player_won,
        session,
    }))
}

