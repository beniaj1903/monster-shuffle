use rand::rngs::StdRng;
use rand::Rng;
use crate::models::{MoveData, PokemonInstance, StatusCondition, MoveMeta};

/// Verifica si un estado alterado se aplica exitosamente
/// Si chance es 0:
///   - Para ataques de estado puro (sin daño): asume 100%
///   - Para efectos secundarios: asume 0%
/// En la práctica, si chance es 0 y el movimiento tiene poder, asumimos 0%
/// Si chance es 0 y el movimiento NO tiene poder, asumimos 100%
pub fn check_ailment_success(chance: u8, move_has_power: bool, rng: &mut StdRng) -> bool {
    // Si chance es 0, determinar según el contexto
    if chance == 0 {
        // Si el movimiento tiene poder (ataque de daño), chance 0 = 0%
        // Si el movimiento NO tiene poder (ataque de estado puro), chance 0 = 100%
        return !move_has_power;
    }

    // Si chance > 0, generar un número 0-100 y comparar
    let roll = rng.gen_range(0..=100);
    roll <= chance as u32
}

/// Inicializa el PP de un movimiento si no está inicializado (current_pp == 0)
/// Busca el movimiento en el Pokémon y lo inicializa con el PP máximo del MoveData
/// Si el movimiento ya tiene max_pp > 0, no hace nada (ya está inicializado)
pub fn initialize_move_pp(pokemon: &mut PokemonInstance, move_template_id: &str, move_data: &MoveData) {
    // Buscar el movimiento aprendido en la lista del Pokémon
    if let Some(learned_move) = pokemon.get_learned_move(move_template_id) {
        // Si el PP no está inicializado (max_pp == 0), inicializarlo con el PP máximo del MoveData
        if learned_move.max_pp == 0 {
            learned_move.max_pp = move_data.pp;
            learned_move.current_pp = move_data.pp;
        }
    }
}

/// Decrementa el PP de un movimiento después de usarlo
/// Retorna true si el movimiento se usó exitosamente (tenía PP > 0)
pub fn consume_move_pp(pokemon: &mut PokemonInstance, move_template_id: &str) -> bool {
    // Buscar el movimiento aprendido en la lista del Pokémon
    if let Some(learned_move) = pokemon.get_learned_move(move_template_id) {
        // Verificar que tenga PP disponible
        if learned_move.current_pp > 0 {
            learned_move.current_pp -= 1;
            return true;
        } else {
            return false; // Sin PP disponible
        }
    }
    false // Movimiento no encontrado
}

/// Verifica si un Pokémon tiene movimientos con PP disponible
/// Retorna true si tiene al menos un movimiento con PP > 0
pub fn has_moves_with_pp(pokemon: &PokemonInstance) -> bool {
    let active_moves = pokemon.get_active_learned_moves();
    active_moves.iter().any(|m| m.current_pp > 0)
}

/// Crea un MoveData para Struggle (movimiento de emergencia)
/// Struggle: Tipo Normal, Potencia 50, PP infinito (no se consume), daño de retroceso 1/4 HP
pub fn create_struggle_move() -> MoveData {
    MoveData {
        id: "struggle".to_string(),
        name: "Forcejeo".to_string(),
        r#type: "Normal".to_string(),
        power: Some(50),
        accuracy: None, // Struggle siempre acierta
        priority: 0,
        pp: 255, // PP infinito (no se consume realmente)
        damage_class: "physical".to_string(),
        meta: MoveMeta {
            ailment: "none".to_string(),
            ailment_chance: 0,
            crit_rate: 0,
            drain: -25, // Recoil: 25% del daño (pero Struggle usa 1/4 del HP máximo)
            flinch_chance: 0,
            stat_chance: 0,
            healing: 0,
            min_hits: None,
            max_hits: None,
            min_turns: None,
            max_turns: None,
            makes_contact: true, // Struggle hace contacto
            forces_switch: false,
        },
        stat_changes: Vec::new(),
        target: "selected-pokemon".to_string(),
    }
}

/// Verifica si un Pokémon puede moverse basándose en su condición de estado
/// Retorna (puede_moverse, logs)
pub(crate) fn can_pokemon_move(pokemon: &mut PokemonInstance, rng: &mut StdRng) -> (bool, Vec<String>) {
    let mut logs = Vec::new();

    // Primero verificar status conditions permanentes
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

    // Luego verificar confusion (volatile status)
    if let Some(ref mut volatile) = pokemon.volatile_status {
        if volatile.confused {
            logs.push(format!("¡{} está confundido!", pokemon.species.display_name));

            // 50% de probabilidad de golpearse a sí mismo
            if rng.gen_bool(0.5) {
                // Aplicar daño por confusión (40 de power, físico, sin STAB)
                let confusion_damage = calculate_confusion_damage(pokemon);
                pokemon.current_hp = pokemon.current_hp.saturating_sub(confusion_damage);

                logs.push(format!(
                    "¡{} se golpeó a sí mismo en confusión y perdió {} HP!",
                    pokemon.species.display_name,
                    confusion_damage
                ));
                return (false, logs);
            } else {
                logs.push(format!("¡{} superó la confusión este turno!", pokemon.species.display_name));
            }
        }

        // Verificar infatuation (Attract)
        if volatile.infatuated_by.is_some() {
            logs.push(format!("¡{} está enamorado!", pokemon.species.display_name));

            // 50% de probabilidad de no poder atacar
            if rng.gen_bool(0.5) {
                logs.push(format!(
                    "¡{} está inmobilizado por el amor!",
                    pokemon.species.display_name
                ));
                return (false, logs);
            } else {
                logs.push(format!("¡{} superó la infatuación este turno!", pokemon.species.display_name));
            }
        }
    }

    (true, logs)
}

/// Calcula el daño que un Pokémon se inflige a sí mismo por confusión
/// Usa power 40, tipo físico, sin STAB ni efectividad
fn calculate_confusion_damage(pokemon: &PokemonInstance) -> u16 {
    let level = pokemon.level;
    let attack = pokemon.base_computed_stats.attack;
    let defense = pokemon.base_computed_stats.defense;
    let power = 40;

    // Fórmula simplificada: ((2 * Level / 5 + 2) * Power * A / D) / 50 + 2
    let base_damage = ((2 * level as u32 / 5 + 2) * power * attack as u32 / defense as u32) / 50 + 2;

    // Aplicar stages si existen
    let mut final_damage = base_damage as f32;
    if let Some(stages) = &pokemon.battle_stages {
        let attack_multiplier = get_stat_multiplier(stages.attack);
        let defense_multiplier = get_stat_multiplier(stages.defense);
        final_damage *= attack_multiplier / defense_multiplier;
    }

    final_damage as u16
}

/// Obtiene el multiplicador de stat basado en el stage (-6 a +6)
fn get_stat_multiplier(stage: i8) -> f32 {
    if stage >= 0 {
        (2.0 + stage as f32) / 2.0
    } else {
        2.0 / (2.0 - stage as f32)
    }
}

