use crate::models::{MoveData, PokemonInstance, PokemonType, WeatherState, TerrainState, WeatherType, TerrainType};
use super::super::damage_system::calculator::parse_type;

/// Hook: Habilidades que se activan al entrar en batalla
/// Se llama cuando un Pokémon entra en combate (inicio de batalla o switch)
/// Retorna el weather y terrain que deberían establecerse (si alguno)
pub fn trigger_on_entry_abilities(
    pokemon: &PokemonInstance,
    opponent: &mut PokemonInstance,
    logs: &mut Vec<String>,
) -> (Option<WeatherState>, Option<TerrainState>) {
    match pokemon.ability.as_str() {
        "intimidate" => {
            // Inicializar battle_stages si no existen
            if opponent.battle_stages.is_none() {
                opponent.init_battle_stages();
            }
            
            if let Some(ref mut stages) = opponent.battle_stages {
                // Bajar el ataque del oponente 1 nivel
                stages.attack = (stages.attack - 1).clamp(-6, 6);
                logs.push(format!(
                    "¡La Intimidación de {} bajó el ataque de {}!",
                    pokemon.species.display_name,
                    opponent.species.display_name
                ));
            }
            (None, None)
        }
        "drought" => {
            // Drought: Activa Sol intenso (5 turnos)
            logs.push(format!(
                "¡La Sequía de {} intensificó los rayos del sol!",
                pokemon.species.display_name
            ));
            (Some(WeatherState::new(WeatherType::Sun)), None)
        }
        "drizzle" => {
            // Drizzle: Activa Lluvia (5 turnos)
            logs.push(format!(
                "¡La Llovizna de {} provocó un diluvio!",
                pokemon.species.display_name
            ));
            (Some(WeatherState::new(WeatherType::Rain)), None)
        }
        "sand-stream" => {
            // Sand Stream: Activa Tormenta de arena (5 turnos)
            logs.push(format!(
                "¡La Corriente de Arena de {} desató una tormenta de arena!",
                pokemon.species.display_name
            ));
            (Some(WeatherState::new(WeatherType::Sandstorm)), None)
        }
        "snow-warning" => {
            // Snow Warning: Activa Granizo (5 turnos)
            logs.push(format!(
                "¡La Alerta de Nieve de {} provocó una granizada!",
                pokemon.species.display_name
            ));
            (Some(WeatherState::new(WeatherType::Hail)), None)
        }
        "electric-surge" => {
            // Electric Surge: Activa Campo Eléctrico (5 turnos)
            logs.push(format!(
                "¡Una corriente eléctrica recorre el campo!",
            ));
            (None, Some(TerrainState::new(TerrainType::Electric)))
        }
        "grassy-surge" => {
            // Grassy Surge: Activa Campo de Hierba (5 turnos)
            logs.push(format!(
                "¡Hierba crece en el combate!",
            ));
            (None, Some(TerrainState::new(TerrainType::Grassy)))
        }
        "misty-surge" => {
            // Misty Surge: Activa Campo de Niebla (5 turnos)
            logs.push(format!(
                "¡Una niebla mística envuelve el campo!",
            ));
            (None, Some(TerrainState::new(TerrainType::Misty)))
        }
        "psychic-surge" => {
            // Psychic Surge: Activa Campo Psíquico (5 turnos)
            logs.push(format!(
                "¡El campo se llena de energía psíquica!",
            ));
            (None, Some(TerrainState::new(TerrainType::Psychic)))
        }
        _ => {
            // Otras habilidades de entrada se pueden añadir aquí
            (None, None)
        }
    }
}

/// Hook: Verifica inmunidades basadas en habilidades
/// Retorna true si el defensor es inmune al movimiento
pub fn check_ability_immunity(
    defender: &PokemonInstance,
    move_data: &MoveData,
    logs: &mut Vec<String>,
) -> bool {
    let move_type = parse_type(&move_data.r#type);
    
    match defender.ability.as_str() {
        "levitate" => {
            if move_type == PokemonType::Ground {
                logs.push(format!(
                    "¡{} levitó sobre el ataque!",
                    defender.species.display_name
                ));
                return true;
            }
        }
        _ => {
            // Otras inmunidades se pueden añadir aquí
        }
    }
    
    false
}

/// Determina si un Pokémon está en contacto con el suelo (grounded)
/// Retorna `false` si el Pokémon no toca el suelo, `true` si está grounded
/// 
/// Un Pokémon NO está grounded (no toca el suelo) si:
/// - Es de tipo Flying (primary o secondary)
/// - Tiene la habilidad "levitate"
/// - Tiene el item "air-balloon"
/// - (Opcional) Tiene estados volátiles que lo levantan (magnet-rise, telekinesis)
pub fn is_grounded(pokemon: &PokemonInstance) -> bool {
    // Verificar si es tipo Flying
    if pokemon.randomized_profile.rolled_primary_type == PokemonType::Flying {
        return false;
    }
    
    if let Some(secondary_type) = pokemon.randomized_profile.rolled_secondary_type {
        if secondary_type == PokemonType::Flying {
            return false;
        }
    }
    
    // Verificar habilidad Levitate
    if pokemon.ability.as_str() == "levitate" {
        return false;
    }
    
    // Verificar item Air Balloon
    if let Some(ref item) = pokemon.held_item {
        if item.as_str() == "air-balloon" {
            return false;
        }
    }
    
    // Nota: Los estados volátiles magnet-rise y telekinesis no están implementados
    // en VolatileStatus actualmente, pero se pueden añadir en el futuro:
    // if let Some(ref volatile) = pokemon.volatile_status {
    //     if volatile.magnet_rise || volatile.telekinesis {
    //         return false;
    //     }
    // }
    
    // Si no cumple ninguna de las condiciones anteriores, está grounded
    true
}

/// Hook: Modifica el stat ofensivo basado en habilidades
/// Retorna un multiplicador para el stat de ataque/especial
pub fn modify_offensive_stat_by_ability(
    attacker: &PokemonInstance,
    move_data: &MoveData,
    _attack_stat_name: &str,
    logs: &mut Vec<String>,
) -> f32 {
    let move_type = parse_type(&move_data.r#type);
    let max_hp = attacker.base_computed_stats.hp;
    let current_hp = attacker.current_hp;
    let hp_threshold = max_hp / 3;
    
    match attacker.ability.as_str() {
        "blaze" => {
            // Blaze: Aumenta el poder de movimientos de tipo Fuego en 50% cuando HP < 33%
            if current_hp < hp_threshold && move_type == PokemonType::Fire {
                logs.push(format!(
                    "¡Arde la habilidad de {}!",
                    attacker.species.display_name
                ));
                return 1.5;
            }
        }
        "overgrow" => {
            // Overgrow: Aumenta el poder de movimientos de tipo Planta en 50% cuando HP < 33%
            if current_hp < hp_threshold && move_type == PokemonType::Grass {
                logs.push(format!(
                    "¡La habilidad de {} se intensifica!",
                    attacker.species.display_name
                ));
                return 1.5;
            }
        }
        "torrent" => {
            // Torrent: Aumenta el poder de movimientos de tipo Agua en 50% cuando HP < 33%
            if current_hp < hp_threshold && move_type == PokemonType::Water {
                logs.push(format!(
                    "¡La habilidad de {} se intensifica!",
                    attacker.species.display_name
                ));
                return 1.5;
            }
        }
        _ => {
            // Otras habilidades que modifican stats se pueden añadir aquí
        }
    }
    
    1.0 // Sin modificación
}

/// Aplica efectos residuales del clima a un Pokémon
/// Retorna (daño total, logs)
pub fn apply_weather_residuals(pokemon: &mut PokemonInstance, weather: Option<&WeatherState>) -> (u16, Vec<String>) {
    let mut logs = Vec::new();
    let mut total_damage = 0u16;
    
    let Some(weather_state) = weather else {
        return (0, logs);
    };
    
    // Aplicar daño residual según el tipo de clima
    match weather_state.weather_type {
        WeatherType::Sandstorm => {
            // Daña 1/16 HP a todos EXCEPTO tipos Rock, Ground, Steel
            let is_immune = pokemon.randomized_profile.rolled_primary_type == PokemonType::Rock
                || pokemon.randomized_profile.rolled_primary_type == PokemonType::Ground
                || pokemon.randomized_profile.rolled_primary_type == PokemonType::Steel
                || pokemon.randomized_profile.rolled_secondary_type == Some(PokemonType::Rock)
                || pokemon.randomized_profile.rolled_secondary_type == Some(PokemonType::Ground)
                || pokemon.randomized_profile.rolled_secondary_type == Some(PokemonType::Steel);
            
            if !is_immune {
                let max_hp = pokemon.base_computed_stats.hp;
                let damage = max_hp / 16;
                let actual_damage = damage.min(pokemon.current_hp);
                pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
                total_damage = actual_damage;
                logs.push(format!(
                    "¡La tormenta de arena golpea a {}!",
                    pokemon.species.display_name
                ));
            }
        }
        WeatherType::Hail => {
            // Daña 1/16 HP a todos EXCEPTO tipos Ice
            let is_immune = pokemon.randomized_profile.rolled_primary_type == PokemonType::Ice
                || pokemon.randomized_profile.rolled_secondary_type == Some(PokemonType::Ice);
            
            if !is_immune {
                let max_hp = pokemon.base_computed_stats.hp;
                let damage = max_hp / 16;
                let actual_damage = damage.min(pokemon.current_hp);
                pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
                total_damage = actual_damage;
                logs.push(format!(
                    "¡El granizo golpea a {}!",
                    pokemon.species.display_name
                ));
            }
        }
        _ => {
            // Sun y Rain no causan daño residual
        }
    }
    
    (total_damage, logs)
}

/// Aplica efectos residuales de condiciones de estado (quemadura, veneno, etc.)
/// Retorna (daño_recibido, logs)
pub fn apply_residual_effects(pokemon: &mut PokemonInstance) -> (u16, Vec<String>) {
    use crate::models::StatusCondition;
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
                // Toxic: aumenta el daño cada turno (1/16, 2/16, 3/16, etc.)
                let max_hp = pokemon.base_computed_stats.hp;

                // Incrementar contador de turnos con BadPoison
                if let Some(ref mut volatile) = pokemon.volatile_status {
                    volatile.badly_poisoned_turns += 1;
                    let turns = volatile.badly_poisoned_turns as u16;

                    // Daño escala: (max_hp / 16) * turns
                    let damage = (max_hp / 16) * turns;
                    let actual_damage = damage.min(pokemon.current_hp);
                    pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
                    total_damage = actual_damage;

                    logs.push(format!(
                        "¡{} sufre gravemente por el veneno! (turno {})",
                        pokemon.species.display_name,
                        turns
                    ));
                } else {
                    // Fallback si no tiene volatile_status (no debería pasar)
                    let damage = max_hp / 8;
                    let actual_damage = damage.min(pokemon.current_hp);
                    pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
                    total_damage = actual_damage;
                    logs.push(format!(
                        "¡{} sufre gravemente por el veneno!",
                        pokemon.species.display_name
                    ));
                }
            }
            _ => {
                // Freeze, Sleep, Paralysis no causan daño residual
            }
        }
    }
    
    (total_damage, logs)
}

