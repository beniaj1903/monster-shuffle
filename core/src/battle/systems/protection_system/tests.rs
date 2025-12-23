//! Tests comprehensivos para el Protection System (Fase 4.2)
//!
//! Este módulo contiene tests exhaustivos para Wide Guard, Quick Guard,
//! Mat Block y Crafty Shield implementados en las Fases 1 y 2.

use crate::models::{
    PokemonInstance, PokemonSpecies, Stats, RandomizedProfile, PokemonType,
    StatModifiers, VolatileStatus, StatStages, MoveData, MoveMeta,
};
use super::processor::*;

/// Helper para crear Pokémon de prueba con configuración personalizada
fn create_test_pokemon() -> PokemonInstance {
    PokemonInstance {
        id: "test-1".to_string(),
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
            rolled_primary_type: PokemonType::Electric,
            rolled_secondary_type: None,
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

/// Helper para crear un movimiento spread (afecta a múltiples objetivos)
fn create_spread_move(move_id: &str, name: &str) -> MoveData {
    MoveData {
        id: move_id.to_string(),
        name: name.to_string(),
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

/// Helper para crear un movimiento de prioridad
fn create_priority_move(priority_level: i8) -> MoveData {
    MoveData {
        id: "quick-attack".to_string(),
        name: "Quick Attack".to_string(),
        r#type: "Normal".to_string(),
        power: Some(40),
        accuracy: Some(100),
        priority: priority_level,
        pp: 30,
        damage_class: "physical".to_string(),
        meta: MoveMeta::default(),
        stat_changes: vec![],
        target: "selected-pokemon".to_string(),
    }
}

/// Helper para crear un movimiento de estado
fn create_status_move(move_id: &str) -> MoveData {
    MoveData {
        id: move_id.to_string(),
        name: "Thunder Wave".to_string(),
        r#type: "Electric".to_string(),
        power: None,
        accuracy: Some(90),
        priority: 0,
        pp: 20,
        damage_class: "status".to_string(),
        meta: MoveMeta::default(),
        stat_changes: vec![],
        target: "selected-pokemon".to_string(),
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

// ==================== WIDE GUARD TESTS ====================

#[cfg(test)]
mod wide_guard {
    use super::*;

    #[test]
    fn test_wide_guard_blocks_all_opponents_moves() {
        let mut pokemon = create_test_pokemon();
        activate_wide_guard(&mut pokemon);

        let earthquake = create_spread_move("earthquake", "Earthquake");
        assert!(
            is_blocked_by_wide_guard(&pokemon, &earthquake),
            "Wide Guard debe bloquear Earthquake (all-opponents)"
        );
    }

    #[test]
    fn test_wide_guard_blocks_all_other_pokemon_moves() {
        let mut pokemon = create_test_pokemon();
        activate_wide_guard(&mut pokemon);

        let mut explosion = create_spread_move("explosion", "Explosion");
        explosion.target = "all-other-pokemon".to_string();

        assert!(
            is_blocked_by_wide_guard(&pokemon, &explosion),
            "Wide Guard debe bloquear Explosion (all-other-pokemon)"
        );
    }

    #[test]
    fn test_wide_guard_blocks_all_pokemon_moves() {
        let mut pokemon = create_test_pokemon();
        activate_wide_guard(&mut pokemon);

        let mut move_data = create_spread_move("haze", "Haze");
        move_data.target = "all-pokemon".to_string();

        assert!(
            is_blocked_by_wide_guard(&pokemon, &move_data),
            "Wide Guard debe bloquear movimientos all-pokemon"
        );
    }

    #[test]
    fn test_wide_guard_does_not_block_single_target() {
        let mut pokemon = create_test_pokemon();
        activate_wide_guard(&mut pokemon);

        let tackle = create_single_target_move();
        assert!(
            !is_blocked_by_wide_guard(&pokemon, &tackle),
            "Wide Guard NO debe bloquear movimientos single-target"
        );
    }

    #[test]
    fn test_wide_guard_inactive_by_default() {
        let pokemon = create_test_pokemon();
        let earthquake = create_spread_move("earthquake", "Earthquake");

        assert!(
            !is_blocked_by_wide_guard(&pokemon, &earthquake),
            "Sin activar Wide Guard, no debe bloquear nada"
        );
    }

    #[test]
    fn test_wide_guard_activation() {
        let mut pokemon = create_test_pokemon();
        activate_wide_guard(&mut pokemon);

        assert!(
            pokemon.volatile_status.as_ref().unwrap().wide_guard_active,
            "activate_wide_guard debe marcar wide_guard_active como true"
        );
    }

    #[test]
    fn test_wide_guard_without_volatile_status() {
        let mut pokemon = create_test_pokemon();
        pokemon.volatile_status = None;

        let earthquake = create_spread_move("earthquake", "Earthquake");
        assert!(
            !is_blocked_by_wide_guard(&pokemon, &earthquake),
            "Sin volatile_status, Wide Guard no puede estar activo"
        );
    }
}

// ==================== QUICK GUARD TESTS ====================

#[cfg(test)]
mod quick_guard {
    use super::*;

    #[test]
    fn test_quick_guard_blocks_priority_1() {
        let mut pokemon = create_test_pokemon();
        activate_quick_guard(&mut pokemon);

        let quick_attack = create_priority_move(1);
        assert!(
            is_blocked_by_quick_guard(&pokemon, &quick_attack),
            "Quick Guard debe bloquear movimientos con priority 1"
        );
    }

    #[test]
    fn test_quick_guard_blocks_priority_2() {
        let mut pokemon = create_test_pokemon();
        activate_quick_guard(&mut pokemon);

        let extreme_speed = create_priority_move(2);
        assert!(
            is_blocked_by_quick_guard(&pokemon, &extreme_speed),
            "Quick Guard debe bloquear movimientos con priority 2"
        );
    }

    #[test]
    fn test_quick_guard_blocks_priority_5() {
        let mut pokemon = create_test_pokemon();
        activate_quick_guard(&mut pokemon);

        let helping_hand = create_priority_move(5);
        assert!(
            is_blocked_by_quick_guard(&pokemon, &helping_hand),
            "Quick Guard debe bloquear movimientos con priority muy alta"
        );
    }

    #[test]
    fn test_quick_guard_does_not_block_priority_0() {
        let mut pokemon = create_test_pokemon();
        activate_quick_guard(&mut pokemon);

        let tackle = create_priority_move(0);
        assert!(
            !is_blocked_by_quick_guard(&pokemon, &tackle),
            "Quick Guard NO debe bloquear movimientos con priority 0"
        );
    }

    #[test]
    fn test_quick_guard_does_not_block_negative_priority() {
        let mut pokemon = create_test_pokemon();
        activate_quick_guard(&mut pokemon);

        let vital_throw = create_priority_move(-1);
        assert!(
            !is_blocked_by_quick_guard(&pokemon, &vital_throw),
            "Quick Guard NO debe bloquear movimientos con priority negativa"
        );
    }

    #[test]
    fn test_quick_guard_inactive_by_default() {
        let pokemon = create_test_pokemon();
        let quick_attack = create_priority_move(1);

        assert!(
            !is_blocked_by_quick_guard(&pokemon, &quick_attack),
            "Sin activar Quick Guard, no debe bloquear nada"
        );
    }

    #[test]
    fn test_quick_guard_activation() {
        let mut pokemon = create_test_pokemon();
        activate_quick_guard(&mut pokemon);

        assert!(
            pokemon.volatile_status.as_ref().unwrap().quick_guard_active,
            "activate_quick_guard debe marcar quick_guard_active como true"
        );
    }

    #[test]
    fn test_quick_guard_without_volatile_status() {
        let mut pokemon = create_test_pokemon();
        pokemon.volatile_status = None;

        let quick_attack = create_priority_move(1);
        assert!(
            !is_blocked_by_quick_guard(&pokemon, &quick_attack),
            "Sin volatile_status, Quick Guard no puede estar activo"
        );
    }
}

// ==================== MAT BLOCK TESTS ====================

#[cfg(test)]
mod mat_block {
    use super::*;

    #[test]
    fn test_mat_block_blocks_physical_moves() {
        let mut pokemon = create_test_pokemon();
        activate_mat_block(&mut pokemon);

        let tackle = create_single_target_move();
        assert!(
            is_blocked_by_mat_block(&pokemon, &tackle),
            "Mat Block debe bloquear movimientos físicos dañinos"
        );
    }

    #[test]
    fn test_mat_block_blocks_special_moves() {
        let mut pokemon = create_test_pokemon();
        activate_mat_block(&mut pokemon);

        let mut thunderbolt = create_single_target_move();
        thunderbolt.id = "thunderbolt".to_string();
        thunderbolt.damage_class = "special".to_string();
        thunderbolt.power = Some(90);

        assert!(
            is_blocked_by_mat_block(&pokemon, &thunderbolt),
            "Mat Block debe bloquear movimientos especiales dañinos"
        );
    }

    #[test]
    fn test_mat_block_blocks_spread_moves() {
        let mut pokemon = create_test_pokemon();
        activate_mat_block(&mut pokemon);

        let earthquake = create_spread_move("earthquake", "Earthquake");
        assert!(
            is_blocked_by_mat_block(&pokemon, &earthquake),
            "Mat Block debe bloquear movimientos spread dañinos"
        );
    }

    #[test]
    fn test_mat_block_does_not_block_status_moves() {
        let mut pokemon = create_test_pokemon();
        activate_mat_block(&mut pokemon);

        let thunder_wave = create_status_move("thunder-wave");
        assert!(
            !is_blocked_by_mat_block(&pokemon, &thunder_wave),
            "Mat Block NO debe bloquear movimientos de estado (power = None)"
        );
    }

    #[test]
    fn test_mat_block_inactive_by_default() {
        let pokemon = create_test_pokemon();
        let tackle = create_single_target_move();

        assert!(
            !is_blocked_by_mat_block(&pokemon, &tackle),
            "Sin activar Mat Block, no debe bloquear nada"
        );
    }

    #[test]
    fn test_mat_block_activation() {
        let mut pokemon = create_test_pokemon();
        activate_mat_block(&mut pokemon);

        assert!(
            pokemon.volatile_status.as_ref().unwrap().mat_block_active,
            "activate_mat_block debe marcar mat_block_active como true"
        );
    }

    #[test]
    fn test_mat_block_without_volatile_status() {
        let mut pokemon = create_test_pokemon();
        pokemon.volatile_status = None;

        let tackle = create_single_target_move();
        assert!(
            !is_blocked_by_mat_block(&pokemon, &tackle),
            "Sin volatile_status, Mat Block no puede estar activo"
        );
    }
}

// ==================== CRAFTY SHIELD TESTS ====================

#[cfg(test)]
mod crafty_shield {
    use super::*;

    #[test]
    fn test_crafty_shield_blocks_status_moves() {
        let mut pokemon = create_test_pokemon();
        activate_crafty_shield(&mut pokemon);

        let thunder_wave = create_status_move("thunder-wave");
        assert!(
            is_blocked_by_crafty_shield(&pokemon, &thunder_wave),
            "Crafty Shield debe bloquear Thunder Wave"
        );
    }

    #[test]
    fn test_crafty_shield_blocks_various_status_moves() {
        let mut pokemon = create_test_pokemon();
        activate_crafty_shield(&mut pokemon);

        let moves = vec![
            create_status_move("toxic"),
            create_status_move("will-o-wisp"),
            create_status_move("spore"),
        ];

        for move_data in moves {
            assert!(
                is_blocked_by_crafty_shield(&pokemon, &move_data),
                "Crafty Shield debe bloquear {}", move_data.id
            );
        }
    }

    #[test]
    fn test_crafty_shield_does_not_block_physical_damaging() {
        let mut pokemon = create_test_pokemon();
        activate_crafty_shield(&mut pokemon);

        let tackle = create_single_target_move();
        assert!(
            !is_blocked_by_crafty_shield(&pokemon, &tackle),
            "Crafty Shield NO debe bloquear movimientos físicos dañinos"
        );
    }

    #[test]
    fn test_crafty_shield_does_not_block_special_damaging() {
        let mut pokemon = create_test_pokemon();
        activate_crafty_shield(&mut pokemon);

        let mut thunderbolt = create_single_target_move();
        thunderbolt.id = "thunderbolt".to_string();
        thunderbolt.damage_class = "special".to_string();
        thunderbolt.power = Some(90);

        assert!(
            !is_blocked_by_crafty_shield(&pokemon, &thunderbolt),
            "Crafty Shield NO debe bloquear movimientos especiales dañinos"
        );
    }

    #[test]
    fn test_crafty_shield_does_not_block_spread_damaging() {
        let mut pokemon = create_test_pokemon();
        activate_crafty_shield(&mut pokemon);

        let earthquake = create_spread_move("earthquake", "Earthquake");
        assert!(
            !is_blocked_by_crafty_shield(&pokemon, &earthquake),
            "Crafty Shield NO debe bloquear movimientos spread dañinos"
        );
    }

    #[test]
    fn test_crafty_shield_inactive_by_default() {
        let pokemon = create_test_pokemon();
        let thunder_wave = create_status_move("thunder-wave");

        assert!(
            !is_blocked_by_crafty_shield(&pokemon, &thunder_wave),
            "Sin activar Crafty Shield, no debe bloquear nada"
        );
    }

    #[test]
    fn test_crafty_shield_activation() {
        let mut pokemon = create_test_pokemon();
        activate_crafty_shield(&mut pokemon);

        assert!(
            pokemon.volatile_status.as_ref().unwrap().crafty_shield_active,
            "activate_crafty_shield debe marcar crafty_shield_active como true"
        );
    }

    #[test]
    fn test_crafty_shield_without_volatile_status() {
        let mut pokemon = create_test_pokemon();
        pokemon.volatile_status = None;

        let thunder_wave = create_status_move("thunder-wave");
        assert!(
            !is_blocked_by_crafty_shield(&pokemon, &thunder_wave),
            "Sin volatile_status, Crafty Shield no puede estar activo"
        );
    }
}

// ==================== ADVANCED PROTECTIONS TESTS ====================

#[cfg(test)]
mod advanced_protections {
    use super::*;

    #[test]
    fn test_check_advanced_protections_returns_wide_guard_message() {
        let mut pokemon = create_test_pokemon();
        activate_wide_guard(&mut pokemon);

        let earthquake = create_spread_move("earthquake", "Earthquake");
        let result = check_advanced_protections(&pokemon, &earthquake);

        assert!(result.is_some(), "Debe retornar un mensaje de protección");
        assert!(
            result.unwrap().contains("Wide Guard"),
            "El mensaje debe mencionar Wide Guard"
        );
    }

    #[test]
    fn test_check_advanced_protections_returns_quick_guard_message() {
        let mut pokemon = create_test_pokemon();
        activate_quick_guard(&mut pokemon);

        let quick_attack = create_priority_move(1);
        let result = check_advanced_protections(&pokemon, &quick_attack);

        assert!(result.is_some());
        assert!(result.unwrap().contains("Quick Guard"));
    }

    #[test]
    fn test_check_advanced_protections_returns_mat_block_message() {
        let mut pokemon = create_test_pokemon();
        activate_mat_block(&mut pokemon);

        let tackle = create_single_target_move();
        let result = check_advanced_protections(&pokemon, &tackle);

        assert!(result.is_some());
        assert!(result.unwrap().contains("Mat Block"));
    }

    #[test]
    fn test_check_advanced_protections_returns_crafty_shield_message() {
        let mut pokemon = create_test_pokemon();
        activate_crafty_shield(&mut pokemon);

        let thunder_wave = create_status_move("thunder-wave");
        let result = check_advanced_protections(&pokemon, &thunder_wave);

        assert!(result.is_some());
        assert!(result.unwrap().contains("Crafty Shield"));
    }

    #[test]
    fn test_check_advanced_protections_priority_order() {
        // Wide Guard tiene prioridad sobre Quick Guard en la función check_advanced_protections
        let mut pokemon = create_test_pokemon();
        activate_wide_guard(&mut pokemon);
        activate_quick_guard(&mut pokemon);

        // Movimiento que es spread Y tiene prioridad
        let mut move_data = create_spread_move("fake-move", "Fake Move");
        move_data.priority = 1;

        let result = check_advanced_protections(&pokemon, &move_data);

        assert!(result.is_some());
        // Wide Guard se verifica primero, debe bloquear
        assert!(
            result.unwrap().contains("Wide Guard"),
            "Wide Guard debe tener prioridad en el orden de verificación"
        );
    }

    #[test]
    fn test_check_advanced_protections_returns_none_when_not_protected() {
        let pokemon = create_test_pokemon();
        let tackle = create_single_target_move();

        let result = check_advanced_protections(&pokemon, &tackle);

        assert!(result.is_none(), "Sin protecciones activas, debe retornar None");
    }

    #[test]
    fn test_clear_advanced_protections_clears_all() {
        let mut pokemon = create_test_pokemon();
        activate_wide_guard(&mut pokemon);
        activate_quick_guard(&mut pokemon);
        activate_mat_block(&mut pokemon);
        activate_crafty_shield(&mut pokemon);

        clear_advanced_protections(&mut pokemon);

        let volatile = pokemon.volatile_status.as_ref().unwrap();
        assert!(!volatile.wide_guard_active, "Wide Guard debe estar inactivo");
        assert!(!volatile.quick_guard_active, "Quick Guard debe estar inactivo");
        assert!(!volatile.mat_block_active, "Mat Block debe estar inactivo");
        assert!(!volatile.crafty_shield_active, "Crafty Shield debe estar inactivo");
    }

    #[test]
    fn test_clear_protections_prevents_blocking() {
        let mut pokemon = create_test_pokemon();
        activate_wide_guard(&mut pokemon);
        activate_quick_guard(&mut pokemon);
        activate_mat_block(&mut pokemon);
        activate_crafty_shield(&mut pokemon);

        clear_advanced_protections(&mut pokemon);

        // Verificar que ninguna protección bloquea después de clear
        let earthquake = create_spread_move("earthquake", "Earthquake");
        let quick_attack = create_priority_move(1);
        let tackle = create_single_target_move();
        let thunder_wave = create_status_move("thunder-wave");

        assert!(!is_blocked_by_wide_guard(&pokemon, &earthquake));
        assert!(!is_blocked_by_quick_guard(&pokemon, &quick_attack));
        assert!(!is_blocked_by_mat_block(&pokemon, &tackle));
        assert!(!is_blocked_by_crafty_shield(&pokemon, &thunder_wave));
    }
}

// ==================== INTEGRATION TESTS ====================

#[cfg(test)]
mod integration {
    use super::*;

    #[test]
    fn test_multiple_protections_can_be_active_simultaneously() {
        let mut pokemon = create_test_pokemon();
        activate_wide_guard(&mut pokemon);
        activate_quick_guard(&mut pokemon);
        activate_mat_block(&mut pokemon);
        activate_crafty_shield(&mut pokemon);

        let volatile = pokemon.volatile_status.as_ref().unwrap();
        assert!(volatile.wide_guard_active);
        assert!(volatile.quick_guard_active);
        assert!(volatile.mat_block_active);
        assert!(volatile.crafty_shield_active);
    }

    #[test]
    fn test_protections_can_be_reactivated_after_clear() {
        let mut pokemon = create_test_pokemon();

        // Primera activación
        activate_wide_guard(&mut pokemon);
        assert!(pokemon.volatile_status.as_ref().unwrap().wide_guard_active);

        // Limpiar
        clear_advanced_protections(&mut pokemon);
        assert!(!pokemon.volatile_status.as_ref().unwrap().wide_guard_active);

        // Reactivar
        activate_wide_guard(&mut pokemon);
        assert!(pokemon.volatile_status.as_ref().unwrap().wide_guard_active);
    }

    #[test]
    fn test_activation_initializes_volatile_status_if_none() {
        let mut pokemon = create_test_pokemon();
        pokemon.volatile_status = None;

        activate_wide_guard(&mut pokemon);

        assert!(pokemon.volatile_status.is_some(), "Debe inicializar volatile_status");
        assert!(
            pokemon.volatile_status.as_ref().unwrap().wide_guard_active,
            "Wide Guard debe estar activo después de inicializar"
        );
    }

    #[test]
    fn test_all_activation_functions_initialize_volatile_status() {
        // Test para cada función de activación
        let mut poke1 = create_test_pokemon();
        poke1.volatile_status = None;
        activate_wide_guard(&mut poke1);
        assert!(poke1.volatile_status.is_some());

        let mut poke2 = create_test_pokemon();
        poke2.volatile_status = None;
        activate_quick_guard(&mut poke2);
        assert!(poke2.volatile_status.is_some());

        let mut poke3 = create_test_pokemon();
        poke3.volatile_status = None;
        activate_mat_block(&mut poke3);
        assert!(poke3.volatile_status.is_some());

        let mut poke4 = create_test_pokemon();
        poke4.volatile_status = None;
        activate_crafty_shield(&mut poke4);
        assert!(poke4.volatile_status.is_some());
    }
}
