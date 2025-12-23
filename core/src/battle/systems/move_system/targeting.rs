use rand::rngs::StdRng;
use rand::Rng;
use crate::models::{FieldPosition, MoveData, PokemonInstance};
use crate::game::{BattleState, PlayerTeam};
use super::super::redirection_system::apply_redirection;

/// Resuelve los objetivos de un movimiento basándose en el tipo de objetivo y la posición del usuario
///
/// Esta función determina a quién golpea un movimiento en combates dobles (VGC).
/// Retorna un vector de posiciones en el campo que serán afectadas por el movimiento.
///
/// # Parámetros
/// - `user_pos`: La posición del Pokémon que usa el movimiento
/// - `move_target_type`: El tipo de objetivo del movimiento (ej: "selected-pokemon", "all-opponents")
/// - `selected_target`: El objetivo seleccionado por el usuario (si aplica)
/// - `battle_state`: El estado de batalla para verificar qué posiciones están ocupadas por Pokémon vivos
/// - `player_team`: El equipo del jugador para verificar qué Pokémon están vivos
/// - `opponent_team`: El equipo del oponente para verificar qué Pokémon están vivos (datos actuales durante execute_turn)
/// - `attacker`: El Pokémon que usa el movimiento (para redirection)
/// - `move_data`: Los datos del movimiento (para redirection)
/// - `rng`: Generador de números aleatorios para objetivos aleatorios
///
/// # Retorna
/// Un vector de posiciones en el campo que serán afectadas por el movimiento
pub fn resolve_targets(
    user_pos: FieldPosition,
    move_target_type: &str,
    selected_target: Option<FieldPosition>,
    battle_state: &BattleState,
    player_team: &PlayerTeam,
    opponent_team: &Vec<PokemonInstance>,
    attacker: &PokemonInstance,
    move_data: &MoveData,
    rng: &mut StdRng,
) -> Vec<FieldPosition> {
    eprintln!("[TARGET] resolve_targets: user_pos={:?}, move_target_type={}, selected_target={:?}, format={:?}",
        user_pos, move_target_type, selected_target, battle_state.format);

    match move_target_type {
        // PRIORIDAD ABSOLUTA: El usuario mismo (ej: Swords Dance, Calm Mind)
        // IGNORA completamente selected_target del frontend
        "user" => {
            vec![user_pos]
        }
        
        // PRIORIDAD ABSOLUTA: Un oponente aleatorio (ej: Outrage, Thrash, Petal Dance)
        // IGNORA selected_target, selecciona aleatoriamente
        "random-opponent" => {
            let mut available_opponents = Vec::new();
            if is_position_alive(FieldPosition::OpponentLeft, battle_state, player_team, opponent_team) {
                available_opponents.push(FieldPosition::OpponentLeft);
            }
            if is_position_alive(FieldPosition::OpponentRight, battle_state, player_team, opponent_team) {
                available_opponents.push(FieldPosition::OpponentRight);
            }
            
            if available_opponents.is_empty() {
                vec![]
            } else {
                // Seleccionar un oponente aleatorio
                let index = rng.gen_range(0..available_opponents.len());
                let selected = available_opponents[index];
                vec![selected]
            }
        }
        
        // Todos los oponentes (ej: Rock Slide, Surf)
        // IGNORA selected_target
        "all-opponents" => {
            let mut targets = Vec::new();
            // Agregar posiciones de oponentes que estén vivas
            if is_position_alive(FieldPosition::OpponentLeft, battle_state, player_team, opponent_team) {
                targets.push(FieldPosition::OpponentLeft);
            }
            if is_position_alive(FieldPosition::OpponentRight, battle_state, player_team, opponent_team) {
                targets.push(FieldPosition::OpponentRight);
            }
            targets
        }
        
        // Todos los otros Pokémon (ej: Earthquake, Explosion)
        // IGNORA selected_target
        "all-other-pokemon" => {
            let mut targets = Vec::new();
            // Agregar todos los oponentes
            if is_position_alive(FieldPosition::OpponentLeft, battle_state, player_team, opponent_team) {
                targets.push(FieldPosition::OpponentLeft);
            }
            if is_position_alive(FieldPosition::OpponentRight, battle_state, player_team, opponent_team) {
                targets.push(FieldPosition::OpponentRight);
            }
            // Agregar el aliado del usuario (si existe y está vivo)
            if let Some(ally_pos) = get_ally_position(user_pos) {
                if is_position_alive(ally_pos, battle_state, player_team, opponent_team) {
                    targets.push(ally_pos);
                }
            }
            targets
        }
        
        // Campo del usuario (ej: Light Screen, Reflect)
        // IGNORA selected_target
        "users-field" => {
            let mut targets = Vec::new();
            if is_position_alive(FieldPosition::PlayerLeft, battle_state, player_team, opponent_team) {
                targets.push(FieldPosition::PlayerLeft);
            }
            if is_position_alive(FieldPosition::PlayerRight, battle_state, player_team, opponent_team) {
                targets.push(FieldPosition::PlayerRight);
            }
            targets
        }
        
        // Campo del oponente (ej: Stealth Rock, Spikes)
        // IGNORA selected_target
        "opponents-field" => {
            let mut targets = Vec::new();
            if is_position_alive(FieldPosition::OpponentLeft, battle_state, player_team, opponent_team) {
                targets.push(FieldPosition::OpponentLeft);
            }
            if is_position_alive(FieldPosition::OpponentRight, battle_state, player_team, opponent_team) {
                targets.push(FieldPosition::OpponentRight);
            }
            targets
        }
        
        // El aliado del usuario (ej: Helping Hand, Follow Me)
        // IGNORA selected_target
        "ally" => {
            if let Some(ally_pos) = get_ally_position(user_pos) {
                if is_position_alive(ally_pos, battle_state, player_team, opponent_team) {
                    vec![ally_pos]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        }
        
        // Objetivo seleccionado específicamente (ej: Tackle, Thunderbolt)
        // AQUÍ SÍ usa selected_target, pero con validación robusta
        "selected-pokemon" => {
            if let Some(target) = selected_target {
                eprintln!("[TARGET] selected-pokemon: target seleccionado = {:?}", target);
                // Validar que el objetivo esté vivo
                let is_alive = is_position_alive(target, battle_state, player_team, opponent_team);
                eprintln!("[TARGET] selected-pokemon: target {:?} está vivo = {}", target, is_alive);

                if is_alive {
                    // Aplicar redirección si está activa
                    let final_target = if let Some(redirected) = apply_redirection(
                        target,
                        user_pos,
                        attacker,
                        move_data,
                        battle_state,
                    ) {
                        eprintln!("[TARGET] selected-pokemon: redirección aplicada {:?} -> {:?}", target, redirected);
                        redirected
                    } else {
                        target
                    };
                    eprintln!("[TARGET] selected-pokemon: objetivo final = {:?}", final_target);
                    vec![final_target]
                } else {
                    eprintln!("[TARGET] selected-pokemon: target {:?} NO está vivo, retornando vacío", target);
                    vec![]
                }
            } else {
                eprintln!("[TARGET] selected-pokemon: NO hay target seleccionado, usando default");
                // Si no hay objetivo seleccionado, usar lógica por defecto según el formato
                use crate::models::BattleFormat;
                match battle_state.format {
                    BattleFormat::Single => {
                        // En Single, si el usuario es del jugador, el objetivo es OpponentLeft
                        // Si el usuario es del oponente, el objetivo es PlayerLeft
                        let default_target = if user_pos == FieldPosition::PlayerLeft || user_pos == FieldPosition::PlayerRight {
                            FieldPosition::OpponentLeft
                        } else {
                            FieldPosition::PlayerLeft
                        };
                        // Validar que el objetivo por defecto esté vivo
                        if is_position_alive(default_target, battle_state, player_team, opponent_team) {
                            vec![default_target]
                        } else {
                            vec![]
                        }
                    }
                    BattleFormat::Double => {
                        // En Double, se requiere un objetivo explícito
                        vec![]
                    }
                }
            }
        }
        
        // Por defecto, si no se reconoce el tipo, intentar usar selected_target o retornar vacío
        _ => {
            if let Some(target) = selected_target {
                if is_position_alive(target, battle_state, player_team, opponent_team) {
                    vec![target]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        }
    }
}

/// Obtiene la posición del aliado del usuario
/// En combates dobles, cada posición tiene un aliado
fn get_ally_position(user_pos: FieldPosition) -> Option<FieldPosition> {
    match user_pos {
        FieldPosition::PlayerLeft => Some(FieldPosition::PlayerRight),
        FieldPosition::PlayerRight => Some(FieldPosition::PlayerLeft),
        FieldPosition::OpponentLeft => Some(FieldPosition::OpponentRight),
        FieldPosition::OpponentRight => Some(FieldPosition::OpponentLeft),
    }
}

/// Verifica si un Pokémon en una posición específica está vivo
/// Retorna true si la posición está ocupada por un Pokémon con HP > 0
fn is_position_alive(
    position: FieldPosition,
    battle_state: &BattleState,
    player_team: &PlayerTeam,
    opponent_team: &Vec<PokemonInstance>,
) -> bool {
    use crate::models::BattleFormat;
    match position {
        FieldPosition::PlayerLeft => {
            // En formato Single, solo hay un Pokémon activo
            if battle_state.format == BattleFormat::Single {
                if let Some(&index) = battle_state.player_active_indices.first() {
                    index < player_team.active_members.len() 
                        && player_team.active_members[index].current_hp > 0
                } else {
                    false
                }
            } else {
                // En formato Double, PlayerLeft es el primer índice
                if let Some(&index) = battle_state.player_active_indices.first() {
                    index < player_team.active_members.len() 
                        && player_team.active_members[index].current_hp > 0
                } else {
                    false
                }
            }
        }
        FieldPosition::PlayerRight => {
            // En formato Single, no hay segunda posición
            if battle_state.format == BattleFormat::Single {
                false
            } else {
                // En formato Double, PlayerRight es el segundo índice
                if let Some(&index) = battle_state.player_active_indices.get(1) {
                    index < player_team.active_members.len() 
                        && player_team.active_members[index].current_hp > 0
                } else {
                    false
                }
            }
        }
        FieldPosition::OpponentLeft => {
            if battle_state.is_trainer_battle {
                // En formato Single, solo hay un oponente activo
                if battle_state.format == BattleFormat::Single {
                    if let Some(&index) = battle_state.opponent_active_indices.first() {
                        let hp = opponent_team.get(index).map(|p| p.current_hp).unwrap_or(0);
                        eprintln!("[TARGET] is_position_alive OpponentLeft (trainer/single): index={}, HP={}", index, hp);
                        index < opponent_team.len()
                            && opponent_team[index].current_hp > 0
                    } else {
                        eprintln!("[TARGET] is_position_alive OpponentLeft (trainer/single): NO index");
                        false
                    }
                } else {
                    // En formato Double, OpponentLeft es el primer índice
                    if let Some(&index) = battle_state.opponent_active_indices.first() {
                        let hp = opponent_team.get(index).map(|p| p.current_hp).unwrap_or(0);
                        eprintln!("[TARGET] is_position_alive OpponentLeft (trainer/double): index={}, HP={}", index, hp);
                        index < opponent_team.len()
                            && opponent_team[index].current_hp > 0
                    } else {
                        eprintln!("[TARGET] is_position_alive OpponentLeft (trainer/double): NO index");
                        false
                    }
                }
            } else {
                // Batalla salvaje - solo hay un oponente
                // IMPORTANTE: Usar opponent_team del parámetro, no battle_state.opponent_team
                if let Some(&index) = battle_state.opponent_active_indices.first() {
                    let hp = opponent_team.get(index).map(|p| p.current_hp).unwrap_or(0);
                    eprintln!("[TARGET] is_position_alive OpponentLeft (wild): index={}, HP={}", index, hp);
                    index < opponent_team.len()
                        && opponent_team[index].current_hp > 0
                } else {
                    eprintln!("[TARGET] is_position_alive OpponentLeft (wild): NO index");
                    false
                }
            }
        }
        FieldPosition::OpponentRight => {
            // En formato Single, no hay segunda posición
            if battle_state.format == BattleFormat::Single {
                false
            } else if battle_state.is_trainer_battle {
                // En formato Double, OpponentRight es el segundo índice
                if let Some(&index) = battle_state.opponent_active_indices.get(1) {
                    index < opponent_team.len()
                        && opponent_team[index].current_hp > 0
                } else {
                    false
                }
            } else {
                // Batalla salvaje - no hay segunda posición
                false
            }
        }
    }
}

