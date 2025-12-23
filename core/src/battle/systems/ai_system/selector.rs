//! Selector de movimientos para la IA
//!
//! Este módulo contiene la lógica para seleccionar movimientos
//! para los oponentes controlados por IA.

use crate::models::PokemonInstance;
use super::super::validation_system::has_moves_with_pp;

/// Selecciona un movimiento para la IA del oponente
///
/// Implementa una estrategia simple: selecciona el primer movimiento
/// con PP disponible. Si no hay movimientos con PP, retorna "struggle".
///
/// # Argumentos
/// * `pokemon` - Pokémon controlado por IA
/// * `_logs` - Vector de logs (reservado para futuras estrategias que generen logs)
///
/// # Retorna
/// ID del movimiento seleccionado (String)
///
/// # TODO
/// - Implementar estrategias más inteligentes (elegir por tipo, poder, etc.)
/// - Considerar el tipo del oponente para elegir movimientos efectivos
/// - Implementar niveles de dificultad de IA
pub fn select_ai_move(pokemon: &PokemonInstance, _logs: &mut Vec<String>) -> String {
    // Verificar si tiene movimientos con PP
    if !has_moves_with_pp(pokemon) {
        return "struggle".to_string();
    }

    // Obtener movimientos con PP disponible
    let moves_with_pp: Vec<String> = pokemon
        .get_active_learned_moves()
        .iter()
        .filter(|m| m.current_pp > 0)
        .map(|m| m.move_id.clone())
        .collect();

    if moves_with_pp.is_empty() {
        "struggle".to_string()
    } else {
        // Por ahora: seleccionar el primer movimiento con PP
        // TODO: Implementar IA más inteligente
        moves_with_pp[0].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PokemonSpecies, RandomizedProfile, LearnedMove, Stats, PokemonType, StatModifiers};

    #[test]
    fn test_select_ai_move_returns_first_with_pp() {
        let mut pokemon = create_test_pokemon();

        // Agregar movimientos con PP
        pokemon.randomized_profile.learned_moves = vec![
            LearnedMove {
                move_id: "tackle".to_string(),
                max_pp: 35,
                current_pp: 35,
            },
            LearnedMove {
                move_id: "thunderbolt".to_string(),
                max_pp: 15,
                current_pp: 15,
            },
        ];

        let mut logs = Vec::new();
        let move_id = select_ai_move(&pokemon, &mut logs);

        assert_eq!(move_id, "tackle");
    }

    #[test]
    fn test_select_ai_move_skips_moves_without_pp() {
        let mut pokemon = create_test_pokemon();

        // Primer movimiento sin PP, segundo con PP
        pokemon.randomized_profile.learned_moves = vec![
            LearnedMove {
                move_id: "tackle".to_string(),
                max_pp: 35,
                current_pp: 0, // Sin PP
            },
            LearnedMove {
                move_id: "thunderbolt".to_string(),
                max_pp: 15,
                current_pp: 15,
            },
        ];

        let mut logs = Vec::new();
        let move_id = select_ai_move(&pokemon, &mut logs);

        assert_eq!(move_id, "thunderbolt");
    }

    #[test]
    fn test_select_ai_move_returns_struggle_when_no_pp() {
        let mut pokemon = create_test_pokemon();

        // Todos los movimientos sin PP
        pokemon.randomized_profile.learned_moves = vec![
            LearnedMove {
                move_id: "tackle".to_string(),
                max_pp: 35,
                current_pp: 0,
            },
        ];

        let mut logs = Vec::new();
        let move_id = select_ai_move(&pokemon, &mut logs);

        assert_eq!(move_id, "struggle");
    }

    fn create_test_pokemon() -> PokemonInstance {
        PokemonInstance {
            id: "test-0".to_string(),
            species: PokemonSpecies {
                species_id: "pikachu".to_string(),
                display_name: "Pikachu".to_string(),
                generation: 1,
                primary_type: PokemonType::Electric,
                secondary_type: None,
                base_stats: Stats {
                    hp: 35,
                    attack: 55,
                    defense: 40,
                    special_attack: 50,
                    special_defense: 50,
                    speed: 90,
                },
                move_pool: vec!["tackle".to_string()],
                possible_abilities: vec!["static".to_string()],
                is_starter_candidate: false,
                evolutions: Vec::new(),
            },
            level: 50,
            current_hp: 100,
            status_condition: None,
            volatile_status: None,
            battle_stages: None,
            ability: "static".to_string(),
            held_item: None,
            individual_values: Stats::default(),
            effort_values: Stats::default(),
            randomized_profile: RandomizedProfile {
                rolled_primary_type: PokemonType::Electric,
                rolled_secondary_type: None,
                rolled_ability_id: "static".to_string(),
                stat_modifiers: crate::models::StatModifiers::default(),
                learned_moves: Vec::new(),
                moves: Vec::new(),
            },
            base_computed_stats: Stats {
                hp: 100,
                attack: 70,
                defense: 60,
                special_attack: 65,
                special_defense: 65,
                speed: 105,
            },
        }
    }
}
