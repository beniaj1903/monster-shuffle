//! Tests comprehensivos para el Item System (Fase 4.1)
//!
//! Este módulo contiene tests exhaustivos para todos los items implementados
//! en las Fases 1 y 2, con >90% code coverage objetivo.

use crate::models::{PokemonInstance, PokemonSpecies, Stats, RandomizedProfile, PokemonType, StatModifiers, StatusCondition, VolatileStatus, StatStages};
use super::item_effects::apply_item_effect;

/// Helper para crear Pokémon de prueba con configuración personalizada
fn create_test_pokemon(
    item: Option<String>,
    hp: u16,
    max_hp: u16,
    status: Option<StatusCondition>,
) -> PokemonInstance {
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
        current_hp: hp,
        status_condition: status,
        held_item: item,
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
            hp: max_hp,
            attack: 100,
            defense: 100,
            special_attack: 100,
            special_defense: 100,
            speed: 100,
        },
    }
}

// ==================== CHOICE ITEMS TESTS ====================

#[cfg(test)]
mod choice_items {
    use super::*;

    #[test]
    fn test_choice_band_boosts_damage() {
        let mut pokemon = create_test_pokemon(Some("choice-band".to_string()), 200, 200, None);
        let result = apply_item_effect("choice-band", &mut pokemon, Some("tackle"), None);

        assert_eq!(result.damage_multiplier, 1.5, "Choice Band debe dar +50% daño");
        assert!(!result.consumed, "Choice Band no es consumible");
        assert_eq!(result.move_locked, Some("tackle".to_string()), "Debe bloquear el movimiento");
    }

    #[test]
    fn test_choice_band_locks_move() {
        let mut pokemon = create_test_pokemon(Some("choice-band".to_string()), 200, 200, None);
        let result = apply_item_effect("choice-band", &mut pokemon, Some("earthquake"), None);

        assert_eq!(
            result.move_locked,
            Some("earthquake".to_string()),
            "Debe bloquear a Earthquake"
        );
    }

    #[test]
    fn test_choice_specs_boosts_special_damage() {
        let mut pokemon = create_test_pokemon(Some("choice-specs".to_string()), 200, 200, None);
        let result = apply_item_effect("choice-specs", &mut pokemon, Some("thunderbolt"), None);

        assert_eq!(result.damage_multiplier, 1.5, "Choice Specs debe dar +50% daño especial");
        assert!(!result.consumed, "Choice Specs no es consumible");
        assert_eq!(result.move_locked, Some("thunderbolt".to_string()));
    }

    #[test]
    fn test_choice_scarf_locks_move() {
        let mut pokemon = create_test_pokemon(Some("choice-scarf".to_string()), 200, 200, None);
        let result = apply_item_effect("choice-scarf", &mut pokemon, Some("quick-attack"), None);

        assert_eq!(
            result.move_locked,
            Some("quick-attack".to_string()),
            "Choice Scarf debe bloquear el movimiento"
        );
        assert!(!result.consumed, "Choice Scarf no es consumible");
    }

    #[test]
    fn test_choice_items_require_move_id() {
        let mut pokemon = create_test_pokemon(Some("choice-band".to_string()), 200, 200, None);
        let result = apply_item_effect("choice-band", &mut pokemon, None, None);

        assert_eq!(result.damage_multiplier, 1.0, "Sin move_id, no debe aplicar boost");
        assert_eq!(result.move_locked, None, "Sin move_id, no debe bloquear");
    }
}

// ==================== LIFE ORB TESTS ====================

#[cfg(test)]
mod life_orb {
    use super::*;

    #[test]
    fn test_life_orb_boosts_damage() {
        let mut pokemon = create_test_pokemon(Some("life-orb".to_string()), 200, 200, None);
        let result = apply_item_effect("life-orb", &mut pokemon, Some("tackle"), Some(50));

        assert_eq!(result.damage_multiplier, 1.3, "Life Orb debe dar +30% daño");
        assert!(!result.consumed, "Life Orb no es consumible");
    }

    #[test]
    fn test_life_orb_applies_recoil() {
        let mut pokemon = create_test_pokemon(Some("life-orb".to_string()), 200, 200, None);
        let result = apply_item_effect("life-orb", &mut pokemon, Some("tackle"), Some(50));

        assert_eq!(result.recoil_damage, 20, "Recoil debe ser 10% de 200 HP = 20");
        assert_eq!(pokemon.current_hp, 180, "HP debe reducirse por recoil");
    }

    #[test]
    fn test_life_orb_recoil_cannot_kill() {
        let mut pokemon = create_test_pokemon(Some("life-orb".to_string()), 15, 200, None);
        let result = apply_item_effect("life-orb", &mut pokemon, Some("tackle"), Some(50));

        assert_eq!(result.recoil_damage, 20, "Recoil calculado debe ser 20");
        assert_eq!(pokemon.current_hp, 0, "HP debe usar saturating_sub (no underflow)");
    }

    #[test]
    fn test_life_orb_requires_damage_dealt() {
        let mut pokemon = create_test_pokemon(Some("life-orb".to_string()), 200, 200, None);
        let result = apply_item_effect("life-orb", &mut pokemon, Some("tackle"), None);

        assert_eq!(result.damage_multiplier, 1.0, "Sin damage_dealt, no debe aplicar boost");
        assert_eq!(result.recoil_damage, 0, "Sin damage_dealt, no debe aplicar recoil");
        assert_eq!(pokemon.current_hp, 200, "HP no debe cambiar");
    }
}

// ==================== ASSAULT VEST TESTS ====================

#[cfg(test)]
mod assault_vest {
    use super::*;

    #[test]
    fn test_assault_vest_is_passive() {
        let mut pokemon = create_test_pokemon(Some("assault-vest".to_string()), 200, 200, None);
        let result = apply_item_effect("assault-vest", &mut pokemon, None, None);

        // Assault Vest no genera efectos directos en apply_item_effect
        // Su efecto se aplica en el damage calculator y validation system
        assert_eq!(result.damage_multiplier, 1.0);
        assert!(!result.consumed);
    }
}

// ==================== SITRUS BERRY TESTS ====================

#[cfg(test)]
mod sitrus_berry {
    use super::*;

    #[test]
    fn test_sitrus_berry_heals_25_percent() {
        let mut pokemon = create_test_pokemon(Some("sitrus-berry".to_string()), 90, 200, None);
        let result = apply_item_effect("sitrus-berry", &mut pokemon, None, None);

        assert_eq!(result.healed_hp, 50, "Debe curar 25% de 200 HP = 50");
        assert_eq!(pokemon.current_hp, 140, "HP debe ser 90 + 50 = 140");
        assert!(result.consumed, "Sitrus Berry debe consumirse");
        assert!(pokemon.held_item.is_none(), "Item debe removerse después de consumirse");
    }

    #[test]
    fn test_sitrus_berry_cannot_overheal() {
        let mut pokemon = create_test_pokemon(Some("sitrus-berry".to_string()), 180, 200, None);
        let result = apply_item_effect("sitrus-berry", &mut pokemon, None, None);

        assert_eq!(result.healed_hp, 20, "Solo debe curar hasta HP máximo (200 - 180 = 20)");
        assert_eq!(pokemon.current_hp, 200, "HP no debe exceder el máximo");
        assert!(result.consumed);
    }

    #[test]
    fn test_sitrus_berry_at_full_hp() {
        let mut pokemon = create_test_pokemon(Some("sitrus-berry".to_string()), 200, 200, None);
        let result = apply_item_effect("sitrus-berry", &mut pokemon, None, None);

        assert_eq!(result.healed_hp, 0, "No debe curar si ya está al máximo");
        assert_eq!(pokemon.current_hp, 200);
        assert!(result.consumed, "Aún así debe consumirse");
    }
}

// ==================== LUM BERRY TESTS ====================

#[cfg(test)]
mod lum_berry {
    use super::*;

    #[test]
    fn test_lum_berry_cures_burn() {
        let mut pokemon = create_test_pokemon(
            Some("lum-berry".to_string()),
            200,
            200,
            Some(StatusCondition::Burn),
        );
        let result = apply_item_effect("lum-berry", &mut pokemon, None, None);

        assert!(result.status_cured, "Debe indicar que curó el status");
        assert!(pokemon.status_condition.is_none(), "Burn debe ser curado");
        assert!(result.consumed, "Lum Berry debe consumirse");
        assert!(pokemon.held_item.is_none(), "Item debe removerse");
    }

    #[test]
    fn test_lum_berry_cures_paralysis() {
        let mut pokemon = create_test_pokemon(
            Some("lum-berry".to_string()),
            200,
            200,
            Some(StatusCondition::Paralysis),
        );
        let result = apply_item_effect("lum-berry", &mut pokemon, None, None);

        assert!(result.status_cured);
        assert!(pokemon.status_condition.is_none(), "Paralysis debe ser curado");
        assert!(result.consumed);
    }

    #[test]
    fn test_lum_berry_cures_poison() {
        let mut pokemon = create_test_pokemon(
            Some("lum-berry".to_string()),
            200,
            200,
            Some(StatusCondition::Poison),
        );
        let result = apply_item_effect("lum-berry", &mut pokemon, None, None);

        assert!(result.status_cured);
        assert!(pokemon.status_condition.is_none(), "Poison debe ser curado");
        assert!(result.consumed);
    }

    #[test]
    fn test_lum_berry_cures_bad_poison() {
        let mut pokemon = create_test_pokemon(
            Some("lum-berry".to_string()),
            200,
            200,
            Some(StatusCondition::BadPoison),
        );
        let result = apply_item_effect("lum-berry", &mut pokemon, None, None);

        assert!(result.status_cured);
        assert!(pokemon.status_condition.is_none(), "Bad Poison debe ser curado");
        assert!(result.consumed);
    }

    #[test]
    fn test_lum_berry_cures_freeze() {
        let mut pokemon = create_test_pokemon(
            Some("lum-berry".to_string()),
            200,
            200,
            Some(StatusCondition::Freeze),
        );
        let result = apply_item_effect("lum-berry", &mut pokemon, None, None);

        assert!(result.status_cured);
        assert!(pokemon.status_condition.is_none(), "Freeze debe ser curado");
        assert!(result.consumed);
    }

    #[test]
    fn test_lum_berry_no_effect_when_healthy() {
        let mut pokemon = create_test_pokemon(Some("lum-berry".to_string()), 200, 200, None);
        let result = apply_item_effect("lum-berry", &mut pokemon, None, None);

        assert!(!result.status_cured, "No debe indicar curación si no hay status");
        assert!(!result.consumed, "No debe consumirse si no hay nada que curar");
        assert!(pokemon.held_item.is_some(), "Item debe mantenerse");
    }
}

// ==================== WEAKNESS POLICY TESTS ====================

#[cfg(test)]
mod weakness_policy {
    use super::*;

    #[test]
    fn test_weakness_policy_boosts_stats() {
        let mut pokemon = create_test_pokemon(Some("weakness-policy".to_string()), 200, 200, None);
        let result = apply_item_effect("weakness-policy", &mut pokemon, None, None);

        assert_eq!(result.stat_boosts.len(), 2, "Debe dar 2 stat boosts");
        assert!(
            result.stat_boosts.contains(&("attack".to_string(), 2)),
            "Debe incluir +2 Attack"
        );
        assert!(
            result.stat_boosts.contains(&("sp_attack".to_string(), 2)),
            "Debe incluir +2 Sp. Attack"
        );
        assert!(result.consumed, "Weakness Policy debe consumirse");
        assert!(pokemon.held_item.is_none(), "Item debe removerse");
    }

    #[test]
    fn test_weakness_policy_is_one_time_use() {
        let mut pokemon = create_test_pokemon(Some("weakness-policy".to_string()), 200, 200, None);

        // Primera activación
        let result1 = apply_item_effect("weakness-policy", &mut pokemon, None, None);
        assert!(result1.consumed);
        assert!(pokemon.held_item.is_none());

        // Segunda activación (ya no tiene el item)
        let result2 = apply_item_effect("weakness-policy", &mut pokemon, None, None);
        assert_eq!(result2.stat_boosts.len(), 0, "No debe dar boosts sin el item");
    }
}

// ==================== INTEGRATION TESTS ====================

#[cfg(test)]
mod integration {
    use super::*;

    #[test]
    fn test_multiple_items_independently() {
        // Test que diferentes items funcionan correctamente cuando se usan en secuencia

        // Choice Band
        let mut poke1 = create_test_pokemon(Some("choice-band".to_string()), 200, 200, None);
        let r1 = apply_item_effect("choice-band", &mut poke1, Some("tackle"), None);
        assert_eq!(r1.damage_multiplier, 1.5);

        // Life Orb
        let mut poke2 = create_test_pokemon(Some("life-orb".to_string()), 200, 200, None);
        let r2 = apply_item_effect("life-orb", &mut poke2, Some("tackle"), Some(50));
        assert_eq!(r2.damage_multiplier, 1.3);
        assert_eq!(poke2.current_hp, 180);

        // Sitrus Berry
        let mut poke3 = create_test_pokemon(Some("sitrus-berry".to_string()), 100, 200, None);
        let _r3 = apply_item_effect("sitrus-berry", &mut poke3, None, None);
        assert_eq!(poke3.current_hp, 150);

        // Todos funcionan independientemente
        assert_eq!(poke1.current_hp, 200);
        assert_eq!(poke2.current_hp, 180);
        assert_eq!(poke3.current_hp, 150);
    }

    #[test]
    fn test_unknown_item_has_no_effect() {
        let mut pokemon = create_test_pokemon(Some("unknown-item".to_string()), 200, 200, None);
        let result = apply_item_effect("unknown-item", &mut pokemon, None, None);

        // Item desconocido no debe generar efectos
        assert_eq!(result.damage_multiplier, 1.0);
        assert!(!result.consumed);
        assert_eq!(result.stat_boosts.len(), 0);
        assert_eq!(result.healed_hp, 0);
        assert_eq!(pokemon.current_hp, 200);
    }
}
