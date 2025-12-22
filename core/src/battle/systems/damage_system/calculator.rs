use rand::rngs::StdRng;
use rand::Rng;
use crate::models::{MoveData, PokemonInstance, PokemonType, StatusCondition, WeatherState, TerrainState, WeatherType};
use super::super::super::effects::{is_grounded, check_ability_immunity, modify_offensive_stat_by_ability};
use super::super::ability_system::{get_ability_hooks, AbilityTrigger, AbilityEffect};

/// Calcula el daño de un movimiento
/// Retorna (daño, mensaje de efectividad, es_crítico)
pub fn calculate_damage(
    attacker: &PokemonInstance,
    defender: &PokemonInstance,
    move_data: &MoveData,
    is_critical: bool,
    rng: &mut StdRng,
    mut logs: Option<&mut Vec<String>>,
    weather: Option<&WeatherState>,
    terrain: Option<&TerrainState>,
) -> (u16, String, bool) {
    // Si el movimiento no tiene poder, retorna 0
    let Some(power) = move_data.power else {
        eprintln!("[DEBUG] calculate_damage: Movimiento sin poder, retornando 0");
        return (0, String::new(), false);
    };
    eprintln!("[DEBUG] calculate_damage: Poder base: {}", power);

    // Obtener el tipo del movimiento
    let move_type = parse_type(&move_data.r#type);

    // Hook: Verificar inmunidades basadas en habilidades (antes de calcular daño)
    if let Some(ref mut log_vec) = logs {
        if check_ability_immunity(defender, move_data, log_vec) {
            // Si es inmune, retornar 0 de daño
            return (0, String::new(), false);
        }
    } else {
        // Si no hay logs disponibles, usar logs temporales
        let mut temp_logs = Vec::new();
        if check_ability_immunity(defender, move_data, &mut temp_logs) {
            return (0, String::new(), false);
        }
    }

    // Calcular efectividad de tipo
    let mut defender_types = vec![defender.randomized_profile.rolled_primary_type];
    if let Some(secondary) = defender.randomized_profile.rolled_secondary_type {
        defender_types.push(secondary);
    }

    let type_effectiveness = get_type_effectiveness(&move_type, &defender_types);

    // Verificar STAB (Same Type Attack Bonus)
    // Considera la habilidad Adaptability para x2.0 en lugar de x1.5
    let stab_multiplier = get_stab_multiplier(attacker, &move_type);

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
        return (0, String::new(), false);
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

    // Hook: Modificar stat ofensivo basado en habilidades (Blaze, Overgrow, Torrent)
    let ability_multiplier = if let Some(ref mut log_vec) = logs {
        modify_offensive_stat_by_ability(
            attacker,
            move_data,
            attack_stat_name,
            log_vec,
        )
    } else {
        // Si no hay logs disponibles, usar logs temporales (se descartan)
        let mut temp_logs = Vec::new();
        modify_offensive_stat_by_ability(
            attacker,
            move_data,
            attack_stat_name,
            &mut temp_logs,
        )
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

    // Hook: Modificar stat defensivo basado en clima (Sandstorm para Rock, Hail para Ice)
    let weather_defense_multiplier = modify_defensive_stat_by_weather(weather, defender, defense_stat_name);

    // Calcular stats efectivos (aplicar multiplicador de habilidad al ataque y clima a la defensa)
    let attack = (base_attack_stat as f32) * attack_multiplier * ability_multiplier;
    let defense = (base_defense_stat as f32) * defense_multiplier * weather_defense_multiplier;

    // Fórmula de daño Gen 3+
    // Damage = ((((2 * Level / 5 + 2) * Power * A / D) / 50) + 2) * Modifiers
    let level = attacker.level as f32;
    let power = power as f32;

    let base_damage = ((2.0 * level / 5.0 + 2.0) * power * attack / defense) / 50.0 + 2.0;

    // Modificadores: STAB y efectividad de tipo
    let mut modifiers = stab_multiplier * type_effectiveness;

    // Hook: Aplicar multiplicador de clima (Sun/Rain afectan Fire/Water)
    let weather_damage_mod = apply_weather_damage_mod(weather, &move_data.r#type);
    modifiers *= weather_damage_mod;

    // Hook: Aplicar multiplicador de terreno
    let terrain_damage_mod = apply_terrain_damage_mod(terrain, attacker, defender, move_data, logs);
    modifiers *= terrain_damage_mod;

    // Hook: Aplicar multiplicadores de habilidades de daño (Tough Claws, Sheer Force, Technician, etc.)
    let ability_damage_mod = apply_ability_damage_multiplier(attacker, move_data, &move_type);
    modifiers *= ability_damage_mod;

    // Efecto de Quemadura (Burn): reduce el daño físico a la mitad
    if move_data.damage_class == "physical" && attacker.status_condition == Some(StatusCondition::Burn) {
        modifiers *= 0.5;
    }

    // Multiplicador de crítico (Gen 7+: x1.5)
    if is_critical {
        modifiers *= 1.5;
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

    (damage, effectiveness_msg, is_critical)
}

/// Convierte un string de tipo a PokemonType enum
pub fn parse_type(type_str: &str) -> PokemonType {
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
pub fn get_type_effectiveness(move_type: &PokemonType, defender_types: &[PokemonType]) -> f32 {
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
/// Compara el tipo del movimiento con los tipos del Pokémon (tanto species como randomized_profile)
fn has_stab(attacker: &PokemonInstance, move_type: &PokemonType) -> bool {
    // Verificar tipos de la especie base
    let species_match = attacker.species.primary_type == *move_type
        || attacker.species.secondary_type.as_ref()
            .map(|t| *t == *move_type)
            .unwrap_or(false);
    
    // Verificar tipos randomizados (si existen)
    let randomized_match = attacker.randomized_profile.rolled_primary_type == *move_type
        || attacker.randomized_profile.rolled_secondary_type.as_ref()
            .map(|t| *t == *move_type)
            .unwrap_or(false);
    
    species_match || randomized_match
}

/// Calcula el multiplicador STAB considerando la habilidad del atacante
/// - STAB normal: 1.5x
/// - Con Adaptability: 2.0x
fn get_stab_multiplier(attacker: &PokemonInstance, move_type: &PokemonType) -> f32 {
    if has_stab(attacker, move_type) {
        // Verificar si tiene la habilidad Adaptability
        if attacker.ability == "adaptability" {
            2.0
        } else {
            1.5
        }
    } else {
        1.0
    }
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
pub fn get_effective_speed(pokemon: &PokemonInstance) -> f32 {
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

/// Verifica si un ataque es crítico basado en el crit_rate del movimiento
/// Probabilidades según Gen 7+:
/// - Stage 0 (crit_rate 0): 1/24 (~4.17%)
/// - Stage 1 (crit_rate 1): 1/8 (~12.5%)
/// - Stage 2 (crit_rate 2): 1/2 (~50%)
/// - Stage 3+ (crit_rate 3+): 100%
pub fn check_critical_hit(move_crit_rate: u8, rng: &mut StdRng) -> bool {
    match move_crit_rate {
        0 => {
            // 1/24 = ~4.17%
            let roll = rng.gen_range(0..24);
            roll == 0
        }
        1 => {
            // 1/8 = 12.5%
            let roll = rng.gen_range(0..8);
            roll == 0
        }
        2 => {
            // 1/2 = 50%
            rng.gen_bool(0.5)
        }
        _ => {
            // Stage 3+: 100% crítico
            true
        }
    }
}

/// Calcula el número de golpes para movimientos multi-hit
/// Si min o max son None, retorna 1 (movimiento normal)
/// Si son iguales, retorna ese número
/// Para movimientos 2-5 golpes, usa probabilidades aproximadas:
/// - 2 golpes: 35%
/// - 3 golpes: 35%
/// - 4 golpes: 15%
/// - 5 golpes: 15%
pub fn calculate_hit_count(min_hits: Option<u8>, max_hits: Option<u8>, rng: &mut StdRng) -> u8 {
    // Si alguno es None, es un movimiento normal (1 golpe)
    let (min, max) = match (min_hits, max_hits) {
        (Some(min), Some(max)) => (min, max),
        _ => return 1,
    };

    // Si min y max son iguales, retorna ese número
    if min == max {
        return min;
    }

    // Validar rango (debe ser 2-5 para movimientos multi-hit típicos)
    if min < 2 || max > 5 || min > max {
        return 1; // Fallback a 1 golpe si el rango es inválido
    }

    // Para movimientos 2-5 golpes, usar probabilidades aproximadas
    let roll = rng.gen_range(0..100);
    
    match (min, max) {
        (2, 5) => {
            // Double Slap, Bullet Seed, etc.
            if roll < 35 {
                2
            } else if roll < 70 {
                3
            } else if roll < 85 {
                4
            } else {
                5
            }
        }
        (2, 3) => {
            // Movimientos 2-3 golpes
            if roll < 50 {
                2
            } else {
                3
            }
        }
        (2, 4) => {
            // Movimientos 2-4 golpes
            if roll < 40 {
                2
            } else if roll < 80 {
                3
            } else {
                4
            }
        }
        (3, 5) => {
            // Movimientos 3-5 golpes
            if roll < 40 {
                3
            } else if roll < 75 {
                4
            } else {
                5
            }
        }
        _ => {
            // Para otros rangos, usar distribución uniforme
            rng.gen_range(min..=max)
        }
    }
}

/// Aplica multiplicador de daño basado en el clima
/// Retorna el multiplicador (1.0 si no hay efecto)
fn apply_weather_damage_mod(weather: Option<&WeatherState>, move_type: &str) -> f32 {
    let Some(weather_state) = weather else {
        return 1.0;
    };
    
    match weather_state.weather_type {
        WeatherType::Sun => {
            if move_type == "Fire" {
                1.5 // Sol intensifica fuego
            } else if move_type == "Water" {
                0.5 // Sol debilita agua
            } else {
                1.0
            }
        }
        WeatherType::Rain => {
            if move_type == "Water" {
                1.5 // Lluvia intensifica agua
            } else if move_type == "Fire" {
                0.5 // Lluvia debilita fuego
            } else {
                1.0
            }
        }
        WeatherType::Sandstorm | WeatherType::Hail => {
            // Sin efecto en daño directo de ataques
            1.0
        }
        WeatherType::None => 1.0,
    }
}

/// Modifica el stat defensivo basado en el clima
/// Retorna el multiplicador (1.0 si no hay efecto)
fn modify_defensive_stat_by_weather(
    weather: Option<&WeatherState>,
    defender: &PokemonInstance,
    defense_stat_name: &str,
) -> f32 {
    let Some(weather_state) = weather else {
        return 1.0;
    };
    
    match weather_state.weather_type {
        WeatherType::Sandstorm => {
            // Sandstorm: Rock types get +50% Special Defense
            if defense_stat_name == "special_defense" {
                let is_rock = defender.randomized_profile.rolled_primary_type == PokemonType::Rock
                    || defender.randomized_profile.rolled_secondary_type == Some(PokemonType::Rock);
                if is_rock {
                    return 1.5;
                }
            }
        }
        WeatherType::Hail => {
            // Hail (Snow in Gen 9): Ice types get +50% Defense
            if defense_stat_name == "defense" {
                let is_ice = defender.randomized_profile.rolled_primary_type == PokemonType::Ice
                    || defender.randomized_profile.rolled_secondary_type == Some(PokemonType::Ice);
                if is_ice {
                    return 1.5;
                }
            }
        }
        _ => {}
    }
    
    1.0
}

/// Aplica multiplicador de daño basado en el terreno
/// Retorna el multiplicador (1.0 si no hay efecto)
fn apply_terrain_damage_mod(
    terrain: Option<&TerrainState>,
    attacker: &PokemonInstance,
    defender: &PokemonInstance,
    move_data: &MoveData,
    mut logs: Option<&mut Vec<String>>,
) -> f32 {
    use crate::models::TerrainType;
    let Some(terrain_state) = terrain else {
        return 1.0;
    };
    
    let move_type = parse_type(&move_data.r#type);
    let attacker_grounded = is_grounded(attacker);
    let defender_grounded = is_grounded(defender);
    
    match terrain_state.terrain_type {
        TerrainType::Electric => {
            // Electric Terrain: Atacante grounded + Move Electric -> x1.3
            if attacker_grounded && move_type == PokemonType::Electric {
                if let Some(ref mut log_vec) = logs {
                    log_vec.push(format!(
                        "¡El Campo Eléctrico potencia el ataque de {}!",
                        attacker.species.display_name
                    ));
                }
                return 1.3;
            }
        }
        TerrainType::Psychic => {
            // Psychic Terrain: Atacante grounded + Move Psychic -> x1.3
            if attacker_grounded && move_type == PokemonType::Psychic {
                if let Some(ref mut log_vec) = logs {
                    log_vec.push(format!(
                        "¡El Campo Psíquico potencia el ataque de {}!",
                        attacker.species.display_name
                    ));
                }
                return 1.3;
            }
        }
        TerrainType::Grassy => {
            // Grassy Terrain: Atacante grounded + Move Grass -> x1.3
            if attacker_grounded && move_type == PokemonType::Grass {
                if let Some(ref mut log_vec) = logs {
                    log_vec.push(format!(
                        "¡El Campo de Hierba potencia el ataque de {}!",
                        attacker.species.display_name
                    ));
                }
                return 1.3;
            }
            
            // Grassy Terrain: Movimientos de área Ground (earthquake, bulldoze, magnitude) -> x0.5
            let ground_area_moves = ["earthquake", "bulldoze", "magnitude"];
            if ground_area_moves.contains(&move_data.id.as_str()) {
                if let Some(ref mut log_vec) = logs {
                    log_vec.push(format!(
                        "¡El Campo de Hierba reduce el daño de {}!",
                        move_data.name
                    ));
                }
                return 0.5;
            }
        }
        TerrainType::Misty => {
            // Misty Terrain: Move Dragon -> x0.5 (Si el defensor toca el suelo)
            if defender_grounded && move_type == PokemonType::Dragon {
                if let Some(ref mut log_vec) = logs {
                    log_vec.push(format!(
                        "¡El Campo de Niebla reduce el daño de {}!",
                        move_data.name
                    ));
                }
                return 0.5;
            }
        }
    }

    1.0
}

/// Aplica multiplicadores de daño basados en habilidades del atacante
/// (Tough Claws, Sheer Force, Technician, Iron Fist, etc.)
fn apply_ability_damage_multiplier(
    attacker: &PokemonInstance,
    move_data: &MoveData,
    move_type: &PokemonType,
) -> f32 {
    let ability_id = &attacker.ability;
    let hooks = get_ability_hooks(ability_id);

    // Buscar modificadores de daño BeforeDamage
    for hook in hooks.iter().filter(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage)) {
        match &hook.effect {
            // Multiplica el daño de movimientos de contacto (Tough Claws)
            AbilityEffect::BoostContactMoves { multiplier } => {
                if move_data.meta.makes_contact {
                    return *multiplier;
                }
            },
            // Boost condicional por HP bajo (Blaze, Torrent, Overgrow)
            // Ya se maneja en modify_offensive_stat_by_ability, pero también lo aplicamos aquí
            AbilityEffect::BoostTypeAtLowHP { move_type: boosted_type, multiplier, hp_threshold } => {
                if move_type == boosted_type {
                    let hp_percentage = (attacker.current_hp as f32) / (attacker.base_computed_stats.hp as f32);
                    if hp_percentage <= *hp_threshold {
                        return *multiplier;
                    }
                }
            },
            _ => {},
        }
    }

    1.0 // Sin modificador
}

