//! Reseteo de estados de turno
//!
//! Este módulo maneja el reseteo de flags y estados volátiles que duran solo un turno.

use crate::models::PokemonInstance;
use crate::game::{BattleState, PlayerTeam};

/// Resetea flags de turno para todos los Pokémon activos
///
/// Resetea estados volátiles que solo duran un turno, como Protect.
///
/// # Argumentos
/// * `player_team` - Equipo del jugador
/// * `opponent_team` - Equipo del oponente
/// * `battle_state` - Estado actual de la batalla
pub fn reset_turn_flags(
    player_team: &mut PlayerTeam,
    opponent_team: &mut Vec<PokemonInstance>,
    battle_state: &BattleState,
) {
    // Resetear flags del jugador
    for &idx in &battle_state.player_active_indices {
        if let Some(pokemon) = player_team.active_members.get_mut(idx) {
            if let Some(ref mut volatile) = pokemon.volatile_status {
                // Resetear protected (Protect solo dura 1 turno)
                if volatile.protected {
                    volatile.protected = false;
                    // Resetear contador si no usó protect este turno
                    volatile.protect_counter = 0;
                }
            }
        }
    }

    // Resetear flags del oponente
    for &idx in &battle_state.opponent_active_indices {
        if let Some(pokemon) = opponent_team.get_mut(idx) {
            if let Some(ref mut volatile) = pokemon.volatile_status {
                if volatile.protected {
                    volatile.protected = false;
                    volatile.protect_counter = 0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        PokemonSpecies, RandomizedProfile, Stats, PokemonType, VolatileStatus, StatModifiers,
    };

    fn create_test_pokemon_with_protect() -> PokemonInstance {
        let mut volatile = VolatileStatus::new();
        volatile.protected = true;
        volatile.protect_counter = 1;

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
            volatile_status: Some(volatile),
            battle_stages: None,
            ability: "static".to_string(),
            held_item: None,
            individual_values: Stats::default(),
            effort_values: Stats::default(),
            randomized_profile: RandomizedProfile {
                rolled_primary_type: PokemonType::Electric,
                rolled_secondary_type: None,
                rolled_ability_id: "static".to_string(),
                stat_modifiers: StatModifiers::default(),
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

    #[test]
    fn test_reset_turn_flags_clears_protect() {
        let mut player_team = PlayerTeam {
            active_members: vec![create_test_pokemon_with_protect()],
            benched_members: Vec::new(),
        };
        let mut opponent_team = vec![create_test_pokemon_with_protect()];
        let battle_state = BattleState {
            player_active_indices: vec![0],
            opponent_active_indices: vec![0],
            is_trainer_battle: false,
            opponent_instance: create_test_pokemon_with_protect(),
            weather: None,
            terrain: None,
            turn_number: 1,
            battle_format: crate::models::BattleFormat::Singles,
        };

        reset_turn_flags(&mut player_team, &mut opponent_team, &battle_state);

        // Verificar que protected se resetea
        assert_eq!(
            player_team.active_members[0]
                .volatile_status
                .as_ref()
                .unwrap()
                .protected,
            false
        );
        assert_eq!(
            opponent_team[0]
                .volatile_status
                .as_ref()
                .unwrap()
                .protected,
            false
        );
    }
}
