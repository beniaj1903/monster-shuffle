use rand::rngs::StdRng;
use rand::Rng;
use std::collections::HashMap;
use crate::models::{MoveData, PokemonInstance, FieldPosition, WeatherState, TerrainState, WeatherType, TerrainType};
use crate::game::{BattleState, PlayerTeam};
use super::context::BattleContext;
use super::targeting::resolve_targets;
use super::mechanics::get_effective_speed;
use super::effects::{trigger_on_entry_abilities, is_grounded, apply_weather_residuals, apply_residual_effects};
use super::checks::{consume_move_pp, create_struggle_move};
use super::{BattleOutcome, TurnResult};

/// Candidato de acción para la cola de prioridad global
/// Representa un Pokémon que va a ejecutar una acción en este turno
struct ActionCandidate {
    /// Posición del Pokémon en el campo
    position: FieldPosition,
    /// Si es del jugador (true) o del oponente (false)
    is_player: bool,
    /// Velocidad efectiva del Pokémon (considerando stages y parálisis)
    speed: u16,
    /// Prioridad del movimiento seleccionado
    priority: i8,
    /// Datos del movimiento seleccionado
    move_data: MoveData,
    /// ID del template del movimiento (para consumo de PP)
    move_template_id: String,
    /// Objetivo seleccionado (si aplica, para movimientos "selected-pokemon")
    selected_target: Option<FieldPosition>,
    /// Nombre del Pokémon para logs
    pokemon_name: String,
}

/// Determina el resultado cuando el jugador se debilita
/// Verifica si hay más Pokémon del jugador disponibles
fn determine_player_outcome(player_team: &PlayerTeam, player_active_index: usize) -> BattleOutcome {
    // Buscar si hay otro Pokémon del jugador disponible
    let has_more_pokemon = player_team.active_members.iter()
        .enumerate()
        .any(|(i, p)| i != player_active_index && p.current_hp > 0);
    
    if has_more_pokemon {
        BattleOutcome::PlayerMustSwitch
    } else {
        BattleOutcome::PlayerLost
    }
}

/// Determina el resultado cuando el enemigo se debilita
/// Verifica si hay más enemigos disponibles en el equipo
/// Si hay más enemigos, cambia al siguiente y retorna EnemySwitched
/// Si no hay más, retorna PlayerWon
fn determine_enemy_outcome(battle_state: &mut BattleState) -> BattleOutcome {
    if battle_state.is_trainer_battle {
        // Buscar si hay otro Pokémon enemigo disponible
        if battle_state.switch_to_next_opponent() {
            // El enemigo cambió de Pokémon - el log se añadirá en el handler
            BattleOutcome::EnemySwitched
        } else {
            // No hay más enemigos - jugador ganó
            BattleOutcome::PlayerWon
        }
    } else {
        // Batalla salvaje - solo un enemigo, jugador ganó
        BattleOutcome::PlayerWon
    }
}

/// Ejecuta un turno completo de batalla usando una cola de prioridad global
/// 
/// Soporta N combatientes (2 en Single, 4 en Double).
/// 
/// Flujo:
/// 1. Recopila todos los combatientes en el campo
/// 2. Ordena por prioridad -> velocidad -> RNG
/// 3. Ejecuta acciones en orden, verificando si están vivos antes de cada acción
/// 4. Aplica spread damage penalty (0.75x) para movimientos de área
/// 5. Aplica efectos residuales al final
/// 
/// Retorna un TurnResult con los logs y el resultado de la batalla.
/// 
/// Nota: `player_move` y `enemy_move` se usan como fallback cuando no hay acciones pendientes.
/// Si `battle_state.pending_player_actions` tiene acciones, se usan esas en su lugar.
pub fn execute_turn(
    player_mon: &mut PokemonInstance,
    enemy_mon: &mut PokemonInstance,
    player_move: &MoveData,
    player_move_template_id: &str,
    enemy_move: &MoveData,
    enemy_move_template_id: &str,
    player_team: &PlayerTeam,
    player_active_index: usize,
    battle_state: &mut BattleState,
    rng: &mut StdRng,
    player_moves_data: Option<&HashMap<String, MoveData>>,
) -> TurnResult {
    eprintln!("[DEBUG] execute_turn: Iniciando turno #{}", battle_state.turn_counter);
    eprintln!("[DEBUG] execute_turn: Player: {} (HP: {})", player_mon.species.display_name, player_mon.current_hp);
    eprintln!("[DEBUG] execute_turn: Enemy: {} (HP: {})", enemy_mon.species.display_name, enemy_mon.current_hp);
    eprintln!("[DEBUG] execute_turn: Pending actions: {}", battle_state.pending_player_actions.len());
    
    let mut result = TurnResult::new();

    // Inicializar estados volátiles si no existen
    let is_first_turn = player_mon.volatile_status.is_none() || enemy_mon.volatile_status.is_none();
    
    if player_mon.volatile_status.is_none() {
        player_mon.init_battle_stages();
    }
    if enemy_mon.volatile_status.is_none() {
        enemy_mon.init_battle_stages();
    }

    // Hook: Habilidades que se activan al entrar en batalla (solo en el primer turno)
    if is_first_turn {
        // Activar habilidad del jugador contra el enemigo
        let (weather, terrain) = trigger_on_entry_abilities(player_mon, enemy_mon, &mut result.logs);
        if let Some(w) = weather {
            battle_state.weather = Some(w);
        }
        if let Some(t) = terrain {
            battle_state.terrain = Some(t);
        }
        // Activar habilidad del enemigo contra el jugador (sobrescribe si también tiene clima/terreno)
        let (weather, terrain) = trigger_on_entry_abilities(enemy_mon, player_mon, &mut result.logs);
        if let Some(w) = weather {
            battle_state.weather = Some(w);
        }
        if let Some(t) = terrain {
            battle_state.terrain = Some(t);
        }
    }

    // Resetear estados volátiles al inicio del turno (flinch se resetea)
    player_mon.reset_turn_volatiles();
    enemy_mon.reset_turn_volatiles();

    // PASO 1: Recopilar todos los combatientes en el campo y crear la cola de prioridad
    let player_name = player_mon.species.display_name.clone();
    let enemy_name = enemy_mon.species.display_name.clone();
    let player_effective_speed = get_effective_speed(player_mon) as u16;
    let enemy_effective_speed = get_effective_speed(enemy_mon) as u16;
    
    eprintln!("[DEBUG] execute_turn: Velocidades efectivas - Player: {}, Enemy: {}", player_effective_speed, enemy_effective_speed);
    
    // Crear lista de candidatos
    // Leer las acciones pendientes del jugador desde battle_state
    let mut candidates = Vec::new();
    
    // Agregar candidatos del jugador basándose en pending_player_actions
    eprintln!("[DEBUG] execute_turn: Procesando {} acciones pendientes del jugador", battle_state.pending_player_actions.len());
    for (idx, action) in battle_state.pending_player_actions.iter().enumerate() {
        if idx < battle_state.player_active_indices.len() {
            let player_index = battle_state.player_active_indices[action.user_index];
            if player_index < player_team.active_members.len() {
                let pokemon = &player_team.active_members[player_index];
                let position = if idx == 0 {
                    FieldPosition::PlayerLeft
                } else {
                    FieldPosition::PlayerRight
                };
                
                // Obtener el MoveData para el movimiento
                let move_data = if action.move_id == "struggle" {
                    eprintln!("[DEBUG] execute_turn: Jugador {} usa Struggle", pokemon.species.display_name);
                    create_struggle_move()
                } else if let Some(moves_data) = player_moves_data {
                    // Si tenemos acceso a los MoveData, usarlos
                    moves_data.get(&action.move_id)
                        .cloned()
                        .unwrap_or_else(|| {
                            eprintln!("[DEBUG] execute_turn: WARNING - Movimiento {} no encontrado en moves_data, usando fallback", action.move_id);
                            player_move.clone()
                        })
                } else {
                    // Fallback: usar player_move
                    eprintln!("[DEBUG] execute_turn: WARNING - No hay moves_data disponible, usando fallback");
                    player_move.clone()
                };
                
                let speed = get_effective_speed(pokemon) as u16;
                eprintln!("[DEBUG] execute_turn: Candidato jugador - {}: {} (velocidad: {}, prioridad: {})", 
                    pokemon.species.display_name, action.move_id, speed, move_data.priority);
                candidates.push(ActionCandidate {
                    position,
                    is_player: true,
                    speed,
                    priority: move_data.priority,
                    move_data,
                    move_template_id: action.move_id.clone(),
                    selected_target: action.target_position,
                    pokemon_name: pokemon.species.display_name.clone(),
                });
            }
        }
    }
    
    // Si no hay acciones pendientes (compatibilidad con código antiguo), usar la primera acción
    if candidates.is_empty() {
        candidates.push(ActionCandidate {
            position: FieldPosition::PlayerLeft,
            is_player: true,
            speed: player_effective_speed,
            priority: player_move.priority,
            move_data: player_move.clone(),
            move_template_id: player_move_template_id.to_string(),
            selected_target: None,
            pokemon_name: player_name.clone(),
        });
    }
    
    // Agregar candidatos del enemigo
    // Por ahora, solo agregamos el primer oponente activo
    eprintln!("[DEBUG] execute_turn: Candidato enemigo - {}: {} (velocidad: {}, prioridad: {})", 
        enemy_name, enemy_move_template_id, enemy_effective_speed, enemy_move.priority);
    candidates.push(ActionCandidate {
        position: FieldPosition::OpponentLeft,
        is_player: false,
        speed: enemy_effective_speed,
        priority: enemy_move.priority,
        move_data: enemy_move.clone(),
        move_template_id: enemy_move_template_id.to_string(),
        selected_target: None,
        pokemon_name: enemy_name.clone(),
    });
    
    eprintln!("[DEBUG] execute_turn: Total candidatos recopilados: {}", candidates.len());
    
    // PASO 2: Ordenar la cola de prioridad
    // Orden: Priority DESC -> Speed DESC -> Speed Tie RNG
    eprintln!("[DEBUG] execute_turn: Ordenando candidatos por prioridad y velocidad...");
    candidates.sort_by(|a, b| {
        // Primero por prioridad (mayor primero)
        match b.priority.cmp(&a.priority) {
            std::cmp::Ordering::Equal => {
                // Luego por velocidad (mayor primero)
                match b.speed.cmp(&a.speed) {
                    std::cmp::Ordering::Equal => {
                        // Speed tie: decidir al azar (usar índice como seed)
                        // En un empate real, ambos tienen la misma probabilidad
                        std::cmp::Ordering::Equal
                    }
                    other => other,
                }
            }
            other => other,
        }
    });
    
    // Si hay empate de velocidad y prioridad, usar RNG para desempatar
    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            if candidates[i].priority == candidates[j].priority 
                && candidates[i].speed == candidates[j].speed {
                // Speed tie: decidir al azar
                if rng.gen_bool(0.5) {
                    candidates.swap(i, j);
                }
            }
        }
    }

    // PASO 3: Ejecutar acciones en orden de la cola de prioridad
    eprintln!("[DEBUG] execute_turn: Orden final de ejecución:");
    for (i, c) in candidates.iter().enumerate() {
        eprintln!("[DEBUG] execute_turn:   {}. {} ({}) - Prioridad: {}, Velocidad: {}", 
            i + 1, c.pokemon_name, if c.is_player { "Jugador" } else { "Enemigo" }, c.priority, c.speed);
    }
    
    // Iterar sobre los candidatos ordenados
    for (candidate_idx, candidate) in candidates.iter().enumerate() {
        eprintln!("[DEBUG] execute_turn: === Ejecutando acción {} de {} ===", candidate_idx + 1, candidates.len());
        eprintln!("[DEBUG] execute_turn: Combatiente: {} ({})", candidate.pokemon_name, if candidate.is_player { "Jugador" } else { "Enemigo" });
        eprintln!("[DEBUG] execute_turn: Movimiento: {} (prioridad: {})", candidate.move_template_id, candidate.priority);
        
        // Verificar si el combatiente sigue vivo (pudo morir por un ataque anterior)
        let is_alive = if candidate.is_player {
            player_mon.current_hp > 0
        } else {
            enemy_mon.current_hp > 0
        };
        
        if !is_alive {
            eprintln!("[DEBUG] execute_turn: Combatiente {} está debilitado, saltando...", candidate.pokemon_name);
            continue; // Saltar si el combatiente está debilitado
        }
        
        // Detectar movimientos de clima y terreno antes de ejecutar
        let move_id = &candidate.move_data.id;
        if *move_id == "sunny-day" {
            battle_state.weather = Some(WeatherState::new(WeatherType::Sun));
        } else if *move_id == "rain-dance" {
            battle_state.weather = Some(WeatherState::new(WeatherType::Rain));
        } else if *move_id == "sandstorm" {
            battle_state.weather = Some(WeatherState::new(WeatherType::Sandstorm));
        } else if *move_id == "hail" {
            battle_state.weather = Some(WeatherState::new(WeatherType::Hail));
        }
        
        if *move_id == "electric-terrain" {
            battle_state.terrain = Some(TerrainState::new(TerrainType::Electric));
        } else if *move_id == "grassy-terrain" {
            battle_state.terrain = Some(TerrainState::new(TerrainType::Grassy));
        } else if *move_id == "misty-terrain" {
            battle_state.terrain = Some(TerrainState::new(TerrainType::Misty));
        } else if *move_id == "psychic-terrain" {
            battle_state.terrain = Some(TerrainState::new(TerrainType::Psychic));
        }
        
        // Resolver targets usando resolve_targets
        eprintln!("[DEBUG] execute_turn: Resolviendo objetivos para movimiento {} (target_type: {})", 
            candidate.move_template_id, candidate.move_data.target);
        let targets = resolve_targets(
            candidate.position,
            &candidate.move_data.target,
            candidate.selected_target,
            battle_state,
            player_team,
            rng,
        );
        
        eprintln!("[DEBUG] execute_turn: Objetivos resueltos: {} objetivo(s)", targets.len());
        for (i, target) in targets.iter().enumerate() {
            eprintln!("[DEBUG] execute_turn:   Objetivo {}: {:?}", i + 1, target);
        }
        
        if targets.is_empty() {
            eprintln!("[DEBUG] execute_turn: No hay objetivos válidos, saltando acción");
            // No hay objetivos válidos, continuar
            continue;
        }
        
        // Determinar si es un movimiento de área (múltiples objetivos)
        let is_spread_move = targets.len() > 1;
        let spread_multiplier = if is_spread_move { 0.75 } else { 1.0 };
        if is_spread_move {
            eprintln!("[DEBUG] execute_turn: Movimiento de área detectado, aplicando multiplicador de spread: {:.2}x", spread_multiplier);
        }
        
        // Ejecutar el ataque contra cada objetivo
        for (target_idx, target_pos) in targets.iter().enumerate() {
            eprintln!("[DEBUG] execute_turn: --- Atacando objetivo {} de {}: {:?} ---", target_idx + 1, targets.len(), target_pos);
            // Determinar quién es el defensor basándose en la posición del objetivo
            // En formato Single, solo hay PlayerLeft y OpponentLeft
            let is_target_player = *target_pos == FieldPosition::PlayerLeft || *target_pos == FieldPosition::PlayerRight;
            let is_target_opponent = *target_pos == FieldPosition::OpponentLeft || *target_pos == FieldPosition::OpponentRight;
            
            // Verificar que el objetivo sea válido (no puede atacarse a sí mismo en Single)
            if candidate.is_player && is_target_player {
                continue; // No puede atacarse a sí mismo
            }
            if !candidate.is_player && is_target_opponent {
                continue; // No puede atacarse a sí mismo
            }
            
            // Verificar si el defensor sigue vivo
            let defender_hp = if is_target_opponent {
                enemy_mon.current_hp
            } else {
                player_mon.current_hp
            };
            
            if defender_hp == 0 {
                continue;
            }
            
            // Ejecutar el ataque
            let attacker_name = if candidate.is_player {
                candidate.pokemon_name.clone()
            } else {
                format!("El enemigo {}", candidate.pokemon_name)
            };
            
            let defender_name = if is_target_opponent {
                enemy_mon.species.display_name.clone()
            } else {
                player_mon.species.display_name.clone()
            };
            
            // Crear el contexto de batalla con las referencias correctas
            let mut ctx = if candidate.is_player {
                BattleContext::new(
                    player_mon,
                    enemy_mon,
                    &candidate.move_data,
                    attacker_name.clone(),
                    defender_name.clone(),
                    rng,
                    battle_state.weather.as_ref(),
                    battle_state.terrain.as_ref(),
                )
            } else {
                BattleContext::new(
                    enemy_mon,
                    player_mon,
                    &candidate.move_data,
                    attacker_name.clone(),
                    defender_name.clone(),
                    rng,
                    battle_state.weather.as_ref(),
                    battle_state.terrain.as_ref(),
                )
            };
            
            // Check de estado
            eprintln!("[DEBUG] execute_turn: Verificando si {} puede ejecutar el movimiento...", attacker_name);
            if !ctx.can_execute_move() {
                eprintln!("[DEBUG] execute_turn: {} NO puede ejecutar el movimiento (estado bloqueado)", attacker_name);
                result.logs.extend(ctx.logs);
                continue;
            }
            eprintln!("[DEBUG] execute_turn: {} puede ejecutar el movimiento", attacker_name);
            
            // Calcular daño (aplicar spread multiplier si es necesario)
            let is_charging = if let Some(ref volatile) = ctx.attacker.volatile_status {
                if let (Some(min_turns), Some(max_turns)) = (candidate.move_data.meta.min_turns, candidate.move_data.meta.max_turns) {
                    (min_turns > 1 || max_turns > 1) && volatile.charging_move.is_none()
                } else {
                    false
                }
            } else {
                false
            };
            
            let base_damage = if is_charging {
                eprintln!("[DEBUG] execute_turn: Movimiento está cargando, daño base: 0");
                0
            } else {
                let damage = ctx.calculate_damage();
                eprintln!("[DEBUG] execute_turn: Daño base calculado: {}", damage);
                damage
            };
            
            // Aplicar spread damage penalty
            let total_damage = if is_spread_move && base_damage > 0 {
                let spread_damage = (base_damage as f32 * spread_multiplier) as u16;
                eprintln!("[DEBUG] execute_turn: Aplicando spread penalty: {} * {:.2} = {}", base_damage, spread_multiplier, spread_damage);
                spread_damage
            } else {
                base_damage
            };
            eprintln!("[DEBUG] execute_turn: Daño total a aplicar: {}", total_damage);
            
            // Consumir PP (solo una vez por movimiento, no por objetivo)
            if candidate.move_data.id != "struggle" && target_pos == targets.first().unwrap() {
                eprintln!("[DEBUG] execute_turn: Consumiendo PP del movimiento {}", candidate.move_template_id);
                consume_move_pp(ctx.attacker, &candidate.move_template_id);
            }
            
            if total_damage == 0 && !is_charging {
                result.logs.extend(ctx.logs);
                continue;
            }
            
            if is_charging {
                ctx.apply_move_effects(0);
                result.logs.extend(ctx.logs);
                continue;
            }
            
            // Aplicar daño
            eprintln!("[DEBUG] execute_turn: HP del defensor antes del ataque: {}", ctx.defender.current_hp);
            if total_damage >= ctx.defender.current_hp {
                eprintln!("[DEBUG] execute_turn: ¡{} se debilitó! (daño: {} >= HP: {})", ctx.defender_name, total_damage, ctx.defender.current_hp);
                ctx.defender.current_hp = 0;
                ctx.logs.push(format!("{} se debilitó", ctx.defender_name));
                ctx.apply_move_effects(total_damage);
                result.logs.extend(ctx.logs);
                
                // Verificar resultado de batalla
                if candidate.is_player {
                    eprintln!("[DEBUG] execute_turn: Jugador derrotó al enemigo, determinando resultado...");
                    // Jugador derrotó al enemigo
                    *battle_state.get_opponent_active_mut() = enemy_mon.clone();
                    result.outcome = determine_enemy_outcome(battle_state);
                    eprintln!("[DEBUG] execute_turn: Resultado: {:?}", result.outcome);
                    return result;
                } else {
                    eprintln!("[DEBUG] execute_turn: Enemigo derrotó al jugador, determinando resultado...");
                    // Enemigo derrotó al jugador
                    result.outcome = determine_player_outcome(player_team, player_active_index);
                    eprintln!("[DEBUG] execute_turn: Resultado: {:?}", result.outcome);
                    return result;
                }
            } else {
                ctx.defender.current_hp -= total_damage;
                eprintln!("[DEBUG] execute_turn: HP del defensor después del ataque: {} (daño: {})", ctx.defender.current_hp, total_damage);
            }
            
            // Aplicar efectos secundarios
            eprintln!("[DEBUG] execute_turn: Aplicando efectos secundarios del movimiento...");
            ctx.apply_move_effects(total_damage);
            result.logs.extend(ctx.logs);
        }
    }
    
    // BLOQUE FINAL: Limpieza de estados volátiles y efectos residuales post-turno
    // Solo si la batalla no ha terminado después de que todos los combatientes intentaran actuar
    
    // Limpieza de protect_counter: Si un Pokémon NO usó protección este turno, resetear el contador
    if let Some(ref mut volatile) = player_mon.volatile_status {
        if !volatile.protected {
            volatile.protect_counter = 0;
        }
    }
    if let Some(ref mut volatile) = enemy_mon.volatile_status {
        if !volatile.protected {
            volatile.protect_counter = 0;
        }
    }
    
    // Aplicar efectos residuales al jugador (status conditions)
    let (_, player_residual_logs) = apply_residual_effects(player_mon);
    result.logs.extend(player_residual_logs);
    
    // Aplicar efectos residuales del clima al jugador
    if let Some(ref weather_state) = battle_state.weather {
        let (weather_damage, weather_logs) = apply_weather_residuals(player_mon, Some(weather_state));
        result.logs.extend(weather_logs);
        if weather_damage > 0 && player_mon.current_hp == 0 {
            result.logs.push(format!(
                "{} se debilitó",
                player_mon.species.display_name
            ));
            result.outcome = determine_player_outcome(player_team, player_active_index);
            return result;
        }
    }
    
    // Verificar si el jugador murió por daño residual
    if player_mon.current_hp == 0 {
        result.logs.push(format!(
            "{} se debilitó",
            player_mon.species.display_name
        ));
        result.outcome = determine_player_outcome(player_team, player_active_index);
        return result;
    }

    // Aplicar efectos residuales al enemigo (status conditions)
    let (_, enemy_residual_logs) = apply_residual_effects(enemy_mon);
    result.logs.extend(enemy_residual_logs);
    
    // Aplicar efectos residuales del clima al enemigo
    if let Some(ref weather_state) = battle_state.weather {
        let (weather_damage, weather_logs) = apply_weather_residuals(enemy_mon, Some(weather_state));
        result.logs.extend(weather_logs);
        if weather_damage > 0 && enemy_mon.current_hp == 0 {
            result.logs.push(format!(
                "{} se debilitó",
                enemy_mon.species.display_name
            ));
            // Actualizar el HP del oponente en el battle_state ANTES de verificar reservas
            *battle_state.get_opponent_active_mut() = enemy_mon.clone();
            result.outcome = determine_enemy_outcome(battle_state);
            return result;
        }
    }
    
    // Verificar si el enemigo murió por daño residual
    if enemy_mon.current_hp == 0 {
        result.logs.push(format!(
            "{} se debilitó",
            enemy_mon.species.display_name
        ));
        // Actualizar el HP del oponente en el battle_state ANTES de verificar reservas
        *battle_state.get_opponent_active_mut() = enemy_mon.clone();
        result.outcome = determine_enemy_outcome(battle_state);
        return result;
    }
    
    // Aplicar curación de Grassy Terrain
    if let Some(ref terrain_state) = battle_state.terrain {
        if terrain_state.terrain_type == TerrainType::Grassy {
            // Curar al jugador si está grounded
            if is_grounded(player_mon) && player_mon.current_hp > 0 && player_mon.current_hp < player_mon.base_computed_stats.hp {
                let max_hp = player_mon.base_computed_stats.hp;
                let healing = max_hp / 16;
                let new_hp = (player_mon.current_hp + healing).min(max_hp);
                let actual_healing = new_hp - player_mon.current_hp;
                player_mon.current_hp = new_hp;
                if actual_healing > 0 {
                    result.logs.push(format!(
                        "¡El Campo de Hierba cura a {}!",
                        player_mon.species.display_name
                    ));
                }
            }
            
            // Curar al enemigo si está grounded
            if is_grounded(&enemy_mon) && enemy_mon.current_hp > 0 && enemy_mon.current_hp < enemy_mon.base_computed_stats.hp {
                let max_hp = enemy_mon.base_computed_stats.hp;
                let healing = max_hp / 16;
                let new_hp = (enemy_mon.current_hp + healing).min(max_hp);
                let actual_healing = new_hp - enemy_mon.current_hp;
                enemy_mon.current_hp = new_hp;
                if actual_healing > 0 {
                    result.logs.push(format!(
                        "¡El Campo de Hierba cura a {}!",
                        enemy_mon.species.display_name
                    ));
                }
            }
        }
    }
    
    // Decrementar turnos del clima y verificar si expira
    if let Some(ref mut weather_state) = battle_state.weather {
        if weather_state.turns_remaining > 0 {
            weather_state.turns_remaining -= 1;
        }
        
        if weather_state.turns_remaining == 0 {
            result.logs.push("¡El clima volvió a la normalidad!".to_string());
            battle_state.weather = None;
        }
    }
    
    // Decrementar turnos del terreno y verificar si expira
    if let Some(ref mut terrain_state) = battle_state.terrain {
        if terrain_state.turns_remaining > 0 {
            terrain_state.turns_remaining -= 1;
        }
        
        if terrain_state.turns_remaining == 0 {
            result.logs.push("¡El terreno volvió a la normalidad!".to_string());
            battle_state.terrain = None;
        }
    }

    // LIMPIEZA: Resetear estados volátiles al final del turno
    // Esto asegura que flinch no persista al siguiente turno
    player_mon.reset_turn_volatiles();
    enemy_mon.reset_turn_volatiles();

    // Si llegamos aquí, ambos Pokémon siguen vivos después de todos los efectos
    result.outcome = BattleOutcome::Continue;
    result
}

/// Ejecuta solo el ataque del enemigo (usado cuando el jugador cambia de Pokémon)
/// 
/// Cuando el jugador cambia de Pokémon, el enemigo ataca automáticamente al nuevo Pokémon entrante.
/// Esta función ejecuta solo el ataque del enemigo sin el ataque del jugador.
pub fn execute_enemy_attack(
    player_mon: &mut PokemonInstance,
    enemy_mon: &mut PokemonInstance,
    enemy_move: &MoveData,
    enemy_move_template_id: &str,
    rng: &mut StdRng,
    weather: Option<&WeatherState>,
    terrain: Option<&TerrainState>,
) -> TurnResult {
    let mut result = TurnResult::new();

    // Usar BattleContext para ejecutar el ataque del enemigo
    let enemy_name = format!("El enemigo {}", enemy_mon.species.display_name);
    let player_name = player_mon.species.display_name.clone();
    
    let mut ctx = BattleContext::new(
        enemy_mon,
        player_mon,
        enemy_move,
        enemy_name,
        player_name,
        rng,
        weather,
        terrain,
    );

    // Paso 1: Chequeo de estado
    if !ctx.can_execute_move() {
        result.logs.extend(ctx.logs);
        result.outcome = BattleOutcome::Continue;
        return result;
    }

    // Paso 2: Cálculo de daño
    let total_damage = ctx.calculate_damage();
    result.enemy_damage_dealt = total_damage;
    
    // Consumir PP solo si el movimiento se ejecutó (aunque no haya golpeado)
    // Struggle no consume PP (es infinito)
    if enemy_move.id != "struggle" {
        consume_move_pp(ctx.attacker, enemy_move_template_id);
    }
    
    if total_damage == 0 {
        // No golpeó, solo añadir logs
        result.logs.extend(ctx.logs);
        result.outcome = BattleOutcome::Continue;
        return result;
    }

    // Aplicar daño acumulado
    if total_damage >= ctx.defender.current_hp {
        ctx.defender.current_hp = 0;
        ctx.logs.push(format!(
            "{} se debilitó",
            ctx.defender.species.display_name
        ));
        // Aplicar efectos secundarios antes de retornar
        ctx.apply_move_effects(total_damage);
        result.logs.extend(ctx.logs);
        // Nota: Para execute_enemy_attack, no tenemos acceso a player_team
        // El handler deberá verificar esto después
        result.outcome = BattleOutcome::PlayerMustSwitch;
        return result;
    } else {
        ctx.defender.current_hp -= total_damage;
    }

    // Paso 3: Efectos secundarios
    ctx.apply_move_effects(total_damage);
    result.logs.extend(ctx.logs);

    // BLOQUE FINAL: Efectos residuales post-turno
    // Aplicar efectos residuales al jugador
    let (_, player_residual_logs) = apply_residual_effects(player_mon);
    result.logs.extend(player_residual_logs);
    
    // Verificar si el jugador murió por daño residual
    if player_mon.current_hp == 0 {
        result.logs.push(format!(
            "{} se debilitó",
            player_mon.species.display_name
        ));
        // Nota: Para execute_enemy_attack, no tenemos acceso a player_team
        // El handler deberá verificar esto después
        result.outcome = BattleOutcome::PlayerMustSwitch;
        return result;
    }

    // Si llegamos aquí, el jugador sigue vivo
    result.outcome = BattleOutcome::Continue;
    result
}

