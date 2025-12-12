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

/// Calcula el multiplicador de stat basado en el stage
/// Fórmula estándar de Pokémon:
/// - Si stage >= 0: (2.0 + stage) / 2.0
/// - Si stage < 0: 2.0 / (2.0 + abs(stage))
fn get_stat_multiplier(stage: i8) -> f32 {
    if stage >= 0 {
        (2.0 + stage as f32) / 2.0
    } else {
        2.0 / (2.0 - stage as f32)
    }
}

/// Calcula la velocidad efectiva considerando stages y condiciones de estado
fn get_effective_speed(pokemon: &PokemonInstance) -> f32 {
    let base_speed = pokemon.base_computed_stats.speed as f32;
    
    // Aplicar multiplicador de stage de velocidad
    let speed_multiplier = if let Some(ref stages) = pokemon.battle_stages {
        get_stat_multiplier(stages.speed)
    } else {
        1.0
    };
    
    let speed_with_stages = base_speed * speed_multiplier;
    
    // Aplicar efecto de parálisis (reduce velocidad a la mitad)
    if pokemon.status_condition == Some(StatusCondition::Paralysis) {
        speed_with_stages * 0.5
    } else {
        speed_with_stages
    }
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
    let (base_attack_stat, base_defense_stat, attack_stat_name, defense_stat_name) = if move_data.damage_class == "physical" {
        (
            attacker.base_computed_stats.attack,
            defender.base_computed_stats.defense,
            "attack",
            "defense",
        )
    } else if move_data.damage_class == "special" {
        (
            attacker.base_computed_stats.special_attack,
            defender.base_computed_stats.special_defense,
            "special_attack",
            "special_defense",
        )
    } else {
        // "status" - no hace daño
        return (0, String::new());
    };

    // Aplicar multiplicadores de stats stages
    let attack_multiplier = if let Some(ref stages) = attacker.battle_stages {
        get_stat_multiplier(match attack_stat_name {
            "attack" => stages.attack,
            "special_attack" => stages.special_attack,
            _ => 0,
        })
    } else {
        1.0
    };

    let defense_multiplier = if let Some(ref stages) = defender.battle_stages {
        get_stat_multiplier(match defense_stat_name {
            "defense" => stages.defense,
            "special_defense" => stages.special_defense,
            _ => 0,
        })
    } else {
        1.0
    };

    // Calcular stats efectivos
    let attack = (base_attack_stat as f32) * attack_multiplier;
    let defense = (base_defense_stat as f32) * defense_multiplier;

    // Fórmula de daño Gen 3+
    // Damage = ((((2 * Level / 5 + 2) * Power * A / D) / 50) + 2) * Modifiers
    let level = attacker.level as f32;
    let power = power as f32;

    let base_damage = ((2.0 * level / 5.0 + 2.0) * power * attack / defense) / 50.0 + 2.0;

    // Modificadores: STAB y efectividad de tipo
    let mut modifiers = stab_multiplier * type_effectiveness;

    // Efecto de Quemadura (Burn): reduce el daño físico a la mitad
    if move_data.damage_class == "physical" && attacker.status_condition == Some(StatusCondition::Burn) {
        modifiers *= 0.5;
    }

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

/// Aplica los efectos secundarios de un movimiento (cambios de stats y estados alterados)
fn apply_move_secondary_effects(
    move_data: &MoveData,
    attacker: &mut PokemonInstance,
    defender: &mut PokemonInstance,
    logs: &mut Vec<String>,
    rng: &mut StdRng,
) {
    // Aplicar cambios de stats
    if !move_data.stat_changes.is_empty() {
        // Verificar probabilidad de stat_chance (si es 0, asumimos 100%)
        let stat_chance = if move_data.meta.stat_chance == 0 {
            100
        } else {
            move_data.meta.stat_chance
        };

        let roll = rng.gen_range(0..=100);
        if roll <= stat_chance as u32 {
            // Guardar los nombres antes del loop para evitar problemas de borrow
            let attacker_name = attacker.species.display_name.clone();
            let defender_name = defender.species.display_name.clone();
            
            for stat_change in &move_data.stat_changes {
                // Determinar el objetivo del cambio basándose en el campo 'target' del movimiento
                // Si target es "user", aplicar al atacante (quien usó el movimiento)
                // Si target es "selected-pokemon", "all-opponents", o cualquier otro, aplicar al defensor
                let apply_to_user = move_data.target == "user";
                
                // Aplicar el cambio según el objetivo
                if apply_to_user {
                    // Inicializar battle_stages si no existen
                    if attacker.battle_stages.is_none() {
                        attacker.init_battle_stages();
                    }

                    if let Some(ref mut stages) = attacker.battle_stages {
                        let old_stage = match stat_change.stat.as_str() {
                            "attack" => stages.attack,
                            "defense" => stages.defense,
                            "special_attack" => stages.special_attack,
                            "special_defense" => stages.special_defense,
                            "speed" => stages.speed,
                            "accuracy" => stages.accuracy,
                            "evasion" => stages.evasion,
                            _ => continue, // Ignorar stats desconocidos
                        };

                        stages.apply_change(&stat_change.stat, stat_change.change);
                        let new_stage = match stat_change.stat.as_str() {
                            "attack" => stages.attack,
                            "defense" => stages.defense,
                            "special_attack" => stages.special_attack,
                            "special_defense" => stages.special_defense,
                            "speed" => stages.speed,
                            "accuracy" => stages.accuracy,
                            "evasion" => stages.evasion,
                            _ => continue,
                        };

                        // Generar mensaje de log
                        let stat_name = match stat_change.stat.as_str() {
                            "attack" => "ataque",
                            "defense" => "defensa",
                            "special_attack" => "ataque especial",
                            "special_defense" => "defensa especial",
                            "speed" => "velocidad",
                            "accuracy" => "precisión",
                            "evasion" => "evasión",
                            _ => continue,
                        };

                        if new_stage > old_stage {
                            logs.push(format!(
                                "¡El {} de {} subió!",
                                stat_name,
                                attacker_name
                            ));
                        } else if new_stage < old_stage {
                            logs.push(format!(
                                "¡El {} de {} bajó!",
                                stat_name,
                                attacker_name
                            ));
                        }
                    }
                } else {
                    // Afecta al defensor
                    // Inicializar battle_stages si no existen
                    if defender.battle_stages.is_none() {
                        defender.init_battle_stages();
                    }

                    if let Some(ref mut stages) = defender.battle_stages {
                        let old_stage = match stat_change.stat.as_str() {
                            "attack" => stages.attack,
                            "defense" => stages.defense,
                            "special_attack" => stages.special_attack,
                            "special_defense" => stages.special_defense,
                            "speed" => stages.speed,
                            "accuracy" => stages.accuracy,
                            "evasion" => stages.evasion,
                            _ => continue, // Ignorar stats desconocidos
                        };

                        stages.apply_change(&stat_change.stat, stat_change.change);
                        let new_stage = match stat_change.stat.as_str() {
                            "attack" => stages.attack,
                            "defense" => stages.defense,
                            "special_attack" => stages.special_attack,
                            "special_defense" => stages.special_defense,
                            "speed" => stages.speed,
                            "accuracy" => stages.accuracy,
                            "evasion" => stages.evasion,
                            _ => continue,
                        };

                        // Generar mensaje de log
                        let stat_name = match stat_change.stat.as_str() {
                            "attack" => "ataque",
                            "defense" => "defensa",
                            "special_attack" => "ataque especial",
                            "special_defense" => "defensa especial",
                            "speed" => "velocidad",
                            "accuracy" => "precisión",
                            "evasion" => "evasión",
                            _ => continue,
                        };

                        if new_stage > old_stage {
                            logs.push(format!(
                                "¡El {} de {} subió!",
                                stat_name,
                                defender_name
                            ));
                        } else if new_stage < old_stage {
                            logs.push(format!(
                                "¡El {} de {} bajó!",
                                stat_name,
                                defender_name
                            ));
                        }
                    }
                }
            }
        }
    }

    // Aplicar estados alterados
    if move_data.meta.ailment != "none" && move_data.meta.ailment_chance > 0 {
        let roll = rng.gen_range(0..=100);
        if roll <= move_data.meta.ailment_chance as u32 {
            // Verificar si el defensor ya tiene un estado
            if defender.status_condition.is_none() {
                // Verificar inmunidades por tipo
                let is_immune = match move_data.meta.ailment.as_str() {
                    "burn" => {
                        // Fuego no se quema
                        defender.randomized_profile.rolled_primary_type == PokemonType::Fire
                            || defender.randomized_profile.rolled_secondary_type == Some(PokemonType::Fire)
                    }
                    "paralysis" => {
                        // Eléctrico no se paraliza
                        defender.randomized_profile.rolled_primary_type == PokemonType::Electric
                            || defender.randomized_profile.rolled_secondary_type == Some(PokemonType::Electric)
                    }
                    "poison" => {
                        // Veneno y Acero no se envenenan
                        defender.randomized_profile.rolled_primary_type == PokemonType::Poison
                            || defender.randomized_profile.rolled_primary_type == PokemonType::Steel
                            || defender.randomized_profile.rolled_secondary_type == Some(PokemonType::Poison)
                            || defender.randomized_profile.rolled_secondary_type == Some(PokemonType::Steel)
                    }
                    _ => false,
                };

                if !is_immune {
                    // Aplicar el estado alterado
                    let status = match move_data.meta.ailment.as_str() {
                        "burn" => Some(StatusCondition::Burn),
                        "paralysis" => Some(StatusCondition::Paralysis),
                        "poison" => Some(StatusCondition::Poison),
                        "bad-poison" | "tox" => Some(StatusCondition::BadPoison),
                        "sleep" => Some(StatusCondition::Sleep),
                        "freeze" => Some(StatusCondition::Freeze),
                        _ => None,
                    };

                    if let Some(status_condition) = status {
                        defender.status_condition = Some(status_condition);
                        let status_msg = match status_condition {
                            StatusCondition::Burn => "quemó",
                            StatusCondition::Paralysis => "paralizó",
                            StatusCondition::Poison => "envenenó",
                            StatusCondition::BadPoison => "envenenó gravemente",
                            StatusCondition::Sleep => "durmió",
                            StatusCondition::Freeze => "congeló",
                        };
                        logs.push(format!(
                            "¡{} se ha {}!",
                            defender.species.display_name,
                            status_msg
                        ));
                    }
                }
            }
        }
    }
}

/// Aplica efectos residuales de condiciones de estado (quemadura, veneno, etc.)
/// Retorna (daño_recibido, logs)
pub fn apply_residual_effects(pokemon: &mut PokemonInstance) -> (u16, Vec<String>) {
    let mut logs = Vec::new();
    let mut total_damage = 0u16;
    
    if let Some(status) = &pokemon.status_condition {
        match status {
            StatusCondition::Burn => {
                let max_hp = pokemon.base_computed_stats.hp;
                let damage = max_hp / 16;
                let actual_damage = damage.min(pokemon.current_hp);
                pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
                total_damage = actual_damage;
                logs.push(format!(
                    "¡{} se lastima por la quemadura!",
                    pokemon.species.display_name
                ));
            }
            StatusCondition::Poison => {
                let max_hp = pokemon.base_computed_stats.hp;
                let damage = max_hp / 8;
                let actual_damage = damage.min(pokemon.current_hp);
                pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
                total_damage = actual_damage;
                logs.push(format!(
                    "¡{} sufre por el veneno!",
                    pokemon.species.display_name
                ));
            }
            StatusCondition::BadPoison => {
                // Toxic: aumenta el daño cada turno (simplificado: daño fijo alto)
                let max_hp = pokemon.base_computed_stats.hp;
                let damage = max_hp / 4;
                let actual_damage = damage.min(pokemon.current_hp);
                pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
                total_damage = actual_damage;
                logs.push(format!(
                    "¡{} sufre gravemente por el veneno!",
                    pokemon.species.display_name
                ));
            }
            _ => {
                // Freeze, Sleep, Paralysis no causan daño residual
            }
        }
    }
    
    (total_damage, logs)
}

/// Verifica si un Pokémon puede moverse basándose en su condición de estado
/// Retorna (puede_moverse, logs)
fn can_pokemon_move(pokemon: &mut PokemonInstance, rng: &mut StdRng) -> (bool, Vec<String>) {
    let mut logs = Vec::new();
    
    if let Some(status) = &pokemon.status_condition {
        match status {
            StatusCondition::Sleep => {
                // 33% de probabilidad de despertar
                if rng.gen_bool(0.33) {
                    pokemon.status_condition = None;
                    logs.push(format!(
                        "¡{} se despertó!",
                        pokemon.species.display_name
                    ));
                    return (true, logs);
                } else {
                    logs.push(format!(
                        "¡{} está dormido!",
                        pokemon.species.display_name
                    ));
                    return (false, logs);
                }
            }
            StatusCondition::Freeze => {
                // 20% de probabilidad de descongelarse
                if rng.gen_bool(0.2) {
                    pokemon.status_condition = None;
                    logs.push(format!(
                        "¡{} se descongeló!",
                        pokemon.species.display_name
                    ));
                    return (true, logs);
                } else {
                    logs.push(format!(
                        "¡{} está congelado!",
                        pokemon.species.display_name
                    ));
                    return (false, logs);
                }
            }
            StatusCondition::Paralysis => {
                // 25% de probabilidad de NO moverse
                if rng.gen_bool(0.25) {
                    logs.push(format!(
                        "¡{} está paralizado y no se puede mover!",
                        pokemon.species.display_name
                    ));
                    return (false, logs);
                }
                // Si pasa el 75%, puede moverse (no se añade log)
            }
            _ => {
                // Burn, Poison, BadPoison no impiden el movimiento
                // Retornar true sin logs adicionales
            }
        }
    }
    
    (true, logs)
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

    // PASO 1: Determinar quién ataca primero basándose en la velocidad efectiva
    // Considera stages de velocidad y condiciones de estado (parálisis)
    // También considera la prioridad del movimiento
    let player_effective_speed = get_effective_speed(player_mon);
    let enemy_effective_speed = get_effective_speed(enemy_mon);
    
    // Comparar prioridad primero (mayor prioridad ataca primero)
    let player_goes_first = if player_move.priority > enemy_move.priority {
        true
    } else if enemy_move.priority > player_move.priority {
        false
    } else {
        // Misma prioridad: comparar velocidad efectiva
        if player_effective_speed > enemy_effective_speed {
            true
        } else if enemy_effective_speed > player_effective_speed {
            false
        } else {
            // Empate: decidir al azar
            rng.gen_bool(0.5)
        }
    };

    // PASO 2: Ejecutar ataques en orden
    // Helper interno para ejecutar el ataque de un Pokémon
    // Retorna true si el objetivo se debilitó
    let execute_attack = |attacker: &mut PokemonInstance,
                          defender: &mut PokemonInstance,
                          move_data: &MoveData,
                          attacker_name: &str,
                          defender_name: &str,
                          logs: &mut Vec<String>,
                          rng: &mut StdRng| -> bool {
        // Check previo: ¿Puede moverse?
        let (can_move, status_logs) = can_pokemon_move(attacker, rng);
        logs.extend(status_logs);

        if !can_move {
            // No puede moverse, solo añadir logs y continuar
            return false;
        }

        // Puede moverse: ejecutar el ataque
        logs.push(format!("{} usó {}", attacker_name, move_data.name));

        // Verificar precisión
        let move_hits = if let Some(accuracy) = move_data.accuracy {
            let roll = rng.gen_range(0..=100);
            roll <= accuracy as u32
        } else {
            true
        };

        if !move_hits {
            logs.push("¡Pero falló!".to_string());
            return false;
        }

        // Calcular y aplicar daño
        let (damage, effectiveness_msg) = calculate_damage(attacker, defender, move_data, rng);
        
        if !effectiveness_msg.is_empty() {
            logs.push(effectiveness_msg);
        }

        // Aplicar daño
        if damage >= defender.current_hp {
            defender.current_hp = 0;
            logs.push(format!("{} recibió {} de daño", defender_name, damage));
            logs.push(format!("{} se debilitó", defender_name));
            return true; // Objetivo debilitado
        } else {
            defender.current_hp -= damage;
            logs.push(format!("{} recibió {} de daño", defender_name, damage));
        }

        // Aplicar efectos secundarios del movimiento (stats y estados)
        apply_move_secondary_effects(
            move_data,
            attacker,
            defender,
            logs,
            rng,
        );

        false // Objetivo sigue vivo
    };

    // PASO 2: Ejecutar ataques en orden
    // Guardar nombres antes de las llamadas para evitar problemas de borrow
    let player_name = player_mon.species.display_name.clone();
    let enemy_name = enemy_mon.species.display_name.clone();
    
    if player_goes_first {
        // Jugador ataca primero
        let enemy_dead = execute_attack(
            player_mon,
            enemy_mon,
            player_move,
            &player_name,
            &enemy_name,
            &mut result.logs,
            rng,
        );
        
        if enemy_dead {
            // Actualizar el HP del oponente en el battle_state ANTES de verificar reservas
            *battle_state.get_opponent_active_mut() = enemy_mon.clone();
            result.outcome = determine_enemy_outcome(battle_state);
            return result;
        }

        // Segundo ataque - Solo si el enemigo sigue vivo
        let enemy_name_for_log = format!("El enemigo {}", enemy_name);
        let player_dead = execute_attack(
            enemy_mon,
            player_mon,
            enemy_move,
            &enemy_name_for_log,
            &player_name,
            &mut result.logs,
            rng,
        );

        if player_dead {
            result.outcome = determine_player_outcome(player_team, player_active_index);
            return result;
        }
    } else {
        // Enemigo ataca primero
        let enemy_name_for_log = format!("El enemigo {}", enemy_name);
        let player_dead = execute_attack(
            enemy_mon,
            player_mon,
            enemy_move,
            &enemy_name_for_log,
            &player_name,
            &mut result.logs,
            rng,
        );

        if player_dead {
            result.outcome = determine_player_outcome(player_team, player_active_index);
            return result;
        }

        // Segundo ataque - Solo si el jugador sigue vivo
        let enemy_dead = execute_attack(
            player_mon,
            enemy_mon,
            player_move,
            &player_name,
            &enemy_name,
            &mut result.logs,
            rng,
        );

        if enemy_dead {
            // Actualizar el HP del oponente en el battle_state ANTES de verificar reservas
            *battle_state.get_opponent_active_mut() = enemy_mon.clone();
            result.outcome = determine_enemy_outcome(battle_state);
            return result;
        }
    }

    // BLOQUE FINAL: Efectos residuales post-turno
    // Solo si la batalla no ha terminado después de que ambos intentaran actuar
    
    // Aplicar efectos residuales al jugador
    let (_, player_residual_logs) = apply_residual_effects(player_mon);
    result.logs.extend(player_residual_logs);
    
    // Verificar si el jugador murió por daño residual
    if player_mon.current_hp == 0 {
        result.logs.push(format!(
            "{} se debilitó",
            player_mon.species.display_name
        ));
        result.outcome = determine_player_outcome(player_team, player_active_index);
        return result;
    }

    // Aplicar efectos residuales al enemigo
    let (_, enemy_residual_logs) = apply_residual_effects(enemy_mon);
    result.logs.extend(enemy_residual_logs);
    
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
    rng: &mut StdRng,
) -> TurnResult {
    let mut result = TurnResult::new();

    // Verificar si el enemigo puede moverse (condiciones de estado)
    let (enemy_can_move, enemy_status_logs) = can_pokemon_move(enemy_mon, rng);
    result.logs.extend(enemy_status_logs);

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

        // Aplicar efectos secundarios del movimiento (stats y estados)
        apply_move_secondary_effects(
            enemy_move,
            enemy_mon,
            player_mon,
            &mut result.logs,
            rng,
        );
    } else {
        result.logs.push("¡Pero falló!".to_string());
    }

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

