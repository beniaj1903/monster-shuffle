use axum::{extract::State, http::StatusCode, response::Json};
use core::battle::{execute_enemy_attack, execute_turn, trigger_on_entry_abilities, TurnResult, initialize_move_pp, has_moves_with_pp, create_struggle_move};
use core::experience::apply_victory_level_up;
use core::game::{GameSession, GameState, PendingPlayerAction};
use core::models::FieldPosition;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

/// Payload para enviar un movimiento en batalla
#[derive(Deserialize, Debug)]
pub struct SubmitMoveRequest {
    pub session_id: String,
    /// ID del movimiento seleccionado
    pub move_id: String,
    /// Índice del Pokémon del jugador que ejecuta la acción (0-1 en dobles)
    pub user_index: usize,
    /// Posición del objetivo (si aplica, para movimientos "selected-pokemon")
    pub target_position: Option<FieldPosition>,
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
    /// Indica si el turno se ejecutó o si faltan más acciones (en dobles)
    pub turn_executed: bool,
    /// Número de acciones pendientes (0 si el turno se ejecutó)
    pub pending_actions: usize,
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
    eprintln!("[DEBUG] submit_move: ===== INICIO submit_move =====");
    eprintln!("[DEBUG] submit_move: Payload recibido completo: {:?}", payload);
    eprintln!("[DEBUG] submit_move: session_id: {}", payload.session_id);
    eprintln!("[DEBUG] submit_move: move_id: {}", payload.move_id);
    eprintln!("[DEBUG] submit_move: user_index: {}", payload.user_index);
    eprintln!("[DEBUG] submit_move: target_position: {:?}", payload.target_position);
    
    // Buscar la sesión
    let mut session = match state.sessions.get(&payload.session_id) {
        Some(s) => {
            eprintln!("[DEBUG] submit_move: Sesión encontrada");
            s.clone()
        }
        None => {
            eprintln!("[DEBUG] submit_move: ERROR - Sesión no encontrada: {}", payload.session_id);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    
    eprintln!("[DEBUG] submit_move: Sesión estado: {:?}", session.state);

    // Validar que esté en estado de batalla (normal o gimnasio)
    if session.state != GameState::Battle && session.state != GameState::GymBattle {
        eprintln!("[DEBUG] submit_move: ERROR - Estado inválido: {:?} (esperado Battle o GymBattle)", session.state);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Obtener el estado de batalla (clonar para poder modificarlo)
    let mut battle_state = match session.battle.clone() {
        Some(bs) => {
            eprintln!("[DEBUG] submit_move: BattleState encontrado");
            bs
        }
        None => {
            eprintln!("[DEBUG] submit_move: ERROR - No hay BattleState en la sesión");
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    eprintln!("[DEBUG] submit_move: player_active_indices: {:?}", battle_state.player_active_indices);
    eprintln!("[DEBUG] submit_move: player_active_indices.len(): {}", battle_state.player_active_indices.len());

    // Validar que user_index corresponde a un Pokémon activo
    if payload.user_index >= battle_state.player_active_indices.len() {
        eprintln!("[DEBUG] submit_move: ERROR - user_index ({}) >= player_active_indices.len() ({})", 
            payload.user_index, battle_state.player_active_indices.len());
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let player_active_index = battle_state.player_active_indices[payload.user_index];
    eprintln!("[DEBUG] submit_move: player_active_index (del array): {}", player_active_index);
    eprintln!("[DEBUG] submit_move: session.team.active_members.len(): {}", session.team.active_members.len());
    
    if player_active_index >= session.team.active_members.len() {
        eprintln!("[DEBUG] submit_move: ERROR - player_active_index ({}) >= active_members.len() ({})", 
            player_active_index, session.team.active_members.len());
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validación: Verificar que el Pokémon activo no esté debilitado
    let player_hp = session.team.active_members[player_active_index].current_hp;
    eprintln!("[DEBUG] submit_move: HP del Pokémon activo: {}", player_hp);
    if player_hp == 0 {
        eprintln!("[DEBUG] submit_move: ERROR - Pokémon activo está debilitado (HP = 0)");
        return Err(StatusCode::BAD_REQUEST);
    }

    let player_mon = &session.team.active_members[player_active_index];
    eprintln!("[DEBUG] submit_move: Pokémon activo: {} (HP: {})", player_mon.species.display_name, player_mon.current_hp);
    
    // Verificar si el movimiento existe y tiene PP disponible
    let player_has_pp = has_moves_with_pp(player_mon);
    eprintln!("[DEBUG] submit_move: ¿Tiene movimientos con PP?: {}", player_has_pp);
    let move_id = if !player_has_pp {
        eprintln!("[DEBUG] submit_move: Sin PP disponible, forzando Struggle");
        "struggle".to_string()
    } else {
        // Verificar que el movimiento existe en los movimientos aprendidos
        let active_learned_moves = player_mon.get_active_learned_moves();
        let found_move = active_learned_moves.iter()
            .find(|m| m.move_id == payload.move_id);
        
        if let Some(learned_move) = found_move {
            eprintln!("[DEBUG] submit_move: Movimiento {} encontrado, PP: {}/{}", payload.move_id, learned_move.current_pp, learned_move.max_pp);
            // Verificar PP
            if learned_move.current_pp == 0 {
                eprintln!("[DEBUG] submit_move: Movimiento sin PP, forzando Struggle");
                "struggle".to_string()
            } else {
                payload.move_id.clone()
            }
        } else {
            eprintln!("[DEBUG] submit_move: Movimiento {} no encontrado en movimientos aprendidos, forzando Struggle", payload.move_id);
            // Movimiento no encontrado, usar Struggle
            "struggle".to_string()
        }
    };
    eprintln!("[DEBUG] submit_move: Movimiento final a usar: {}", move_id);

    // Crear la acción pendiente
    let action = PendingPlayerAction {
        user_index: payload.user_index,
        move_id: move_id.clone(),
        target_position: payload.target_position,
    };

    // Agregar o reemplazar la acción para este Pokémon
    // Si ya existe una acción para este user_index, reemplazarla
    battle_state.pending_player_actions.retain(|a| a.user_index != payload.user_index);
    battle_state.pending_player_actions.push(action);
    
    eprintln!("[DEBUG] submit_move: Acciones pendientes después de agregar: {}", battle_state.pending_player_actions.len());
    for (i, action) in battle_state.pending_player_actions.iter().enumerate() {
        eprintln!("[DEBUG] submit_move:   Acción {}: user_index={}, move_id={}, target={:?}", 
            i + 1, action.user_index, action.move_id, action.target_position);
    }

    // Verificar si todas las acciones están listas
    let required_actions = battle_state.player_active_indices.len();
    let pending_count = battle_state.pending_player_actions.len();
    
    eprintln!("[DEBUG] submit_move: Acciones requeridas: {}, Pendientes: {}", required_actions, pending_count);
    
    // Si no todas las acciones están listas, retornar sin ejecutar el turno
    if pending_count < required_actions {
        eprintln!("[DEBUG] submit_move: Faltan acciones, esperando más... (faltan: {})", required_actions - pending_count);
        session.battle = Some(battle_state.clone());
        state.sessions.insert(payload.session_id.clone(), session.clone());
        return Ok(Json(SubmitMoveResponse {
            result: TurnResult::new(),
            player_hp: player_mon.current_hp,
            enemy_hp: battle_state.get_opponent_active().current_hp,
            battle_over: false,
            player_won: None,
            session: None,
            turn_executed: false,
            pending_actions: required_actions - pending_count,
        }));
    }

    // Todas las acciones están listas, ejecutar el turno
    eprintln!("[DEBUG] submit_move: Todas las acciones están listas, ejecutando turno...");
    let mut rng = StdRng::from_entropy();
    
    // Obtener las acciones del jugador (ordenadas por user_index)
    battle_state.pending_player_actions.sort_by_key(|a| a.user_index);
    eprintln!("[DEBUG] submit_move: Acciones ordenadas para ejecución:");
    for (i, action) in battle_state.pending_player_actions.iter().enumerate() {
        eprintln!("[DEBUG] submit_move:   {}: user_index={}, move_id={}, target={:?}", 
            i + 1, action.user_index, action.move_id, action.target_position);
    }
    
    // Inicializar PP para todos los Pokémon activos del jugador
    for action in &battle_state.pending_player_actions {
        let player_index = battle_state.player_active_indices[action.user_index];
        if player_index < session.team.active_members.len() {
            let player_mon = &mut session.team.active_members[player_index];
            if action.move_id != "struggle" {
                if let Some(move_data) = state.moves.get(&action.move_id) {
                    initialize_move_pp(player_mon, &action.move_id, move_data);
                }
            }
        }
    }
    
    // Para compatibilidad con execute_turn actual, usamos la primera acción
    // execute_turn leerá las acciones pendientes desde battle_state
    let first_action = &battle_state.pending_player_actions[0];
    let first_player_index = battle_state.player_active_indices[first_action.user_index];
    let mut player_mon = session.team.active_members[first_player_index].clone();
    let mut enemy_mon = battle_state.get_opponent_active().clone();

    // Determinar el movimiento del jugador (para compatibilidad con la firma actual)
    let (player_move_template_id, use_struggle_player) = if first_action.move_id == "struggle" {
        ("struggle".to_string(), true)
    } else {
        (first_action.move_id.clone(), false)
    };

    // IA Enemiga: Elegir un movimiento aleatorio del enemigo
    // Primero inicializar PP de todos los movimientos del enemigo
    let enemy_learned_moves = enemy_mon.get_active_learned_moves();
    let enemy_move_ids: Vec<String> = enemy_learned_moves
        .iter()
        .map(|m| m.move_id.clone())
        .collect();
    
    // Inicializar PP de todos los movimientos del enemigo
    for move_id in &enemy_move_ids {
        if let Some(move_data) = state.moves.get(move_id) {
            initialize_move_pp(&mut enemy_mon, move_id, move_data);
        }
    }
    
    // Verificar si el enemigo tiene movimientos con PP disponible
    let enemy_has_pp = has_moves_with_pp(&enemy_mon);
    
    // Determinar el movimiento del enemigo
    let (enemy_move_template_id, use_struggle_enemy) = if !enemy_has_pp {
        // Sin PP disponible: usar Struggle
        ("struggle".to_string(), true)
    } else {
        // Filtrar movimientos con PP disponible
        let enemy_learned_moves_after = enemy_mon.get_active_learned_moves();
        let moves_with_pp: Vec<String> = enemy_learned_moves_after
            .iter()
            .filter(|m| m.current_pp > 0)
            .map(|m| m.move_id.clone())
            .collect();
        
        if moves_with_pp.is_empty() {
            // No hay movimientos con PP, usar Struggle
            ("struggle".to_string(), true)
        } else {
            // Seleccionar un movimiento aleatorio de los que tienen PP
            let move_id = moves_with_pp.choose(&mut rng)
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
            (move_id.clone(), false)
        }
    };
    
    // Obtener los MoveData (o crear Struggle)
    let player_move = if use_struggle_player {
        create_struggle_move()
    } else {
        state
            .moves
            .get(&player_move_template_id)
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
            .clone()
    };
    
    let enemy_move = if use_struggle_enemy {
        create_struggle_move()
    } else {
        state
            .moves
            .get(&enemy_move_template_id)
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
            .clone()
    };

    // Inicializar battle_stages si no están inicializados (al entrar en batalla)
    if player_mon.battle_stages.is_none() {
        player_mon.init_battle_stages();
    }
    if enemy_mon.battle_stages.is_none() {
        enemy_mon.init_battle_stages();
    }

    // Ejecutar el turno
    eprintln!("[DEBUG] submit_move: Llamando a execute_turn...");
    // Pasar los MoveData para que execute_turn pueda construir los candidatos correctamente
    let turn_result = execute_turn(
        &mut player_mon,
        &mut enemy_mon,
        &player_move,
        &player_move_template_id,
        &enemy_move,
        &enemy_move_template_id,
        &session.team,
        player_active_index,
        &mut battle_state,
        &mut rng,
        Some(&state.moves), // Pasar los MoveData para que execute_turn pueda usarlos
    );

    eprintln!("[DEBUG] submit_move: Turno ejecutado, resultado: {:?}", turn_result.outcome);
    eprintln!("[DEBUG] submit_move: HP después del turno - Player: {}, Enemy: {}", player_mon.current_hp, enemy_mon.current_hp);
    
    // Actualizar los Pokémon en la sesión
    session.team.active_members[player_active_index] = player_mon.clone();
    
    // IMPORTANTE: Solo actualizar el oponente si NO cambió de Pokémon
    // Si cambió (EnemySwitched), el battle_state ya tiene el nuevo Pokémon activo
    if turn_result.outcome != core::battle::BattleOutcome::EnemySwitched {
        *battle_state.get_opponent_active_mut() = enemy_mon.clone();
        eprintln!("[DEBUG] submit_move: Oponente actualizado en battle_state");
    } else {
        eprintln!("[DEBUG] submit_move: Oponente cambió, no actualizando enemy_mon (battle_state ya tiene el nuevo)");
    }
    // Si es EnemySwitched, el battle_state ya tiene el nuevo oponente activo
    // y enemy_mon sigue siendo el Pokémon anterior (debilitado), así que NO lo actualizamos
    
    battle_state.turn_counter += 1;
    eprintln!("[DEBUG] submit_move: Contador de turnos: {}", battle_state.turn_counter);
    
    // Limpiar las acciones pendientes después de ejecutar el turno
    battle_state.pending_player_actions.clear();
    eprintln!("[DEBUG] submit_move: Acciones pendientes limpiadas");

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
                turn_executed: true,
                pending_actions: 0,
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

            // Hook: Activar habilidades de entrada del nuevo Pokémon enemigo
            let mut entry_logs = Vec::new();
            let (weather_to_set, terrain_to_set) = {
                let next_opponent_mut = battle_state.get_opponent_active_mut();
                trigger_on_entry_abilities(&next_opponent, next_opponent_mut, &mut entry_logs)
            };
            // Establecer el weather si la habilidad lo activó
            if let Some(weather) = weather_to_set {
                battle_state.weather = Some(weather);
            }
            // Establecer el terrain si la habilidad lo activó
            if let Some(terrain) = terrain_to_set {
                battle_state.terrain = Some(terrain);
            }
            // Añadir logs de habilidades de entrada al battle_state
            for log in &entry_logs {
                battle_state.add_log(log.clone());
            }
            
            // Obtener el oponente actualizado para el response
            let next_opponent = battle_state.get_opponent_active().clone();
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
                turn_executed: true,
                pending_actions: 0,
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
                turn_executed: true,
                pending_actions: 0,
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
                            
                            // Obtener max_pp del movimiento
                            let max_pp = state.moves.get(random_move)
                                .map(|md| md.pp)
                                .unwrap_or(0);
                            pokemon.learn_new_move(random_move.clone(), max_pp);
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
                                // Obtener max_pp del movimiento
                                let max_pp = state.moves.get(random_move)
                                    .map(|md| md.pp)
                                    .unwrap_or(0);
                                defeated_pokemon.learn_new_move(random_move.clone(), max_pp);
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
            } else if is_gym_victory {
                // Si es victoria de Gym, generar loot y cambiar a LootSelection
                let available_items = vec![
                    "leftovers",
                    "life-orb",
                    "choice-band",
                    "choice-specs",
                    "choice-scarf",
                    "sitrus-berry",
                    "rocky-helmet",
                    "focus-sash",
                ];
                
                let mut rng = StdRng::from_entropy();
                let mut loot = Vec::new();
                
                // Seleccionar 3 objetos aleatorios únicos
                let mut available_items_copy: Vec<String> = available_items.iter().map(|s| s.to_string()).collect();
                for _ in 0..3.min(available_items_copy.len()) {
                    if let Some(item) = available_items_copy.choose(&mut rng) {
                        let item_clone = item.clone();
                        loot.push(item_clone.clone());
                        // Remover el item seleccionado para evitar duplicados
                        available_items_copy.retain(|x| x != &item_clone);
                    }
                }
                
                session.loot_options = Some(loot);
                session.state = GameState::LootSelection;
                battle_state.add_log("¡Has recibido objetos de recompensa!".to_string());
            } else {
                // Volver al mapa para encuentros normales
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
                turn_executed: true,
                pending_actions: 0,
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
                turn_executed: true,
                pending_actions: 0,
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
    eprintln!("[DEBUG] switch_pokemon: Recibida solicitud - session_id: {}, switch_to_index: {}", 
        payload.session_id, payload.switch_to_index);
    
    // Buscar la sesión
    let mut session = state
        .sessions
        .get(&payload.session_id)
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();
    
    eprintln!("[DEBUG] switch_pokemon: Sesión encontrada, estado: {:?}", session.state);

    // Validar que esté en estado de batalla (normal o gimnasio)
    if session.state != GameState::Battle && session.state != GameState::GymBattle {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Obtener el estado de batalla (clonar para poder modificarlo)
    let mut battle_state = session
        .battle
        .clone()
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Limpiar acciones pendientes cuando el jugador cambia de Pokémon
    // (el cambio de Pokémon cancela las acciones del turno)
    battle_state.pending_player_actions.clear();

    // Validar que el índice sea válido
    if payload.switch_to_index >= session.team.active_members.len() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Obtener el primer índice activo (compatibilidad con Single)
    let current_player_index = battle_state.player_active_indices.first().copied().ok_or(StatusCode::BAD_REQUEST)?;
    eprintln!("[DEBUG] switch_pokemon: Índice actual: {}, Índice objetivo: {}", current_player_index, payload.switch_to_index);
    
    // Validar que no sea el mismo Pokémon que ya está activo
    if payload.switch_to_index == current_player_index {
        eprintln!("[DEBUG] switch_pokemon: ERROR - Intento de cambiar al mismo Pokémon");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validar que el Pokémon al que se quiere cambiar no esté debilitado
    let target_pokemon = &session.team.active_members[payload.switch_to_index];
    eprintln!("[DEBUG] switch_pokemon: Pokémon objetivo: {} (HP: {})", target_pokemon.species.display_name, target_pokemon.current_hp);
    if target_pokemon.current_hp == 0 {
        eprintln!("[DEBUG] switch_pokemon: ERROR - Pokémon objetivo está debilitado");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Obtener el Pokémon actual ANTES del cambio para determinar si es forzado
    let current_active = &session.team.active_members[current_player_index];
    let is_forced_switch = current_active.current_hp == 0;
    eprintln!("[DEBUG] switch_pokemon: Cambio forzado: {} (HP actual: {})", is_forced_switch, current_active.current_hp);

    // Obtener el nombre del Pokémon actual y del nuevo
    let current_pokemon_name = current_active.species.display_name.clone();
    let new_pokemon_name = target_pokemon.species.display_name.clone();

    // Cambiar el Pokémon activo (actualizar el primer slot en Single)
    if let Some(first) = battle_state.player_active_indices.first_mut() {
        *first = payload.switch_to_index;
    } else {
        battle_state.player_active_indices.push(payload.switch_to_index);
    }

    // Obtener el nuevo Pokémon activo (mutable)
    let new_active_index = payload.switch_to_index;
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
        let mut rng = StdRng::from_entropy();
        
        // Inicializar PP de todos los movimientos del enemigo
        let enemy_learned_moves = enemy_mon.get_active_learned_moves();
        let enemy_move_ids: Vec<String> = enemy_learned_moves
            .iter()
            .map(|m| m.move_id.clone())
            .collect();
        
        for move_id in &enemy_move_ids {
            if let Some(move_data) = state.moves.get(move_id) {
                initialize_move_pp(&mut enemy_mon, move_id, move_data);
            }
        }
        
        // Verificar si el enemigo tiene movimientos con PP disponible
        let enemy_has_pp = has_moves_with_pp(&enemy_mon);
        
        // Determinar el movimiento del enemigo
        let (enemy_move, enemy_move_template_id) = if !enemy_has_pp {
            // Sin PP disponible: usar Struggle
            (create_struggle_move(), "struggle".to_string())
        } else {
            // Filtrar movimientos con PP disponible
            let enemy_learned_moves_after = enemy_mon.get_active_learned_moves();
            let moves_with_pp: Vec<String> = enemy_learned_moves_after
                .iter()
                .filter(|m| m.current_pp > 0)
                .map(|m| m.move_id.clone())
                .collect();
            
            if moves_with_pp.is_empty() {
                // No hay movimientos con PP, usar Struggle
                (create_struggle_move(), "struggle".to_string())
            } else {
                // Seleccionar un movimiento aleatorio de los que tienen PP
                let move_id = moves_with_pp.choose(&mut rng)
                    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
                let move_data = state
                    .moves
                    .get(move_id)
                    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
                    .clone();
                (move_data, move_id.clone())
            }
        };

        // Inicializar battle_stages si no están inicializados
        if new_active_pokemon.battle_stages.is_none() {
            new_active_pokemon.init_battle_stages();
        }
        if enemy_mon.battle_stages.is_none() {
            enemy_mon.init_battle_stages();
        }

        // Hook: Activar habilidades de entrada del nuevo Pokémon
        let mut entry_logs = Vec::new();
        let (weather_to_set, terrain_to_set) = trigger_on_entry_abilities(&new_active_pokemon, &mut enemy_mon, &mut entry_logs);
        // Establecer el weather si la habilidad lo activó
        if let Some(weather) = weather_to_set {
            battle_state.weather = Some(weather);
        }
        // Establecer el terrain si la habilidad lo activó
        if let Some(terrain) = terrain_to_set {
            battle_state.terrain = Some(terrain);
        }
        // Añadir logs de habilidades de entrada al battle_state
        for log in &entry_logs {
            battle_state.add_log(log.clone());
        }

        // Ejecutar el ataque del enemigo
        let weather_ref = battle_state.weather.as_ref();
        let terrain_ref = battle_state.terrain.as_ref();
        execute_enemy_attack(
            &mut new_active_pokemon,
            &mut enemy_mon,
            &enemy_move,
            &enemy_move_template_id,
            &mut rng,
            weather_ref,
            terrain_ref,
        )
    };

    // Actualizar los Pokémon en la sesión
    session.team.active_members[new_active_index] = new_active_pokemon.clone();
    
    // Actualizar el oponente en el battle_state (solo si hubo ataque)
    if !is_forced_switch {
        if battle_state.is_trainer_battle {
            // Actualizar el primer slot del oponente (en Single solo hay uno)
            if let Some(first) = battle_state.opponent_active_indices.first() {
                battle_state.opponent_team[*first] = enemy_mon.clone();
            }
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

