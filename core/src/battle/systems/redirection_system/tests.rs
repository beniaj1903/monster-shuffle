//! Tests comprehensivos para el Redirection System (Fase 4.2)
//!
//! Este módulo contiene tests exhaustivos para Follow Me, Rage Powder,
//! Spotlight y Ally Switch implementados en las Fases 1 y 2.

use crate::models::{
    PokemonInstance, PokemonSpecies, Stats, RandomizedProfile, PokemonType,
    StatModifiers, VolatileStatus, StatStages, MoveData, MoveMeta,
    BattleFormat, FieldPosition,
};
use crate::game::BattleState;
use super::processor::*;

/// Helper para crear Pokémon de prueba con tipo específico
fn create_test_pokemon(primary_type: PokemonType, secondary_type: Option<PokemonType>) -> PokemonInstance {
    PokemonInstance {
        id: "test-1".to_string(),
        species: PokemonSpecies {
            species_id: "pikachu".to_string(),
            display_name: "Pikachu".to_string(),
            generation: 1,
            primary_type: primary_type,
            secondary_type,
            base_stats: Stats {
                hp: 35,
                attack: 55,
                defense: 40,
                special_attack: 50,
                special_defense: 50,
                speed: 90,
            },
            move_pool: vec!["tackle".to_string(), "thunderbolt".to_string()],
            possible_abilities: vec!["static".to_string()],
            is_starter_candidate: false,
            evolutions: Vec::new(),
        },
        level: 50,
        current_hp: 100,
        status_condition: None,
        held_item: None,
        ability: "static".to_string(),
        battle_stages: Some(StatStages::new()),
        volatile_status: Some(VolatileStatus::new()),
        individual_values: Stats::default(),
        effort_values: Stats::default(),
        randomized_profile: RandomizedProfile {
            rolled_primary_type: primary_type,
            rolled_secondary_type: secondary_type,
            rolled_ability_id: "static".to_string(),
            stat_modifiers: StatModifiers::default(),
            learned_moves: Vec::new(),
            moves: Vec::new(),
        },
        base_computed_stats: Stats {
            hp: 200,
            attack: 100,
            defense: 100,
            special_attack: 100,
            special_defense: 100,
            speed: 100,
        },
    }
}

/// Helper para crear un movimiento single-target
fn create_single_target_move() -> MoveData {
    MoveData {
        id: "tackle".to_string(),
        name: "Tackle".to_string(),
        r#type: "Normal".to_string(),
        power: Some(40),
        accuracy: Some(100),
        priority: 0,
        pp: 35,
        damage_class: "physical".to_string(),
        meta: MoveMeta::default(),
        stat_changes: vec![],
        target: "selected-pokemon".to_string(),
    }
}

/// Helper para crear un movimiento spread
fn create_spread_move() -> MoveData {
    MoveData {
        id: "earthquake".to_string(),
        name: "Earthquake".to_string(),
        r#type: "Ground".to_string(),
        power: Some(100),
        accuracy: Some(100),
        priority: 0,
        pp: 10,
        damage_class: "physical".to_string(),
        meta: MoveMeta::default(),
        stat_changes: vec![],
        target: "all-opponents".to_string(),
    }
}

/// Helper para crear un BattleState básico para tests
fn create_double_battle_state() -> BattleState {
    BattleState::new(
        0,
        vec![
            create_test_pokemon(PokemonType::Normal, None),
            create_test_pokemon(PokemonType::Normal, None),
        ],
        "Opponent".to_string(),
        BattleFormat::Double,
        true,
    )
}

/// Helper para crear un BattleState de batalla single
fn create_single_battle_state() -> BattleState {
    BattleState::new(
        0,
        vec![create_test_pokemon(PokemonType::Normal, None)],
        "Opponent".to_string(),
        BattleFormat::Single,
        true,
    )
}

// ==================== FOLLOW ME TESTS ====================

#[cfg(test)]
mod follow_me {
    use super::*;

    #[test]
    fn test_follow_me_redirects_opponent_single_target() {
        let mut battle_state = create_double_battle_state();
        set_follow_me(&mut battle_state, FieldPosition::PlayerLeft);

        let move_data = create_single_target_move();
        let attacker = create_test_pokemon(PokemonType::Normal, None);

        // Ataque del oponente dirigido a PlayerRight debe redirigirse a PlayerLeft
        let result = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &attacker,
            &move_data,
            &battle_state,
        );

        assert_eq!(
            result,
            Some(FieldPosition::PlayerLeft),
            "Follow Me debe redirigir ataques del oponente a PlayerLeft"
        );
    }

    #[test]
    fn test_follow_me_does_not_redirect_spread_moves() {
        let mut battle_state = create_double_battle_state();
        set_follow_me(&mut battle_state, FieldPosition::PlayerLeft);

        let spread_move = create_spread_move();
        let attacker = create_test_pokemon(PokemonType::Normal, None);

        // Movimientos spread no deben ser redirigidos
        let result = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &attacker,
            &spread_move,
            &battle_state,
        );

        assert_eq!(
            result,
            None,
            "Follow Me NO debe redirigir movimientos spread"
        );
    }

    #[test]
    fn test_follow_me_does_not_redirect_ally_attacks() {
        let mut battle_state = create_double_battle_state();
        set_follow_me(&mut battle_state, FieldPosition::PlayerLeft);

        let move_data = create_single_target_move();
        let attacker = create_test_pokemon(PokemonType::Normal, None);

        // Ataque de aliado (PlayerRight) no debe ser redirigido (opponent_only = true)
        let result = apply_redirection(
            FieldPosition::OpponentLeft,
            FieldPosition::PlayerRight,
            &attacker,
            &move_data,
            &battle_state,
        );

        assert_eq!(
            result,
            None,
            "Follow Me NO debe redirigir ataques de aliados (opponent_only = true)"
        );
    }

    #[test]
    fn test_follow_me_does_not_redirect_if_already_targeting_redirector() {
        let mut battle_state = create_double_battle_state();
        set_follow_me(&mut battle_state, FieldPosition::PlayerLeft);

        let move_data = create_single_target_move();
        let attacker = create_test_pokemon(PokemonType::Normal, None);

        // Si el objetivo ya es el redirector, no cambiar
        let result = apply_redirection(
            FieldPosition::PlayerLeft,
            FieldPosition::OpponentLeft,
            &attacker,
            &move_data,
            &battle_state,
        );

        assert_eq!(
            result,
            None,
            "Follow Me NO debe redirigir si el objetivo ya es el redirector"
        );
    }

    #[test]
    fn test_follow_me_sets_redirection_state() {
        let mut battle_state = create_double_battle_state();
        set_follow_me(&mut battle_state, FieldPosition::PlayerLeft);

        assert!(battle_state.redirection.is_some(), "Debe crear RedirectionState");
        let redirection = battle_state.redirection.as_ref().unwrap();
        assert_eq!(redirection.redirector_position, FieldPosition::PlayerLeft);
        assert_eq!(redirection.redirection_type, "follow-me");
        assert!(redirection.opponent_only, "Follow Me solo afecta al oponente");
    }

    #[test]
    fn test_follow_me_works_from_any_position() {
        // Test que Follow Me funciona desde todas las posiciones posibles
        for position in [
            FieldPosition::PlayerLeft,
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            FieldPosition::OpponentRight,
        ] {
            let mut battle_state = create_double_battle_state();
            set_follow_me(&mut battle_state, position);

            assert_eq!(
                battle_state.redirection.as_ref().unwrap().redirector_position,
                position
            );
        }
    }
}

// ==================== RAGE POWDER TESTS ====================

#[cfg(test)]
mod rage_powder {
    use super::*;

    #[test]
    fn test_rage_powder_redirects_non_grass_attacks() {
        let mut battle_state = create_double_battle_state();
        set_rage_powder(&mut battle_state, FieldPosition::PlayerLeft);

        let move_data = create_single_target_move();
        let normal_attacker = create_test_pokemon(PokemonType::Normal, None);

        let result = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &normal_attacker,
            &move_data,
            &battle_state,
        );

        assert_eq!(
            result,
            Some(FieldPosition::PlayerLeft),
            "Rage Powder debe redirigir ataques de tipo Normal"
        );
    }

    #[test]
    fn test_rage_powder_ignores_primary_grass_type() {
        let mut battle_state = create_double_battle_state();
        set_rage_powder(&mut battle_state, FieldPosition::PlayerLeft);

        let move_data = create_single_target_move();
        let grass_attacker = create_test_pokemon(PokemonType::Grass, None);

        let result = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &grass_attacker,
            &move_data,
            &battle_state,
        );

        assert_eq!(
            result,
            None,
            "Rage Powder NO debe afectar a Pokémon con tipo Grass primario"
        );
    }

    #[test]
    fn test_rage_powder_ignores_secondary_grass_type() {
        let mut battle_state = create_double_battle_state();
        set_rage_powder(&mut battle_state, FieldPosition::PlayerLeft);

        let move_data = create_single_target_move();
        let grass_water_attacker = create_test_pokemon(PokemonType::Water, Some(PokemonType::Grass));

        let result = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &grass_water_attacker,
            &move_data,
            &battle_state,
        );

        assert_eq!(
            result,
            None,
            "Rage Powder NO debe afectar a Pokémon con tipo Grass secundario"
        );
    }

    #[test]
    fn test_rage_powder_redirects_fire_type() {
        let mut battle_state = create_double_battle_state();
        set_rage_powder(&mut battle_state, FieldPosition::PlayerLeft);

        let move_data = create_single_target_move();
        let fire_attacker = create_test_pokemon(PokemonType::Fire, None);

        let result = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &fire_attacker,
            &move_data,
            &battle_state,
        );

        assert_eq!(
            result,
            Some(FieldPosition::PlayerLeft),
            "Rage Powder debe redirigir ataques de tipo Fire"
        );
    }

    #[test]
    fn test_rage_powder_redirects_water_type() {
        let mut battle_state = create_double_battle_state();
        set_rage_powder(&mut battle_state, FieldPosition::PlayerLeft);

        let move_data = create_single_target_move();
        let water_attacker = create_test_pokemon(PokemonType::Water, None);

        let result = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &water_attacker,
            &move_data,
            &battle_state,
        );

        assert_eq!(
            result,
            Some(FieldPosition::PlayerLeft),
            "Rage Powder debe redirigir ataques de tipo Water"
        );
    }

    #[test]
    fn test_rage_powder_sets_redirection_state() {
        let mut battle_state = create_double_battle_state();
        set_rage_powder(&mut battle_state, FieldPosition::PlayerLeft);

        assert!(battle_state.redirection.is_some());
        let redirection = battle_state.redirection.as_ref().unwrap();
        assert_eq!(redirection.redirection_type, "rage-powder");
        assert!(redirection.opponent_only);
    }

    #[test]
    fn test_rage_powder_does_not_redirect_spread_moves() {
        let mut battle_state = create_double_battle_state();
        set_rage_powder(&mut battle_state, FieldPosition::PlayerLeft);

        let spread_move = create_spread_move();
        let attacker = create_test_pokemon(PokemonType::Normal, None);

        let result = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &attacker,
            &spread_move,
            &battle_state,
        );

        assert_eq!(result, None, "Rage Powder NO debe redirigir movimientos spread");
    }
}

// ==================== SPOTLIGHT TESTS ====================

#[cfg(test)]
mod spotlight {
    use super::*;

    #[test]
    fn test_spotlight_redirects_opponent_attacks() {
        let mut battle_state = create_double_battle_state();
        set_spotlight(&mut battle_state, FieldPosition::PlayerLeft);

        let move_data = create_single_target_move();
        let attacker = create_test_pokemon(PokemonType::Normal, None);

        let result = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &attacker,
            &move_data,
            &battle_state,
        );

        assert_eq!(
            result,
            Some(FieldPosition::PlayerLeft),
            "Spotlight debe redirigir ataques del oponente"
        );
    }

    #[test]
    fn test_spotlight_redirects_ally_attacks() {
        let mut battle_state = create_double_battle_state();
        set_spotlight(&mut battle_state, FieldPosition::OpponentLeft);

        let move_data = create_single_target_move();
        let attacker = create_test_pokemon(PokemonType::Normal, None);

        // Spotlight NO tiene opponent_only, debe redirigir ataques de aliados también
        let result = apply_redirection(
            FieldPosition::OpponentRight,
            FieldPosition::PlayerLeft,
            &attacker,
            &move_data,
            &battle_state,
        );

        assert_eq!(
            result,
            Some(FieldPosition::OpponentLeft),
            "Spotlight debe redirigir todos los ataques (opponent_only = false)"
        );
    }

    #[test]
    fn test_spotlight_affects_grass_types() {
        let mut battle_state = create_double_battle_state();
        set_spotlight(&mut battle_state, FieldPosition::PlayerLeft);

        let move_data = create_single_target_move();
        let grass_attacker = create_test_pokemon(PokemonType::Grass, None);

        // A diferencia de Rage Powder, Spotlight afecta a todos los tipos
        let result = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &grass_attacker,
            &move_data,
            &battle_state,
        );

        assert_eq!(
            result,
            Some(FieldPosition::PlayerLeft),
            "Spotlight debe afectar a tipos Grass (no es Rage Powder)"
        );
    }

    #[test]
    fn test_spotlight_sets_redirection_state() {
        let mut battle_state = create_double_battle_state();
        set_spotlight(&mut battle_state, FieldPosition::PlayerLeft);

        assert!(battle_state.redirection.is_some());
        let redirection = battle_state.redirection.as_ref().unwrap();
        assert_eq!(redirection.redirection_type, "spotlight");
        assert!(!redirection.opponent_only, "Spotlight afecta a todos");
    }

    #[test]
    fn test_spotlight_does_not_redirect_spread_moves() {
        let mut battle_state = create_double_battle_state();
        set_spotlight(&mut battle_state, FieldPosition::PlayerLeft);

        let spread_move = create_spread_move();
        let attacker = create_test_pokemon(PokemonType::Normal, None);

        let result = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &attacker,
            &spread_move,
            &battle_state,
        );

        assert_eq!(result, None, "Spotlight NO debe redirigir movimientos spread");
    }
}

// ==================== ALLY SWITCH TESTS ====================

#[cfg(test)]
mod ally_switch {
    use super::*;

    #[test]
    fn test_ally_switch_swaps_player_positions() {
        let mut battle_state = create_double_battle_state();
        let original_indices = battle_state.player_active_indices.clone();

        let success = ally_switch(&mut battle_state, FieldPosition::PlayerLeft);

        assert!(success, "Ally Switch debe ejecutarse exitosamente");
        assert_eq!(
            battle_state.player_active_indices[0],
            original_indices[1],
            "Primera posición debe tener el índice de la segunda"
        );
        assert_eq!(
            battle_state.player_active_indices[1],
            original_indices[0],
            "Segunda posición debe tener el índice de la primera"
        );
    }

    #[test]
    fn test_ally_switch_from_player_right() {
        let mut battle_state = create_double_battle_state();
        let original_indices = battle_state.player_active_indices.clone();

        let success = ally_switch(&mut battle_state, FieldPosition::PlayerRight);

        assert!(success);
        assert_eq!(battle_state.player_active_indices[0], original_indices[1]);
        assert_eq!(battle_state.player_active_indices[1], original_indices[0]);
    }

    #[test]
    fn test_ally_switch_swaps_opponent_positions() {
        let mut battle_state = create_double_battle_state();

        // Asegurar que hay 2 oponentes activos
        battle_state.opponent_active_indices = vec![0, 1];
        let original_indices = battle_state.opponent_active_indices.clone();

        let success = ally_switch(&mut battle_state, FieldPosition::OpponentLeft);

        assert!(success, "Ally Switch debe funcionar para el equipo oponente");
        assert_eq!(
            battle_state.opponent_active_indices[0],
            original_indices[1]
        );
        assert_eq!(
            battle_state.opponent_active_indices[1],
            original_indices[0]
        );
    }

    #[test]
    fn test_ally_switch_from_opponent_right() {
        let mut battle_state = create_double_battle_state();
        battle_state.opponent_active_indices = vec![0, 1];
        let original_indices = battle_state.opponent_active_indices.clone();

        let success = ally_switch(&mut battle_state, FieldPosition::OpponentRight);

        assert!(success);
        assert_eq!(battle_state.opponent_active_indices[0], original_indices[1]);
        assert_eq!(battle_state.opponent_active_indices[1], original_indices[0]);
    }

    #[test]
    fn test_ally_switch_fails_in_single_battle() {
        let mut battle_state = create_single_battle_state();

        let success = ally_switch(&mut battle_state, FieldPosition::PlayerLeft);

        assert!(!success, "Ally Switch NO debe funcionar en batallas single");
    }

    #[test]
    fn test_ally_switch_fails_with_only_one_pokemon() {
        let mut battle_state = create_double_battle_state();
        battle_state.player_active_indices = vec![0]; // Solo 1 Pokémon activo

        let success = ally_switch(&mut battle_state, FieldPosition::PlayerLeft);

        assert!(!success, "Ally Switch requiere 2 Pokémon activos");
    }

    #[test]
    fn test_ally_switch_can_be_used_multiple_times() {
        let mut battle_state = create_double_battle_state();
        let original_indices = battle_state.player_active_indices.clone();

        // Primer switch
        ally_switch(&mut battle_state, FieldPosition::PlayerLeft);
        assert_eq!(battle_state.player_active_indices[0], original_indices[1]);

        // Segundo switch (debe volver al estado original)
        ally_switch(&mut battle_state, FieldPosition::PlayerLeft);
        assert_eq!(battle_state.player_active_indices[0], original_indices[0]);
        assert_eq!(battle_state.player_active_indices[1], original_indices[1]);
    }
}

// ==================== GENERAL REDIRECTION TESTS ====================

#[cfg(test)]
mod general_redirection {
    use super::*;

    #[test]
    fn test_redirection_only_in_double_battles() {
        let mut battle_state = create_single_battle_state();
        set_follow_me(&mut battle_state, FieldPosition::PlayerLeft);

        let move_data = create_single_target_move();
        let attacker = create_test_pokemon(PokemonType::Normal, None);

        let result = apply_redirection(
            FieldPosition::OpponentLeft,
            FieldPosition::PlayerLeft,
            &attacker,
            &move_data,
            &battle_state,
        );

        assert_eq!(
            result,
            None,
            "Redirección NO debe aplicarse en batallas single"
        );
    }

    #[test]
    fn test_clear_redirection_removes_state() {
        let mut battle_state = create_double_battle_state();
        set_follow_me(&mut battle_state, FieldPosition::PlayerLeft);

        assert!(battle_state.redirection.is_some(), "Debe haber redirección activa");

        clear_redirection(&mut battle_state);

        assert!(battle_state.redirection.is_none(), "clear_redirection debe eliminar el estado");
    }

    #[test]
    fn test_no_redirection_when_none_active() {
        let battle_state = create_double_battle_state();

        let move_data = create_single_target_move();
        let attacker = create_test_pokemon(PokemonType::Normal, None);

        let result = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &attacker,
            &move_data,
            &battle_state,
        );

        assert_eq!(result, None, "Sin redirección activa, no debe redirigir");
    }

    #[test]
    fn test_multiple_move_targets_not_redirected() {
        let mut battle_state = create_double_battle_state();
        set_follow_me(&mut battle_state, FieldPosition::PlayerLeft);

        let attacker = create_test_pokemon(PokemonType::Normal, None);

        // Test varios tipos de movimientos multi-target
        let targets = vec![
            "all-opponents",
            "all-other-pokemon",
            "all-pokemon",
            "entire-field",
            "user",
            "all-allies",
        ];

        for target in targets {
            let mut move_data = create_single_target_move();
            move_data.target = target.to_string();

            let result = apply_redirection(
                FieldPosition::PlayerRight,
                FieldPosition::OpponentLeft,
                &attacker,
                &move_data,
                &battle_state,
            );

            assert_eq!(
                result,
                None,
                "Movimiento con target '{}' NO debe ser redirigido",
                target
            );
        }
    }

    #[test]
    fn test_single_target_moves_are_redirected() {
        let mut battle_state = create_double_battle_state();
        set_follow_me(&mut battle_state, FieldPosition::PlayerLeft);

        let attacker = create_test_pokemon(PokemonType::Normal, None);

        // Test que movimientos single-target SÍ son redirigidos
        let single_targets = vec![
            "selected-pokemon",
            "selected-pokemon-me-first",
            "random-opponent",
        ];

        for target in single_targets {
            let mut move_data = create_single_target_move();
            move_data.target = target.to_string();

            let result = apply_redirection(
                FieldPosition::PlayerRight,
                FieldPosition::OpponentLeft,
                &attacker,
                &move_data,
                &battle_state,
            );

            assert_eq!(
                result,
                Some(FieldPosition::PlayerLeft),
                "Movimiento con target '{}' DEBE ser redirigido",
                target
            );
        }
    }
}

// ==================== INTEGRATION TESTS ====================

#[cfg(test)]
mod integration {
    use super::*;

    #[test]
    fn test_redirection_can_be_changed() {
        let mut battle_state = create_double_battle_state();

        // Primero Follow Me
        set_follow_me(&mut battle_state, FieldPosition::PlayerLeft);
        assert_eq!(
            battle_state.redirection.as_ref().unwrap().redirection_type,
            "follow-me"
        );

        // Luego Rage Powder (sobrescribe)
        set_rage_powder(&mut battle_state, FieldPosition::PlayerRight);
        assert_eq!(
            battle_state.redirection.as_ref().unwrap().redirection_type,
            "rage-powder"
        );
        assert_eq!(
            battle_state.redirection.as_ref().unwrap().redirector_position,
            FieldPosition::PlayerRight
        );
    }

    #[test]
    fn test_redirection_after_clear_can_be_reactivated() {
        let mut battle_state = create_double_battle_state();

        set_follow_me(&mut battle_state, FieldPosition::PlayerLeft);
        clear_redirection(&mut battle_state);
        assert!(battle_state.redirection.is_none());

        set_follow_me(&mut battle_state, FieldPosition::PlayerRight);
        assert!(battle_state.redirection.is_some());
        assert_eq!(
            battle_state.redirection.as_ref().unwrap().redirector_position,
            FieldPosition::PlayerRight
        );
    }

    #[test]
    fn test_ally_switch_does_not_affect_redirection_state() {
        let mut battle_state = create_double_battle_state();
        set_follow_me(&mut battle_state, FieldPosition::PlayerLeft);

        ally_switch(&mut battle_state, FieldPosition::PlayerLeft);

        // Redirection state debe mantenerse después de Ally Switch
        assert!(
            battle_state.redirection.is_some(),
            "Ally Switch NO debe limpiar el estado de redirección"
        );
    }

    #[test]
    fn test_complex_scenario_rage_powder_vs_grass() {
        let mut battle_state = create_double_battle_state();
        set_rage_powder(&mut battle_state, FieldPosition::PlayerLeft);

        let move_data = create_single_target_move();

        // Tipo Normal: redirigido
        let normal_attacker = create_test_pokemon(PokemonType::Normal, None);
        let result1 = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &normal_attacker,
            &move_data,
            &battle_state,
        );
        assert_eq!(result1, Some(FieldPosition::PlayerLeft));

        // Tipo Grass primario: NO redirigido
        let grass_attacker = create_test_pokemon(PokemonType::Grass, None);
        let result2 = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &grass_attacker,
            &move_data,
            &battle_state,
        );
        assert_eq!(result2, None);

        // Tipo Water/Grass: NO redirigido (tiene Grass secundario)
        let water_grass = create_test_pokemon(PokemonType::Water, Some(PokemonType::Grass));
        let result3 = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &water_grass,
            &move_data,
            &battle_state,
        );
        assert_eq!(result3, None);
    }

    #[test]
    fn test_spotlight_vs_follow_me_difference() {
        let move_data = create_single_target_move();
        let attacker = create_test_pokemon(PokemonType::Normal, None);

        // Follow Me (opponent_only = true)
        let mut battle_state1 = create_double_battle_state();
        set_follow_me(&mut battle_state1, FieldPosition::PlayerLeft);
        let result1 = apply_redirection(
            FieldPosition::OpponentRight,
            FieldPosition::PlayerRight,
            &attacker,
            &move_data,
            &battle_state1,
        );
        assert_eq!(result1, None, "Follow Me NO redirige ataques de aliados");

        // Spotlight (opponent_only = false)
        let mut battle_state2 = create_double_battle_state();
        set_spotlight(&mut battle_state2, FieldPosition::OpponentLeft);
        let result2 = apply_redirection(
            FieldPosition::OpponentRight,
            FieldPosition::PlayerLeft,
            &attacker,
            &move_data,
            &battle_state2,
        );
        assert_eq!(
            result2,
            Some(FieldPosition::OpponentLeft),
            "Spotlight SÍ redirige todos los ataques"
        );
    }
}
