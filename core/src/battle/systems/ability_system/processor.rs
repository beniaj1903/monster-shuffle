//! Procesador de habilidades
//!
//! Este módulo procesa los hooks de habilidades en diferentes momentos de la batalla.

use crate::models::{PokemonInstance, MoveData};
use crate::game::BattleState;
use super::{get_ability_hooks, AbilityTrigger, AbilityEffect};
use super::super::damage_system::get_effective_speed;

/// Calcula la velocidad de un Pokémon incluyendo modificadores de habilidades
///
/// Aplica multiplicadores basados en clima/terreno (Chlorophyll, Swift Swim, etc.)
pub fn get_speed_with_abilities(
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
///
/// Aplica modificadores de prioridad (Prankster, Gale Wings, Triage, etc.)
pub fn get_priority_with_abilities(
    pokemon: &PokemonInstance,
    move_data: &MoveData,
) -> i8 {
    use super::registry::PriorityCondition;
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

// NOTA: handle_entry_hazards, apply_on_entry_stat_change y apply_end_of_turn_abilities
// permanecen en pipeline.rs por ahora debido a su complejidad y dependencias.
// Se migrarán en una fase posterior si es necesario.
