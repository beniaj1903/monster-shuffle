use rand::rngs::StdRng;
use rand::Rng;
use std::collections::HashMap;
use crate::models::{MoveData, PokemonInstance, FieldPosition, BattleFormat, WeatherType, TerrainType, WeatherState, TerrainState};
use crate::game::{BattleState, PlayerTeam};
use super::context::BattleContext;
use super::targeting::resolve_targets;
use super::mechanics::get_effective_speed;
use super::effects::{apply_weather_residuals, apply_residual_effects};
use super::checks::consume_move_pp;
use super::ability_logic::{get_ability_hooks, AbilityTrigger, AbilityEffect, StatChangeTarget};
use super::{BattleOutcome, TurnResult};

// Importar desde los nuevos módulos de infraestructura y sistemas
use super::infrastructure::{
    get_pokemon, get_pokemon_mut, get_team_index, is_pokemon_alive, resolve_move_data
};
use super::systems::ai_system::select_ai_move;
use super::systems::validation_system::reset_turn_flags;
use super::systems::action_system::ActionCandidate;

// MIGRADO: ActionCandidate ahora está en systems/action_system/models.rs

/// Determina el resultado cuando el jugador se debilita
/// Verifica si hay más Pokémon del jugador disponibles
fn determine_player_outcome(player_team: &PlayerTeam, battle_state: &BattleState) -> BattleOutcome {
    // Buscar si hay otro Pokémon del jugador disponible
    let has_more_pokemon = player_team.active_members.iter()
        .enumerate()
        .any(|(i, p)| !battle_state.player_active_indices.contains(&i) && p.current_hp > 0);

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
fn determine_enemy_outcome(battle_state: &mut BattleState, logs: &mut Vec<String>) -> BattleOutcome {
    // Buscar si hay otro Pokémon enemigo disponible
    if battle_state.switch_to_next_opponent() {
        // El enemigo cambió de Pokémon
        let next_opponent = battle_state.get_opponent_active().clone();
        let opponent_name = battle_state.opponent_name.clone().unwrap_or_else(|| "El entrenador".to_string());
        logs.push(format!(
            "{} envió a {}!",
            opponent_name,
            next_opponent.species.display_name
        ));

        BattleOutcome::EnemySwitched
    } else {
        // No hay más enemigos - jugador ganó
        BattleOutcome::PlayerWon
    }
}

/// Ejecuta un turno completo de batalla usando una cola de prioridad global
///
/// Soporta N combatientes (2 en Single, 4 en Double).
///
/// Flujo:
/// 1. Hook de entrada (solo turno 1)
/// 2. Resetea estados volátiles de inicio de turno
/// 3. Recopila todos los combatientes en el campo
/// 4. Ordena por prioridad -> velocidad -> RNG
/// 5. Ejecuta acciones en orden, verificando si están vivos antes de cada acción
/// 6. Aplica spread damage penalty (0.75x) para movimientos de área
/// 7. Aplica efectos residuales al final
/// 8. Determina resultado de la batalla
///
/// Retorna un TurnResult con los logs y el resultado de la batalla.
pub fn execute_turn(
    player_team: &mut PlayerTeam,
    opponent_team: &mut Vec<PokemonInstance>,
    battle_state: &mut BattleState,
    rng: &mut StdRng,
    move_pool: Option<&HashMap<String, MoveData>>,
) -> TurnResult {
    let mut result = TurnResult::new();

    // 1. Hook de Entrada (Solo turno 1)
    if battle_state.turn_counter == 1 {
        handle_entry_hazards(battle_state, player_team, opponent_team, &mut result.logs);
    }

    // 2. Resetear estados volátiles de inicio de turno (Flinch, Protect counter)
    reset_turn_flags(player_team, opponent_team, battle_state);

    // 3. Fase de Recolección y Ordenamiento
    let mut candidates = collect_action_candidates(
        battle_state,
        player_team,
        opponent_team,
        move_pool,
        &mut result.logs,
    );
    sort_candidates(&mut candidates, rng);

    // 4. Fase de Ejecución (Bucle Principal)
    for candidate in candidates {
        // Verificar si el usuario sigue vivo antes de ejecutar
        if !is_pokemon_alive(candidate.position, candidate.team_index, battle_state, player_team, opponent_team) {
            continue;
        }

        // Ejecutar la acción
        let action_result = execute_single_action(
            candidate,
            battle_state,
            player_team,
            opponent_team,
            rng,
        );

        // Acumular logs
        result.logs.extend(action_result.logs);

        // Acumular daño
        if action_result.is_player_action {
            result.player_damage_dealt += action_result.damage_dealt;
        } else {
            result.enemy_damage_dealt += action_result.damage_dealt;
        }

        // Si hubo un knockout, verificar el resultado de la batalla
        if action_result.caused_knockout {
            let outcome = check_battle_state(battle_state, player_team, &mut result.logs);
            if outcome != BattleOutcome::Continue {
                result.outcome = outcome;
                return result;
            }
        }
    }

    // 5. Fase de Limpieza (End of Turn)
    process_end_of_turn_residuals(
        battle_state,
        player_team,
        opponent_team,
        &mut result.logs,
    );

    // 6. Determinar Resultado Final
    result.outcome = check_battle_state(battle_state, player_team, &mut result.logs);

    result
}

/// Resultado de ejecutar una acción individual
struct ActionResult {
    logs: Vec<String>,
    damage_dealt: u16,
    caused_knockout: bool,
    is_player_action: bool,
}

impl ActionResult {
    fn new(is_player: bool) -> Self {
        Self {
            logs: Vec::new(),
            damage_dealt: 0,
            caused_knockout: false,
            is_player_action: is_player,
        }
    }
}

/// Ejecuta una acción individual de un Pokémon
fn execute_single_action(
    candidate: ActionCandidate,
    battle_state: &mut BattleState,
    player_team: &mut PlayerTeam,
    opponent_team: &mut Vec<PokemonInstance>,
    rng: &mut StdRng,
) -> ActionResult {
    let mut result = ActionResult::new(candidate.is_player);

    // 1. Resolver Objetivos (Targets)
    let targets = resolve_targets(
        candidate.position,
        &candidate.move_data.target,
        candidate.selected_target,
        battle_state,
        player_team,
        rng,
    );

    if targets.is_empty() {
        result.logs.push(format!(
            "¡{} usó {}, pero no había objetivo!",
            candidate.pokemon_name,
            candidate.move_data.name
        ));
        return result;
    }

    // 2. Calcular Daño de Área (Spread penalty)
    let is_spread = targets.len() > 1;
    let spread_factor = if is_spread { 0.75 } else { 1.0 };

    // 3. Consumir PP (solo una vez, antes de ejecutar)
    let should_consume_pp = candidate.move_template_id != "struggle";

    // 4. Iterar sobre los objetivos y ejecutar el movimiento
    for target_pos in targets {
        // Obtener atacante y defensor mutables
        let hit_result = process_move_hit(
            &candidate,
            target_pos,
            spread_factor,
            battle_state,
            player_team,
            opponent_team,
            rng,
        );

        result.logs.extend(hit_result.logs);
        result.damage_dealt += hit_result.damage;

        if hit_result.defender_fainted {
            result.caused_knockout = true;
        }
    }

    // 5. Consumir PP después de ejecutar exitosamente
    if should_consume_pp {
        let attacker = get_pokemon_mut(
            candidate.position,
            candidate.team_index,
            battle_state,
            player_team,
            opponent_team,
        );
        if let Some(mon) = attacker {
            consume_move_pp(mon, &candidate.move_template_id);
        }
    }

    result
}

/// Resultado de golpear a un objetivo
struct HitResult {
    logs: Vec<String>,
    damage: u16,
    defender_fainted: bool,
}

/// Procesa un golpe individual a un objetivo específico
fn process_move_hit(
    candidate: &ActionCandidate,
    target_pos: FieldPosition,
    spread_factor: f32,
    battle_state: &mut BattleState,
    player_team: &mut PlayerTeam,
    opponent_team: &mut Vec<PokemonInstance>,
    rng: &mut StdRng,
) -> HitResult {
    let mut result = HitResult {
        logs: Vec::new(),
        damage: 0,
        defender_fainted: false,
    };

    // Verificar que el defensor sigue vivo
    let defender_index = get_team_index(target_pos, battle_state);
    if defender_index.is_none() {
        return result;
    }
    let defender_index = defender_index.unwrap();

    // Obtener referencias temporales para verificar vida
    let defender_alive = {
        let defender = get_pokemon(target_pos, defender_index, battle_state, player_team, opponent_team);
        defender.map(|d| d.current_hp > 0).unwrap_or(false)
    };

    if !defender_alive {
        return result;
    }

    // Clonar los Pokémon para pasarlos al BattleContext
    let attacker_index = candidate.team_index;

    let (mut attacker_clone, mut defender_clone, weather_clone, terrain_clone) = {
        let attacker = get_pokemon(candidate.position, attacker_index, battle_state, player_team, opponent_team).unwrap();
        let defender = get_pokemon(target_pos, defender_index, battle_state, player_team, opponent_team).unwrap();
        (
            attacker.clone(),
            defender.clone(),
            battle_state.weather.clone(),
            battle_state.terrain.clone(),
        )
    };

    // Guardar el nombre del defensor antes de prestarlo mutablemente
    let defender_name = defender_clone.species.display_name.clone();

    // Crear el BattleContext para procesar el ataque
    // Usamos las clones del weather y terrain para evitar problemas de borrow
    let mut ctx = BattleContext::new(
        &mut attacker_clone,
        &mut defender_clone,
        &candidate.move_data,
        candidate.pokemon_name.clone(),
        defender_name,
        rng,
        weather_clone.as_ref(),
        terrain_clone.as_ref(),
    );

    // Paso 1: Verificar si puede ejecutar el movimiento
    if !ctx.can_execute_move() {
        result.logs = ctx.logs;
        // Aplicar cambios del atacante (puede haber cambiado volatile_status)
        if let Some(attacker) = get_pokemon_mut(candidate.position, attacker_index, battle_state, player_team, opponent_team) {
            *attacker = attacker_clone;
        }
        return result;
    }

    // Paso 2: Calcular daño
    let mut damage = ctx.calculate_damage();

    // Aplicar spread penalty si es necesario
    if spread_factor < 1.0 {
        damage = (damage as f32 * spread_factor) as u16;
    }

    // Paso 3: Aplicar efectos del movimiento
    ctx.apply_move_effects(damage);

    // Obtener logs del contexto (antes de consumirlo)
    result.logs = ctx.logs;
    result.damage = damage;

    // Paso 4: Aplicar el daño al defensor
    // Extraer los clones del contexto (consumiendo el contexto)
    let final_attacker = ctx.attacker;
    let final_defender = ctx.defender;

    // Verificar si el defensor se debilitó
    if final_defender.current_hp == 0 && damage > 0 {
        result.defender_fainted = true;
    }

    // Aplicar cambios de vuelta a los Pokémon originales
    if let Some(attacker) = get_pokemon_mut(candidate.position, attacker_index, battle_state, player_team, opponent_team) {
        *attacker = final_attacker.clone();
    }
    if let Some(defender) = get_pokemon_mut(target_pos, defender_index, battle_state, player_team, opponent_team) {
        *defender = final_defender.clone();
    }

    result
}

/// Recopila todas las acciones pendientes de jugador y oponente
fn collect_action_candidates(
    state: &BattleState,
    p_team: &PlayerTeam,
    o_team: &Vec<PokemonInstance>,
    move_pool: Option<&HashMap<String, MoveData>>,
    logs: &mut Vec<String>,
) -> Vec<ActionCandidate> {
    let mut candidates = Vec::new();

    // 1. Recopilar acciones del JUGADOR
    for action in &state.pending_player_actions {
        if let Some(&team_idx) = state.player_active_indices.get(action.user_index) {
            if let Some(pokemon) = p_team.active_members.get(team_idx) {
                if pokemon.current_hp > 0 {
                    // Determinar posición
                    let pos = match state.format {
                        BattleFormat::Single => FieldPosition::PlayerLeft,
                        BattleFormat::Double => {
                            if action.user_index == 0 {
                                FieldPosition::PlayerLeft
                            } else {
                                FieldPosition::PlayerRight
                            }
                        }
                    };

                    // Resolver MoveData
                    let move_data = resolve_move_data(&action.move_id, move_pool);
                    let priority = get_priority_with_abilities(pokemon, &move_data);

                    candidates.push(ActionCandidate {
                        position: pos,
                        team_index: team_idx,
                        is_player: true,
                        speed: get_speed_with_abilities(pokemon, state),
                        priority,
                        move_data,
                        move_template_id: action.move_id.clone(),
                        selected_target: action.target_position,
                        pokemon_name: pokemon.species.display_name.clone(),
                    });
                }
            }
        }
    }

    // 2. Recopilar acciones del OPONENTE (IA Simple)
    for (i, &team_idx) in state.opponent_active_indices.iter().enumerate() {
        if let Some(pokemon) = o_team.get(team_idx) {
            if pokemon.current_hp > 0 {
                let pos = match state.format {
                    BattleFormat::Single => FieldPosition::OpponentLeft,
                    BattleFormat::Double => {
                        if i == 0 {
                            FieldPosition::OpponentLeft
                        } else {
                            FieldPosition::OpponentRight
                        }
                    }
                };

                // IA muy básica: Seleccionar movimiento aleatorio con PP o Struggle
                let move_id = select_ai_move(pokemon, logs);
                let move_data = resolve_move_data(&move_id, move_pool);
                let priority = get_priority_with_abilities(pokemon, &move_data);

                candidates.push(ActionCandidate {
                    position: pos,
                    team_index: team_idx,
                    is_player: false,
                    speed: get_speed_with_abilities(pokemon, state),
                    priority,
                    move_data,
                    move_template_id: move_id,
                    selected_target: None, // La IA usa lógica por defecto de targeting
                    pokemon_name: pokemon.species.display_name.clone(),
                });
            }
        }
    }

    candidates
}

// MIGRADO: select_ai_move ahora está en systems/ai_system/selector.rs

/// Ordena los candidatos por prioridad -> velocidad -> RNG
fn sort_candidates(candidates: &mut Vec<ActionCandidate>, rng: &mut StdRng) {
    // Asignar un número aleatorio a cada candidato para desempatar
    let mut with_random: Vec<(ActionCandidate, u32)> = candidates
        .drain(..)
        .map(|c| {
            let random = rng.gen::<u32>();
            (c, random)
        })
        .collect();

    // Ordenar por: prioridad (desc) -> velocidad (desc) -> random (asc)
    with_random.sort_by(|a, b| {
        // Primero por prioridad (mayor primero)
        match b.0.priority.cmp(&a.0.priority) {
            std::cmp::Ordering::Equal => {
                // Luego por velocidad (mayor primero)
                match b.0.speed.cmp(&a.0.speed) {
                    std::cmp::Ordering::Equal => {
                        // Desempate aleatorio
                        a.1.cmp(&b.1)
                    }
                    other => other,
                }
            }
            other => other,
        }
    });

    // Restaurar el vector original
    *candidates = with_random.into_iter().map(|(c, _)| c).collect();
}

/// Calcula la velocidad efectiva de un Pokémon incluyendo modificadores de habilidades
fn get_speed_with_abilities(
    pokemon: &PokemonInstance,
    battle_state: &BattleState,
) -> u16 {
    let base_speed = get_effective_speed(pokemon) as u16;
    let ability_id = &pokemon.ability;
    let hooks = get_ability_hooks(ability_id);

    // Buscar modificadores de velocidad
    for hook in hooks.iter().filter(|h| matches!(h.trigger, AbilityTrigger::ModifySpeed)) {
        match &hook.effect {
            AbilityEffect::MultiplySpeedInWeather { weather, multiplier } => {
                // Verificar si el clima activo coincide
                if let Some(current_weather) = &battle_state.weather {
                    if current_weather.weather_type == *weather {
                        return (base_speed as f32 * multiplier) as u16;
                    }
                }
            },
            AbilityEffect::MultiplySpeedInTerrain { terrain, multiplier } => {
                // Verificar si el terreno activo coincide
                if let Some(current_terrain) = &battle_state.terrain {
                    if current_terrain.terrain_type == *terrain {
                        return (base_speed as f32 * multiplier) as u16;
                    }
                }
            },
            _ => {},
        }
    }

    base_speed
}

/// Calcula la prioridad de un movimiento incluyendo modificadores de habilidades
fn get_priority_with_abilities(
    pokemon: &PokemonInstance,
    move_data: &MoveData,
) -> i8 {
    use super::ability_logic::PriorityCondition;
    use crate::models::StatusCondition;

    let base_priority = move_data.priority;
    let ability_id = &pokemon.ability;
    let hooks = get_ability_hooks(ability_id);

    // Buscar modificadores de prioridad
    for hook in hooks.iter().filter(|h| matches!(h.trigger, AbilityTrigger::ModifyPriority)) {
        if let AbilityEffect::ModifyMovePriority { move_type, priority_boost, condition } = &hook.effect {
            // Verificar si el tipo de movimiento coincide (si se especifica)
            let type_matches = if let Some(_required_type) = move_type {
                // TODO: Comparar con el tipo del movimiento cuando esté disponible
                // Por ahora, asumimos que el tipo coincide si es Flying para Gale Wings
                true
            } else {
                // Prankster: solo funciona en movimientos de estado (power = None o power = 0)
                move_data.power.is_none() || move_data.power.unwrap_or(0) == 0
            };

            if !type_matches {
                continue;
            }

            // Verificar condiciones adicionales
            let condition_met = if let Some(cond) = condition {
                match cond {
                    PriorityCondition::FullHP => {
                        pokemon.current_hp == pokemon.base_computed_stats.hp
                    },
                    PriorityCondition::Poisoned => {
                        matches!(pokemon.status_condition, Some(StatusCondition::Poison) | Some(StatusCondition::BadPoison))
                    },
                }
            } else {
                true
            };

            if condition_met {
                return base_priority + priority_boost;
            }
        }
    }

    base_priority
}

// MIGRADO: resolve_move_data ahora está en infrastructure/move_data_loader.rs

/// Maneja los hooks de entrada de batalla (turno 1)
fn handle_entry_hazards(
    battle_state: &mut BattleState,
    player_team: &mut PlayerTeam,
    opponent_team: &mut Vec<PokemonInstance>,
    logs: &mut Vec<String>,
) {
    // Activar habilidades de entrada para todos los Pokémon activos

    // Jugador
    for &idx in &battle_state.player_active_indices.clone() {
        if let Some(player_mon) = player_team.active_members.get(idx) {
            let ability_id = &player_mon.ability;
            let hooks = get_ability_hooks(ability_id);

            // Obtener el nombre de la habilidad para logs
            let pokemon_name = player_mon.species.display_name.clone();
            let ability_name = ability_id.clone(); // TODO: Cargar nombre real de la habilidad

            // Filtrar solo hooks OnEntry
            for hook in hooks.iter().filter(|h| matches!(h.trigger, AbilityTrigger::OnEntry)) {
                match &hook.effect {
                    AbilityEffect::SetWeather { weather, duration } => {
                        // Solo activar si no es None
                        if *weather != WeatherType::None {
                            let weather_state = if *duration == 0 {
                                WeatherState::new(*weather) // Duración por defecto (5 turnos)
                            } else {
                                WeatherState { weather_type: *weather, turns_remaining: *duration }
                            };
                            battle_state.weather = Some(weather_state);

                            let weather_name = match weather {
                                WeatherType::Sun => "harsh sunlight",
                                WeatherType::Rain => "rain",
                                WeatherType::Sandstorm => "sandstorm",
                                WeatherType::Hail => "hail",
                                WeatherType::None => continue,
                            };
                            logs.push(format!("{}'s {} set {}!",
                                pokemon_name,
                                ability_name,
                                weather_name
                            ));
                        }
                    },
                    AbilityEffect::SetTerrain { terrain, duration } => {
                        let terrain_state = if *duration == 0 {
                            TerrainState::new(*terrain) // Duración por defecto (5 turnos)
                        } else {
                            TerrainState { terrain_type: *terrain, turns_remaining: *duration }
                        };
                        battle_state.terrain = Some(terrain_state);

                        let terrain_name = match terrain {
                            TerrainType::Electric => "Electric Terrain",
                            TerrainType::Grassy => "Grassy Terrain",
                            TerrainType::Misty => "Misty Terrain",
                            TerrainType::Psychic => "Psychic Terrain",
                        };
                        logs.push(format!("{}'s {} set {}!",
                            pokemon_name,
                            ability_name,
                            terrain_name
                        ));
                    },
                    AbilityEffect::ModifyStatOnEntry { stat, stages, target } => {
                        apply_on_entry_stat_change(
                            stat,
                            *stages,
                            target,
                            idx,
                            true,
                            battle_state,
                            player_team,
                            opponent_team,
                            &pokemon_name,
                            &ability_name,
                            logs,
                        );
                    },
                    _ => {},
                }
            }
        }
    }

    // Oponente
    for &idx in &battle_state.opponent_active_indices.clone() {
        if let Some(opponent_mon) = opponent_team.get(idx) {
            let ability_id = &opponent_mon.ability;
            let hooks = get_ability_hooks(ability_id);

            // Obtener el nombre de la habilidad para logs
            let pokemon_name = opponent_mon.species.display_name.clone();
            let ability_name = ability_id.clone(); // TODO: Cargar nombre real de la habilidad

            // Filtrar solo hooks OnEntry
            for hook in hooks.iter().filter(|h| matches!(h.trigger, AbilityTrigger::OnEntry)) {
                match &hook.effect {
                    AbilityEffect::SetWeather { weather, duration } => {
                        // Solo activar si no es None
                        if *weather != WeatherType::None {
                            let weather_state = if *duration == 0 {
                                WeatherState::new(*weather) // Duración por defecto (5 turnos)
                            } else {
                                WeatherState { weather_type: *weather, turns_remaining: *duration }
                            };
                            battle_state.weather = Some(weather_state);

                            let weather_name = match weather {
                                WeatherType::Sun => "harsh sunlight",
                                WeatherType::Rain => "rain",
                                WeatherType::Sandstorm => "sandstorm",
                                WeatherType::Hail => "hail",
                                WeatherType::None => continue,
                            };
                            logs.push(format!("{}'s {} set {}!",
                                pokemon_name,
                                ability_name,
                                weather_name
                            ));
                        }
                    },
                    AbilityEffect::SetTerrain { terrain, duration } => {
                        let terrain_state = if *duration == 0 {
                            TerrainState::new(*terrain) // Duración por defecto (5 turnos)
                        } else {
                            TerrainState { terrain_type: *terrain, turns_remaining: *duration }
                        };
                        battle_state.terrain = Some(terrain_state);

                        let terrain_name = match terrain {
                            TerrainType::Electric => "Electric Terrain",
                            TerrainType::Grassy => "Grassy Terrain",
                            TerrainType::Misty => "Misty Terrain",
                            TerrainType::Psychic => "Psychic Terrain",
                        };
                        logs.push(format!("{}'s {} set {}!",
                            pokemon_name,
                            ability_name,
                            terrain_name
                        ));
                    },
                    AbilityEffect::ModifyStatOnEntry { stat, stages, target } => {
                        apply_on_entry_stat_change(
                            stat,
                            *stages,
                            target,
                            idx,
                            false,
                            battle_state,
                            player_team,
                            opponent_team,
                            &pokemon_name,
                            &ability_name,
                            logs,
                        );
                    },
                    _ => {},
                }
            }
        }
    }
}

/// Aplica cambios de stats por habilidades OnEntry (ej: Intimidate, Download)
fn apply_on_entry_stat_change(
    stat: &str,
    stages: i8,
    target: &StatChangeTarget,
    user_idx: usize,
    is_player: bool,
    battle_state: &BattleState,
    player_team: &mut PlayerTeam,
    opponent_team: &mut Vec<PokemonInstance>,
    pokemon_name: &str,
    ability_name: &str,
    logs: &mut Vec<String>,
) {
    match target {
        StatChangeTarget::AllOpponents => {
            if is_player {
                // Afectar a todos los oponentes
                for &opp_idx in &battle_state.opponent_active_indices {
                    if let Some(opp) = opponent_team.get_mut(opp_idx) {
                        apply_stat_stage_change(opp, stat, stages);
                        let change_text = if stages > 0 { "rose" } else { "fell" };
                        logs.push(format!("{}'s {} {}!", opp.species.display_name, stat, change_text));
                    }
                }
                logs.insert(logs.len() - battle_state.opponent_active_indices.len(),
                    format!("{}'s {}!", pokemon_name, ability_name));
            } else {
                // Afectar a todos los jugadores
                for &player_idx in &battle_state.player_active_indices {
                    if let Some(player) = player_team.active_members.get_mut(player_idx) {
                        apply_stat_stage_change(player, stat, stages);
                        let change_text = if stages > 0 { "rose" } else { "fell" };
                        logs.push(format!("{}'s {} {}!", player.species.display_name, stat, change_text));
                    }
                }
                logs.insert(logs.len() - battle_state.player_active_indices.len(),
                    format!("{}'s {}!", pokemon_name, ability_name));
            }
        },
        StatChangeTarget::User => {
            if is_player {
                if let Some(player) = player_team.active_members.get_mut(user_idx) {
                    apply_stat_stage_change(player, stat, stages);
                    let change_text = if stages > 0 { "rose" } else { "fell" };
                    logs.push(format!("{}'s {}!", pokemon_name, ability_name));
                    logs.push(format!("{}'s {} {}!", pokemon_name, stat, change_text));
                }
            } else {
                if let Some(opp) = opponent_team.get_mut(user_idx) {
                    apply_stat_stage_change(opp, stat, stages);
                    let change_text = if stages > 0 { "rose" } else { "fell" };
                    logs.push(format!("{}'s {}!", pokemon_name, ability_name));
                    logs.push(format!("{}'s {} {}!", pokemon_name, stat, change_text));
                }
            }
        },
        _ => {},
    }
}

/// Aplica un cambio de stages a un stat específico
fn apply_stat_stage_change(pokemon: &mut PokemonInstance, stat: &str, stages: i8) {
    if let Some(ref mut battle_stages) = pokemon.battle_stages {
        match stat {
            "attack" => battle_stages.attack = (battle_stages.attack + stages).clamp(-6, 6),
            "defense" => battle_stages.defense = (battle_stages.defense + stages).clamp(-6, 6),
            "special-attack" | "special_attack" => battle_stages.special_attack = (battle_stages.special_attack + stages).clamp(-6, 6),
            "special-defense" | "special_defense" => battle_stages.special_defense = (battle_stages.special_defense + stages).clamp(-6, 6),
            "speed" => battle_stages.speed = (battle_stages.speed + stages).clamp(-6, 6),
            "accuracy" => battle_stages.accuracy = (battle_stages.accuracy + stages).clamp(-6, 6),
            "evasion" => battle_stages.evasion = (battle_stages.evasion + stages).clamp(-6, 6),
            _ => {},
        }
    }
}

/// Aplica efectos de habilidades al final del turno (Speed Boost, Regenerator, etc.)
fn apply_end_of_turn_abilities(
    pokemon: &mut PokemonInstance,
    battle_state: &BattleState,
    logs: &mut Vec<String>,
) {
    use super::ability_logic::HealCondition;

    let ability_id = &pokemon.ability;
    let hooks = get_ability_hooks(ability_id);

    // Filtrar solo hooks EndOfTurn
    for hook in hooks.iter().filter(|h| matches!(h.trigger, AbilityTrigger::EndOfTurn)) {
        match &hook.effect {
            // Boost de stat al final del turno (Speed Boost)
            AbilityEffect::BoostStatEndOfTurn { stat, stages } => {
                if let Some(ref mut battle_stages) = pokemon.battle_stages {
                    match stat.as_str() {
                        "speed" => {
                            let old_stage = battle_stages.speed;
                            battle_stages.speed = (battle_stages.speed + stages).clamp(-6, 6);
                            if battle_stages.speed != old_stage {
                                logs.push(format!("{}'s {} raised its Speed!", pokemon.species.display_name, ability_id));
                            }
                        },
                        "attack" => {
                            let old_stage = battle_stages.attack;
                            battle_stages.attack = (battle_stages.attack + stages).clamp(-6, 6);
                            if battle_stages.attack != old_stage {
                                logs.push(format!("{}'s {} raised its Attack!", pokemon.species.display_name, ability_id));
                            }
                        },
                        _ => {},
                    }
                }
            },
            // Curación al final del turno (Rain Dish, Ice Body)
            AbilityEffect::HealEndOfTurn { fraction, condition } => {
                let should_heal = match condition {
                    Some(HealCondition::Weather(required_weather)) => {
                        battle_state.weather.as_ref().map(|w| w.weather_type == *required_weather).unwrap_or(false)
                    },
                    Some(HealCondition::Terrain(required_terrain)) => {
                        battle_state.terrain.as_ref().map(|t| t.terrain_type == *required_terrain).unwrap_or(false)
                    },
                    None => true, // Siempre cura si no hay condición
                };

                if should_heal {
                    let heal_amount = ((pokemon.base_computed_stats.hp as f32) * fraction) as u16;
                    let old_hp = pokemon.current_hp;
                    pokemon.current_hp = (pokemon.current_hp + heal_amount).min(pokemon.base_computed_stats.hp);
                    let actual_heal = pokemon.current_hp - old_hp;

                    if actual_heal > 0 {
                        logs.push(format!("{}'s {} restored HP!", pokemon.species.display_name, ability_id));
                    }
                }
            },
            _ => {},
        }
    }
}

// MIGRADO: reset_turn_flags ahora está en systems/validation_system/state_resetter.rs

/// Procesa efectos residuales al final del turno
fn process_end_of_turn_residuals(
    battle_state: &mut BattleState,
    player_team: &mut PlayerTeam,
    opponent_team: &mut Vec<PokemonInstance>,
    logs: &mut Vec<String>,
) {
    // 1. Decrementar duración del clima
    if let Some(ref mut weather) = battle_state.weather {
        if weather.turns_remaining > 0 {
            weather.turns_remaining -= 1;
            if weather.turns_remaining == 0 {
                logs.push("¡El clima volvió a la normalidad!".to_string());
                battle_state.weather = None;
            }
        }
    }

    // 2. Decrementar duración del terreno
    if let Some(ref mut terrain) = battle_state.terrain {
        if terrain.turns_remaining > 0 {
            terrain.turns_remaining -= 1;
            if terrain.turns_remaining == 0 {
                logs.push("¡El terreno volvió a la normalidad!".to_string());
                battle_state.terrain = None;
            }
        }
    }

    // 3. Aplicar daño residual del clima a todos los Pokémon activos
    if battle_state.weather.is_some() {
        // Jugador
        for &idx in &battle_state.player_active_indices.clone() {
            if let Some(pokemon) = player_team.active_members.get_mut(idx) {
                if pokemon.current_hp > 0 {
                    let (damage, weather_logs) = apply_weather_residuals(pokemon, battle_state.weather.as_ref());
                    logs.extend(weather_logs);
                    if damage > 0 && pokemon.current_hp == 0 {
                        logs.push(format!("¡{} se debilitó!", pokemon.species.display_name));
                    }
                }
            }
        }

        // Oponente
        for &idx in &battle_state.opponent_active_indices.clone() {
            if let Some(pokemon) = opponent_team.get_mut(idx) {
                if pokemon.current_hp > 0 {
                    let (damage, weather_logs) = apply_weather_residuals(pokemon, battle_state.weather.as_ref());
                    logs.extend(weather_logs);
                    if damage > 0 && pokemon.current_hp == 0 {
                        logs.push(format!("¡{} se debilitó!", pokemon.species.display_name));
                    }
                }
            }
        }
    }

    // 4. Aplicar efectos residuales de estado (veneno, quemadura)
    // Jugador
    for &idx in &battle_state.player_active_indices.clone() {
        if let Some(pokemon) = player_team.active_members.get_mut(idx) {
            if pokemon.current_hp > 0 {
                let (damage, status_logs) = apply_residual_effects(pokemon);
                logs.extend(status_logs);
                if damage > 0 && pokemon.current_hp == 0 {
                    logs.push(format!("¡{} se debilitó!", pokemon.species.display_name));
                }
            }
        }
    }

    // Oponente
    for &idx in &battle_state.opponent_active_indices.clone() {
        if let Some(pokemon) = opponent_team.get_mut(idx) {
            if pokemon.current_hp > 0 {
                let (damage, status_logs) = apply_residual_effects(pokemon);
                logs.extend(status_logs);
                if damage > 0 && pokemon.current_hp == 0 {
                    logs.push(format!("¡{} se debilitó!", pokemon.species.display_name));
                }
            }
        }
    }

    // 5. Aplicar efectos de habilidades EndOfTurn (Speed Boost, Regenerator, etc.)
    // Jugador
    for &idx in &battle_state.player_active_indices.clone() {
        if let Some(pokemon) = player_team.active_members.get_mut(idx) {
            if pokemon.current_hp > 0 {
                apply_end_of_turn_abilities(pokemon, battle_state, logs);
            }
        }
    }

    // Oponente
    for &idx in &battle_state.opponent_active_indices.clone() {
        if let Some(pokemon) = opponent_team.get_mut(idx) {
            if pokemon.current_hp > 0 {
                apply_end_of_turn_abilities(pokemon, battle_state, logs);
            }
        }
    }
}

/// Verifica el estado de la batalla y determina el resultado
fn check_battle_state(
    battle_state: &mut BattleState,
    player_team: &PlayerTeam,
    logs: &mut Vec<String>,
) -> BattleOutcome {
    // Verificar si todos los Pokémon activos del jugador están debilitados
    let all_player_active_fainted = battle_state.player_active_indices.iter().all(|&idx| {
        player_team.active_members.get(idx).map(|p| p.current_hp == 0).unwrap_or(true)
    });

    // Verificar si todos los Pokémon activos del oponente están debilitados
    let all_opponent_active_fainted = if battle_state.is_trainer_battle {
        battle_state.opponent_active_indices.iter().all(|&idx| {
            battle_state.opponent_team.get(idx).map(|p| p.current_hp == 0).unwrap_or(true)
        })
    } else {
        battle_state.opponent_instance.current_hp == 0
    };

    // Si todos los oponentes activos están debilitados
    if all_opponent_active_fainted {
        // Intentar cambiar a un nuevo oponente
        return determine_enemy_outcome(battle_state, logs);
    }

    // Si todos los jugadores activos están debilitados
    if all_player_active_fainted {
        // Verificar si hay más Pokémon disponibles
        return determine_player_outcome(player_team, battle_state);
    }

    // La batalla continúa
    BattleOutcome::Continue
}

// MIGRADO: Las siguientes funciones ahora están en infrastructure/pokemon_accessor.rs
// - is_pokemon_alive
// - get_pokemon
// - get_pokemon_mut
// - get_team_index
