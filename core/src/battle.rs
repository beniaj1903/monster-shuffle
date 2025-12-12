use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::models::{MoveData, PokemonInstance, PokemonType, StatusCondition};
use crate::game::{BattleState, PlayerTeam};

/// Resultado de la batalla después de un turno
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum BattleOutcome {
    /// Nadie murió, la batalla continúa
    Continue,
    /// Todos los enemigos debilitados - Jugador ganó
    PlayerWon,
    /// Todos los Pokémon del jugador debilitados - Jugador perdió
    PlayerLost,
    /// El Pokémon activo del jugador murió, pero le quedan otros vivos
    PlayerMustSwitch,
    /// El enemigo murió y sacó uno nuevo (útil para logs/animación)
    EnemySwitched,
}

/// Resultado de ejecutar un turno de batalla
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurnResult {
    /// Narración de lo que ocurrió en el turno
    pub logs: Vec<String>,
    /// Daño infligido por el jugador
    pub player_damage_dealt: u16,
    /// Daño infligido por el enemigo
    pub enemy_damage_dealt: u16,
    /// Resultado de la batalla después de este turno
    pub outcome: BattleOutcome,
}

impl TurnResult {
    /// Crea un nuevo TurnResult vacío
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            player_damage_dealt: 0,
            enemy_damage_dealt: 0,
            outcome: BattleOutcome::Continue,
        }
    }
}

impl Default for TurnResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Convierte un string de tipo a PokemonType enum
fn parse_type(type_str: &str) -> PokemonType {
    match type_str.to_lowercase().as_str() {
        "normal" => PokemonType::Normal,
        "fire" => PokemonType::Fire,
        "water" => PokemonType::Water,
        "grass" => PokemonType::Grass,
        "electric" => PokemonType::Electric,
        "ice" => PokemonType::Ice,
        "fighting" => PokemonType::Fighting,
        "poison" => PokemonType::Poison,
        "ground" => PokemonType::Ground,
        "flying" => PokemonType::Flying,
        "psychic" => PokemonType::Psychic,
        "bug" => PokemonType::Bug,
        "rock" => PokemonType::Rock,
        "ghost" => PokemonType::Ghost,
        "dragon" => PokemonType::Dragon,
        "dark" => PokemonType::Dark,
        "steel" => PokemonType::Steel,
        "fairy" => PokemonType::Fairy,
        _ => PokemonType::Unknown,
    }
}

/// Calcula la efectividad de tipo entre un movimiento y un Pokémon
/// Retorna un multiplicador (0.0, 0.25, 0.5, 1.0, 2.0, 4.0)
fn get_type_effectiveness(move_type: &PokemonType, defender_types: &[PokemonType]) -> f32 {
    let mut multiplier = 1.0;

    for defender_type in defender_types {
        multiplier *= match (move_type, defender_type) {
            // Normal
            (PokemonType::Normal, PokemonType::Rock) => 0.5,
            (PokemonType::Normal, PokemonType::Ghost) => 0.0,
            (PokemonType::Normal, PokemonType::Steel) => 0.5,
            // Fire
            (PokemonType::Fire, PokemonType::Fire) => 0.5,
            (PokemonType::Fire, PokemonType::Water) => 0.5,
            (PokemonType::Fire, PokemonType::Grass) => 2.0,
            (PokemonType::Fire, PokemonType::Ice) => 2.0,
            (PokemonType::Fire, PokemonType::Bug) => 2.0,
            (PokemonType::Fire, PokemonType::Rock) => 0.5,
            (PokemonType::Fire, PokemonType::Dragon) => 0.5,
            (PokemonType::Fire, PokemonType::Steel) => 2.0,
            // Water
            (PokemonType::Water, PokemonType::Fire) => 2.0,
            (PokemonType::Water, PokemonType::Water) => 0.5,
            (PokemonType::Water, PokemonType::Grass) => 0.5,
            (PokemonType::Water, PokemonType::Ground) => 2.0,
            (PokemonType::Water, PokemonType::Rock) => 2.0,
            (PokemonType::Water, PokemonType::Dragon) => 0.5,
            // Grass
            (PokemonType::Grass, PokemonType::Fire) => 0.5,
            (PokemonType::Grass, PokemonType::Water) => 2.0,
            (PokemonType::Grass, PokemonType::Grass) => 0.5,
            (PokemonType::Grass, PokemonType::Poison) => 0.5,
            (PokemonType::Grass, PokemonType::Ground) => 2.0,
            (PokemonType::Grass, PokemonType::Flying) => 0.5,
            (PokemonType::Grass, PokemonType::Bug) => 0.5,
            (PokemonType::Grass, PokemonType::Rock) => 2.0,
            (PokemonType::Grass, PokemonType::Dragon) => 0.5,
            (PokemonType::Grass, PokemonType::Steel) => 0.5,
            // Electric
            (PokemonType::Electric, PokemonType::Water) => 2.0,
            (PokemonType::Electric, PokemonType::Electric) => 0.5,
            (PokemonType::Electric, PokemonType::Grass) => 0.5,
            (PokemonType::Electric, PokemonType::Ground) => 0.0,
            (PokemonType::Electric, PokemonType::Flying) => 2.0,
            (PokemonType::Electric, PokemonType::Dragon) => 0.5,
            // Ice
            (PokemonType::Ice, PokemonType::Fire) => 0.5,
            (PokemonType::Ice, PokemonType::Water) => 0.5,
            (PokemonType::Ice, PokemonType::Grass) => 2.0,
            (PokemonType::Ice, PokemonType::Ice) => 0.5,
            (PokemonType::Ice, PokemonType::Ground) => 2.0,
            (PokemonType::Ice, PokemonType::Flying) => 2.0,
            (PokemonType::Ice, PokemonType::Dragon) => 2.0,
            (PokemonType::Ice, PokemonType::Steel) => 0.5,
            // Fighting
            (PokemonType::Fighting, PokemonType::Normal) => 2.0,
            (PokemonType::Fighting, PokemonType::Ice) => 2.0,
            (PokemonType::Fighting, PokemonType::Poison) => 0.5,
            (PokemonType::Fighting, PokemonType::Flying) => 0.5,
            (PokemonType::Fighting, PokemonType::Psychic) => 0.5,
            (PokemonType::Fighting, PokemonType::Bug) => 0.5,
            (PokemonType::Fighting, PokemonType::Rock) => 2.0,
            (PokemonType::Fighting, PokemonType::Ghost) => 0.0,
            (PokemonType::Fighting, PokemonType::Dark) => 2.0,
            (PokemonType::Fighting, PokemonType::Steel) => 2.0,
            (PokemonType::Fighting, PokemonType::Fairy) => 0.5,
            // Poison
            (PokemonType::Poison, PokemonType::Grass) => 2.0,
            (PokemonType::Poison, PokemonType::Poison) => 0.5,
            (PokemonType::Poison, PokemonType::Ground) => 0.5,
            (PokemonType::Poison, PokemonType::Rock) => 0.5,
            (PokemonType::Poison, PokemonType::Ghost) => 0.5,
            (PokemonType::Poison, PokemonType::Steel) => 0.0,
            (PokemonType::Poison, PokemonType::Fairy) => 2.0,
            // Ground
            (PokemonType::Ground, PokemonType::Fire) => 2.0,
            (PokemonType::Ground, PokemonType::Electric) => 2.0,
            (PokemonType::Ground, PokemonType::Grass) => 0.5,
            (PokemonType::Ground, PokemonType::Poison) => 2.0,
            (PokemonType::Ground, PokemonType::Flying) => 0.0,
            (PokemonType::Ground, PokemonType::Bug) => 0.5,
            (PokemonType::Ground, PokemonType::Rock) => 2.0,
            (PokemonType::Ground, PokemonType::Steel) => 2.0,
            // Flying
            (PokemonType::Flying, PokemonType::Electric) => 0.5,
            (PokemonType::Flying, PokemonType::Grass) => 2.0,
            (PokemonType::Flying, PokemonType::Fighting) => 2.0,
            (PokemonType::Flying, PokemonType::Bug) => 2.0,
            (PokemonType::Flying, PokemonType::Rock) => 0.5,
            (PokemonType::Flying, PokemonType::Steel) => 0.5,
            // Psychic
            (PokemonType::Psychic, PokemonType::Fighting) => 2.0,
            (PokemonType::Psychic, PokemonType::Poison) => 2.0,
            (PokemonType::Psychic, PokemonType::Psychic) => 0.5,
            (PokemonType::Psychic, PokemonType::Dark) => 0.0,
            (PokemonType::Psychic, PokemonType::Steel) => 0.5,
            // Bug
            (PokemonType::Bug, PokemonType::Fire) => 0.5,
            (PokemonType::Bug, PokemonType::Grass) => 2.0,
            (PokemonType::Bug, PokemonType::Fighting) => 0.5,
            (PokemonType::Bug, PokemonType::Poison) => 0.5,
            (PokemonType::Bug, PokemonType::Flying) => 0.5,
            (PokemonType::Bug, PokemonType::Psychic) => 2.0,
            (PokemonType::Bug, PokemonType::Ghost) => 0.5,
            (PokemonType::Bug, PokemonType::Dark) => 2.0,
            (PokemonType::Bug, PokemonType::Steel) => 0.5,
            (PokemonType::Bug, PokemonType::Fairy) => 0.5,
            // Rock
            (PokemonType::Rock, PokemonType::Fire) => 2.0,
            (PokemonType::Rock, PokemonType::Ice) => 2.0,
            (PokemonType::Rock, PokemonType::Fighting) => 0.5,
            (PokemonType::Rock, PokemonType::Ground) => 0.5,
            (PokemonType::Rock, PokemonType::Flying) => 2.0,
            (PokemonType::Rock, PokemonType::Bug) => 2.0,
            (PokemonType::Rock, PokemonType::Steel) => 0.5,
            // Ghost
            (PokemonType::Ghost, PokemonType::Normal) => 0.0,
            (PokemonType::Ghost, PokemonType::Psychic) => 2.0,
            (PokemonType::Ghost, PokemonType::Ghost) => 2.0,
            (PokemonType::Ghost, PokemonType::Dark) => 0.5,
            // Dragon
            (PokemonType::Dragon, PokemonType::Dragon) => 2.0,
            (PokemonType::Dragon, PokemonType::Steel) => 0.5,
            (PokemonType::Dragon, PokemonType::Fairy) => 0.0,
            // Dark
            (PokemonType::Dark, PokemonType::Fighting) => 0.5,
            (PokemonType::Dark, PokemonType::Psychic) => 2.0,
            (PokemonType::Dark, PokemonType::Ghost) => 2.0,
            (PokemonType::Dark, PokemonType::Dark) => 0.5,
            (PokemonType::Dark, PokemonType::Fairy) => 0.5,
            // Steel
            (PokemonType::Steel, PokemonType::Fire) => 0.5,
            (PokemonType::Steel, PokemonType::Water) => 0.5,
            (PokemonType::Steel, PokemonType::Electric) => 0.5,
            (PokemonType::Steel, PokemonType::Ice) => 2.0,
            (PokemonType::Steel, PokemonType::Rock) => 2.0,
            (PokemonType::Steel, PokemonType::Steel) => 0.5,
            (PokemonType::Steel, PokemonType::Fairy) => 2.0,
            // Fairy
            (PokemonType::Fairy, PokemonType::Fighting) => 2.0,
            (PokemonType::Fairy, PokemonType::Poison) => 0.5,
            (PokemonType::Fairy, PokemonType::Steel) => 0.5,
            (PokemonType::Fairy, PokemonType::Fire) => 0.5,
            (PokemonType::Fairy, PokemonType::Dragon) => 2.0,
            (PokemonType::Fairy, PokemonType::Dark) => 2.0,
            _ => 1.0,
        };
    }

    multiplier
}

/// Verifica si un movimiento tiene STAB (Same Type Attack Bonus)
fn has_stab(attacker: &PokemonInstance, move_type: &PokemonType) -> bool {
    attacker.randomized_profile.rolled_primary_type == *move_type
        || attacker.randomized_profile.rolled_secondary_type.as_ref()
            .map(|t| t == move_type)
            .unwrap_or(false)
}

/// Calcula el daño de un movimiento
/// Retorna (daño, mensaje de efectividad)
fn calculate_damage(
    attacker: &PokemonInstance,
    defender: &PokemonInstance,
    move_data: &MoveData,
    rng: &mut StdRng,
) -> (u16, String) {
    // Si el movimiento no tiene poder, retorna 0
    let Some(power) = move_data.power else {
        return (0, String::new());
    };

    // Obtener el tipo del movimiento
    let move_type = parse_type(&move_data.r#type);

    // Calcular efectividad de tipo
    let mut defender_types = vec![defender.randomized_profile.rolled_primary_type];
    if let Some(secondary) = defender.randomized_profile.rolled_secondary_type {
        defender_types.push(secondary);
    }

    let type_effectiveness = get_type_effectiveness(&move_type, &defender_types);

    // Verificar STAB
    let stab_multiplier = if has_stab(attacker, &move_type) {
        1.5
    } else {
        1.0
    };

    // Determinar el stat de ataque y defensa según la clase de daño
    let (attack_stat, defense_stat) = if move_data.damage_class == "physical" {
        (
            attacker.base_computed_stats.attack,
            defender.base_computed_stats.defense,
        )
    } else if move_data.damage_class == "special" {
        (
            attacker.base_computed_stats.special_attack,
            defender.base_computed_stats.special_defense,
        )
    } else {
        // "status" - no hace daño
        return (0, String::new());
    };

    // Fórmula de daño Gen 3+
    // Damage = ((((2 * Level / 5 + 2) * Power * A / D) / 50) + 2) * Modifiers
    let level = attacker.level as f32;
    let power = power as f32;
    let attack = attack_stat as f32;
    let defense = defense_stat as f32;

    let base_damage = ((2.0 * level / 5.0 + 2.0) * power * attack / defense) / 50.0 + 2.0;

    // Modificadores: STAB y efectividad de tipo
    let modifiers = stab_multiplier * type_effectiveness;

    // Factor aleatorio (0.85 - 1.0)
    let random_factor = rng.gen_range(0.85..=1.0);

    let final_damage = (base_damage * modifiers * random_factor) as u16;

    // Mensaje de efectividad
    let effectiveness_msg = if type_effectiveness >= 2.0 {
        "¡Es súper efectivo!".to_string()
    } else if type_effectiveness <= 0.5 {
        "No es muy efectivo...".to_string()
    } else {
        String::new()
    };

    // Asegurar que el daño sea al menos 1 si el movimiento tiene poder y no es ineficaz
    let damage = if type_effectiveness == 0.0 {
        0
    } else {
        final_damage.max(1)
    };

    (damage, effectiveness_msg)
}

/// Aplica daño de condiciones de estado (quemadura, veneno, etc.)
fn apply_status_damage(pokemon: &mut PokemonInstance, logs: &mut Vec<String>) {
    if let Some(status) = &pokemon.status_condition {
        match status {
            StatusCondition::Burn => {
                let damage = pokemon.base_computed_stats.hp / 16;
                if damage >= pokemon.current_hp {
                    pokemon.current_hp = 0;
                    logs.push(format!("{} se quemó y perdió {} HP!", pokemon.species.display_name, damage));
                } else {
                    pokemon.current_hp -= damage;
                    logs.push(format!("{} se quemó y perdió {} HP!", pokemon.species.display_name, damage));
                }
            }
            StatusCondition::Poison => {
                let damage = pokemon.base_computed_stats.hp / 8;
                if damage >= pokemon.current_hp {
                    pokemon.current_hp = 0;
                    logs.push(format!("{} se envenenó y perdió {} HP!", pokemon.species.display_name, damage));
                } else {
                    pokemon.current_hp -= damage;
                    logs.push(format!("{} se envenenó y perdió {} HP!", pokemon.species.display_name, damage));
                }
            }
            StatusCondition::BadPoison => {
                // Toxic: aumenta el daño cada turno (simplificado: daño fijo alto)
                let damage = pokemon.base_computed_stats.hp / 4;
                if damage >= pokemon.current_hp {
                    pokemon.current_hp = 0;
                    logs.push(format!("{} recibió {} HP de daño por Toxic!", pokemon.species.display_name, damage));
                } else {
                    pokemon.current_hp -= damage;
                    logs.push(format!("{} recibió {} HP de daño por Toxic!", pokemon.species.display_name, damage));
                }
            }
            _ => {}
        }
    }
}

/// Verifica si un Pokémon puede moverse basándose en su condición de estado
fn can_move(pokemon: &PokemonInstance, rng: &mut StdRng) -> bool {
    if let Some(status) = &pokemon.status_condition {
        match status {
            StatusCondition::Freeze => {
                // 20% de probabilidad de descongelarse
                if rng.gen_bool(0.2) {
                    return true;
                }
                return false;
            }
            StatusCondition::Sleep => {
                // Simplificado: duerme por un número aleatorio de turnos
                // Por ahora, siempre duerme (se puede mejorar)
                return false;
            }
            StatusCondition::Paralysis => {
                // 25% de probabilidad de no poder moverse
                if rng.gen_bool(0.25) {
                    return false;
                }
            }
            _ => {}
        }
    }
    true
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

/// Ejecuta un turno completo de batalla
/// 
/// Determina quién ataca primero basándose en la velocidad.
/// Si hay empate, se decide al azar.
/// 
/// Flujo secuencial estricto:
/// 1. Determina orden de ataque (first y second)
/// 2. First ataca a second
/// 3. Si second se debilita, verifica reservas y retorna inmediatamente
/// 4. Solo si second sigue vivo, second ataca a first
/// 5. Si first se debilita, verifica reservas y retorna
/// 
/// Retorna un TurnResult con los logs y el resultado de la batalla.
pub fn execute_turn(
    player_mon: &mut PokemonInstance,
    enemy_mon: &mut PokemonInstance,
    player_move: &MoveData,
    enemy_move: &MoveData,
    player_team: &PlayerTeam,
    player_active_index: usize,
    battle_state: &mut BattleState,
    rng: &mut StdRng,
) -> TurnResult {
    let mut result = TurnResult::new();

    // PASO 1: Determinar quién ataca primero basándose en la velocidad
    let player_speed = player_mon.base_computed_stats.speed;
    let enemy_speed = enemy_mon.base_computed_stats.speed;

    let player_goes_first = if player_speed > enemy_speed {
        true
    } else if enemy_speed > player_speed {
        false
    } else {
        // Empate: decidir al azar
        rng.gen_bool(0.5)
    };

    // Verificar condiciones de estado y precisión ANTES de determinar quién es first/second
    let player_can_move = can_move(player_mon, rng);
    let enemy_can_move = can_move(enemy_mon, rng);
    
    let player_move_hits = if !player_can_move {
        false
    } else if let Some(accuracy) = player_move.accuracy {
        let roll = rng.gen_range(0..=100);
        roll <= accuracy as u32
    } else {
        true
    };

    let enemy_move_hits = if !enemy_can_move {
        false
    } else if let Some(accuracy) = enemy_move.accuracy {
        let roll = rng.gen_range(0..=100);
        roll <= accuracy as u32
    } else {
        true
    };

    // PASO 2: Primer ataque
    if player_goes_first {
        // Jugador ataca primero
        result.logs.push(format!(
            "{} usó {}",
            player_mon.species.display_name, player_move.name
        ));

        if player_move_hits {
            let (damage, effectiveness_msg) = calculate_damage(player_mon, enemy_mon, player_move, rng);
            result.player_damage_dealt = damage;

            if !effectiveness_msg.is_empty() {
                result.logs.push(effectiveness_msg);
            }

            // Aplicar daño
            if damage >= enemy_mon.current_hp {
                enemy_mon.current_hp = 0;
                result.logs.push(format!(
                    "{} recibió {} de daño",
                    enemy_mon.species.display_name, damage
                ));
                result.logs.push(format!(
                    "{} se debilitó",
                    enemy_mon.species.display_name
                ));
                // IMPORTANTE: Actualizar el HP del oponente en el battle_state ANTES de verificar reservas
                // Esto asegura que cuando switch_to_next_opponent busque otros Pokémon, tenga el estado correcto
                *battle_state.get_opponent_active_mut() = enemy_mon.clone();
                // CHECK CRÍTICO: Si el enemigo se debilitó, verificar reservas y retornar
                result.outcome = determine_enemy_outcome(battle_state);
                return result;
            } else {
                enemy_mon.current_hp -= damage;
                result.logs.push(format!(
                    "{} recibió {} de daño",
                    enemy_mon.species.display_name, damage
                ));
            }

            // Aplicar daño de condiciones de estado
            apply_status_damage(enemy_mon, &mut result.logs);
            
            // Verificar si se debilitó por condición de estado
            if enemy_mon.current_hp == 0 {
                result.logs.push(format!(
                    "{} se debilitó",
                    enemy_mon.species.display_name
                ));
                // IMPORTANTE: Actualizar el HP del oponente en el battle_state ANTES de verificar reservas
                *battle_state.get_opponent_active_mut() = enemy_mon.clone();
                result.outcome = determine_enemy_outcome(battle_state);
                return result;
            }
        } else {
            result.logs.push("¡Pero falló!".to_string());
        }

        // PASO 3: Segundo ataque - Solo si el enemigo sigue vivo
        result.logs.push(format!(
            "El enemigo {} usó {}",
            enemy_mon.species.display_name, enemy_move.name
        ));

        if enemy_move_hits {
            let (damage, effectiveness_msg) = calculate_damage(enemy_mon, player_mon, enemy_move, rng);
            result.enemy_damage_dealt = damage;

            if !effectiveness_msg.is_empty() {
                result.logs.push(effectiveness_msg);
            }

            // Aplicar daño
            if damage >= player_mon.current_hp {
                player_mon.current_hp = 0;
                result.logs.push(format!(
                    "{} recibió {} de daño",
                    player_mon.species.display_name, damage
                ));
                result.logs.push(format!(
                    "{} se debilitó",
                    player_mon.species.display_name
                ));
                // CHECK CRÍTICO: Si el jugador se debilitó, verificar reservas y retornar
                result.outcome = determine_player_outcome(player_team, player_active_index);
                return result;
            } else {
                player_mon.current_hp -= damage;
                result.logs.push(format!(
                    "{} recibió {} de daño",
                    player_mon.species.display_name, damage
                ));
            }

            // Aplicar daño de condiciones de estado
            apply_status_damage(player_mon, &mut result.logs);
            
            // Verificar si se debilitó por condición de estado
            if player_mon.current_hp == 0 {
                result.logs.push(format!(
                    "{} se debilitó",
                    player_mon.species.display_name
                ));
                result.outcome = determine_player_outcome(player_team, player_active_index);
                return result;
            }
        } else {
            result.logs.push("¡Pero falló!".to_string());
        }
    } else {
        // Enemigo ataca primero
        result.logs.push(format!(
            "El enemigo {} usó {}",
            enemy_mon.species.display_name, enemy_move.name
        ));

        if enemy_move_hits {
            let (damage, effectiveness_msg) = calculate_damage(enemy_mon, player_mon, enemy_move, rng);
            result.enemy_damage_dealt = damage;

            if !effectiveness_msg.is_empty() {
                result.logs.push(effectiveness_msg);
            }

            // Aplicar daño
            if damage >= player_mon.current_hp {
                player_mon.current_hp = 0;
                result.logs.push(format!(
                    "{} recibió {} de daño",
                    player_mon.species.display_name, damage
                ));
                result.logs.push(format!(
                    "{} se debilitó",
                    player_mon.species.display_name
                ));
                // CHECK CRÍTICO: Si el jugador se debilitó, verificar reservas y retornar
                result.outcome = determine_player_outcome(player_team, player_active_index);
                return result;
            } else {
                player_mon.current_hp -= damage;
                result.logs.push(format!(
                    "{} recibió {} de daño",
                    player_mon.species.display_name, damage
                ));
            }

            // Aplicar daño de condiciones de estado
            apply_status_damage(player_mon, &mut result.logs);
            
            // Verificar si se debilitó por condición de estado
            if player_mon.current_hp == 0 {
                result.logs.push(format!(
                    "{} se debilitó",
                    player_mon.species.display_name
                ));
                result.outcome = determine_player_outcome(player_team, player_active_index);
                return result;
            }
        } else {
            result.logs.push("¡Pero falló!".to_string());
        }

        // PASO 3: Segundo ataque - Solo si el jugador sigue vivo
        result.logs.push(format!(
            "{} usó {}",
            player_mon.species.display_name, player_move.name
        ));

        if player_move_hits {
            let (damage, effectiveness_msg) = calculate_damage(player_mon, enemy_mon, player_move, rng);
            result.player_damage_dealt = damage;

            if !effectiveness_msg.is_empty() {
                result.logs.push(effectiveness_msg);
            }

            // Aplicar daño
            if damage >= enemy_mon.current_hp {
                enemy_mon.current_hp = 0;
                result.logs.push(format!(
                    "{} recibió {} de daño",
                    enemy_mon.species.display_name, damage
                ));
                result.logs.push(format!(
                    "{} se debilitó",
                    enemy_mon.species.display_name
                ));
                // IMPORTANTE: Actualizar el HP del oponente en el battle_state ANTES de verificar reservas
                // Esto asegura que cuando switch_to_next_opponent busque otros Pokémon, tenga el estado correcto
                *battle_state.get_opponent_active_mut() = enemy_mon.clone();
                // CHECK CRÍTICO: Si el enemigo se debilitó, verificar reservas y retornar
                result.outcome = determine_enemy_outcome(battle_state);
                return result;
            } else {
                enemy_mon.current_hp -= damage;
                result.logs.push(format!(
                    "{} recibió {} de daño",
                    enemy_mon.species.display_name, damage
                ));
            }

            // Aplicar daño de condiciones de estado
            apply_status_damage(enemy_mon, &mut result.logs);
            
            // Verificar si se debilitó por condición de estado
            if enemy_mon.current_hp == 0 {
                result.logs.push(format!(
                    "{} se debilitó",
                    enemy_mon.species.display_name
                ));
                // IMPORTANTE: Actualizar el HP del oponente en el battle_state ANTES de verificar reservas
                *battle_state.get_opponent_active_mut() = enemy_mon.clone();
                result.outcome = determine_enemy_outcome(battle_state);
                return result;
            }
        } else {
            result.logs.push("¡Pero falló!".to_string());
        }
    }

    // Si llegamos aquí, ambos Pokémon siguen vivos
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
    rng: &mut StdRng,
) -> TurnResult {
    let mut result = TurnResult::new();

    // Verificar si el enemigo puede moverse (condiciones de estado)
    let enemy_can_move = can_move(enemy_mon, rng);

    // Verificar precisión del movimiento del enemigo (si aplica)
    let enemy_move_hits = if !enemy_can_move {
        false
    } else if let Some(accuracy) = enemy_move.accuracy {
        let roll = rng.gen_range(0..=100);
        roll <= accuracy as u32
    } else {
        // Movimientos sin precisión siempre aciertan
        true
    };

    // El enemigo ataca
    result.logs.push(format!(
        "El enemigo {} usó {}",
        enemy_mon.species.display_name, enemy_move.name
    ));

    if enemy_move_hits {
        let (damage, effectiveness_msg) = calculate_damage(enemy_mon, player_mon, enemy_move, rng);
        result.enemy_damage_dealt = damage;

        if !effectiveness_msg.is_empty() {
            result.logs.push(effectiveness_msg);
        }

        // Aplicar daño al jugador
        if damage >= player_mon.current_hp {
            player_mon.current_hp = 0;
            result.logs.push(format!(
                "{} recibió {} de daño",
                player_mon.species.display_name, damage
            ));
            result.logs.push(format!(
                "{} se debilitó",
                player_mon.species.display_name
            ));
            // Nota: Para execute_enemy_attack, no tenemos acceso a player_team
            // El handler deberá verificar esto después
            result.outcome = BattleOutcome::PlayerMustSwitch;
        } else {
            player_mon.current_hp -= damage;
            result.logs.push(format!(
                "{} recibió {} de daño",
                player_mon.species.display_name, damage
            ));
        }

        // Aplicar daño de condiciones de estado
        apply_status_damage(player_mon, &mut result.logs);
    } else {
        result.logs.push("¡Pero falló!".to_string());
    }

    result
}
