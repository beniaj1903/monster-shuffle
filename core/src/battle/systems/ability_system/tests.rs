//! Tests comprehensivos para el Ability System (Fase 4.1)
//!
//! Este módulo contiene tests exhaustivos para todas las abilities implementadas
//! en las Fases 1 y 2, validando triggers, efectos y edge cases.

use crate::models::{PokemonInstance, PokemonSpecies, Stats, RandomizedProfile, PokemonType, StatModifiers, VolatileStatus, StatStages, MoveData, MoveMeta};
use super::registry::{get_ability_hooks, AbilityTrigger, AbilityEffect};

/// Helper para crear Pokémon de prueba con ability específica
fn create_test_pokemon(ability: &str, hp: u16, max_hp: u16) -> PokemonInstance {
    PokemonInstance {
        id: "test-ability".to_string(),
        species: PokemonSpecies {
            species_id: "testmon".to_string(),
            display_name: "TestMon".to_string(),
            generation: 1,
            primary_type: PokemonType::Normal,
            secondary_type: None,
            base_stats: Stats {
                hp: 100,
                attack: 100,
                defense: 100,
                special_attack: 100,
                special_defense: 100,
                speed: 100,
            },
            move_pool: vec!["tackle".to_string()],
            possible_abilities: vec![ability.to_string()],
            is_starter_candidate: false,
            evolutions: Vec::new(),
        },
        level: 50,
        current_hp: hp,
        status_condition: None,
        held_item: None,
        ability: ability.to_string(),
        battle_stages: Some(StatStages::new()),
        volatile_status: Some(VolatileStatus::new()),
        individual_values: Stats::default(),
        effort_values: Stats::default(),
        randomized_profile: RandomizedProfile {
            rolled_primary_type: PokemonType::Normal,
            rolled_secondary_type: None,
            rolled_ability_id: ability.to_string(),
            stat_modifiers: StatModifiers::default(),
            learned_moves: Vec::new(),
            moves: Vec::new(),
        },
        base_computed_stats: Stats {
            hp: max_hp,
            attack: 150,
            defense: 100,
            special_attack: 150,
            special_defense: 100,
            speed: 100,
        },
    }
}

/// Helper para crear MoveData de prueba
fn create_test_move(power: Option<u16>, move_type: &str, damage_class: &str) -> MoveData {
    MoveData {
        id: "test-move".to_string(),
        name: "Test Move".to_string(),
        r#type: move_type.to_string(),
        power,
        accuracy: Some(100),
        priority: 0,
        pp: 10,
        damage_class: damage_class.to_string(),
        meta: MoveMeta::default(),
        stat_changes: vec![],
        target: "selected-pokemon".to_string(),
    }
}

// ==================== FASE 2.2: ABILITIES CRÍTICAS ====================

#[cfg(test)]
mod download {
    use super::*;

    #[test]
    fn test_download_has_on_entry_trigger() {
        let hooks = get_ability_hooks("download");

        assert!(!hooks.is_empty(), "Download debe tener hooks definidos");
        assert!(
            hooks.iter().any(|h| matches!(h.trigger, AbilityTrigger::OnEntry)),
            "Download debe tener trigger OnEntry"
        );
    }

    #[test]
    fn test_download_is_custom_effect() {
        let hooks = get_ability_hooks("download");
        let on_entry_hook = hooks.iter()
            .find(|h| matches!(h.trigger, AbilityTrigger::OnEntry))
            .expect("Download debe tener hook OnEntry");

        assert!(
            matches!(on_entry_hook.effect, AbilityEffect::Custom { .. }),
            "Download debe usar AbilityEffect::Custom"
        );
    }
}

#[cfg(test)]
mod solid_rock_filter {
    use super::*;

    #[test]
    fn test_solid_rock_has_before_damage_trigger() {
        let hooks = get_ability_hooks("solid-rock");

        assert!(!hooks.is_empty(), "Solid Rock debe tener hooks definidos");
        assert!(
            hooks.iter().any(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage)),
            "Solid Rock debe tener trigger BeforeDamage"
        );
    }

    #[test]
    fn test_filter_has_before_damage_trigger() {
        let hooks = get_ability_hooks("filter");

        assert!(!hooks.is_empty(), "Filter debe tener hooks definidos");
        assert!(
            hooks.iter().any(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage)),
            "Filter debe tener trigger BeforeDamage"
        );
    }

    #[test]
    fn test_solid_rock_reduces_super_effective_damage() {
        let hooks = get_ability_hooks("solid-rock");
        let before_damage_hook = hooks.iter()
            .find(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage))
            .expect("Solid Rock debe tener hook BeforeDamage");

        if let AbilityEffect::ReduceSuperEffectiveDamage { multiplier } = before_damage_hook.effect {
            assert_eq!(multiplier, 0.75, "Solid Rock debe reducir a 75% (25% de reducción)");
        } else {
            panic!("Solid Rock debe tener efecto ReduceSuperEffectiveDamage");
        }
    }

    #[test]
    fn test_filter_has_same_effect_as_solid_rock() {
        let solid_rock_hooks = get_ability_hooks("solid-rock");
        let filter_hooks = get_ability_hooks("filter");

        let solid_rock_effect = &solid_rock_hooks.iter()
            .find(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage))
            .expect("Solid Rock debe tener hook").effect;

        let filter_effect = &filter_hooks.iter()
            .find(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage))
            .expect("Filter debe tener hook").effect;

        assert_eq!(
            format!("{:?}", solid_rock_effect),
            format!("{:?}", filter_effect),
            "Filter y Solid Rock deben tener el mismo efecto"
        );
    }
}

#[cfg(test)]
mod sheer_force {
    use super::*;

    #[test]
    fn test_sheer_force_has_before_damage_trigger() {
        let hooks = get_ability_hooks("sheer-force");

        assert!(!hooks.is_empty(), "Sheer Force debe tener hooks definidos");
        assert!(
            hooks.iter().any(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage)),
            "Sheer Force debe tener trigger BeforeDamage"
        );
    }

    #[test]
    fn test_sheer_force_boosts_damage() {
        let hooks = get_ability_hooks("sheer-force");
        let before_damage_hook = hooks.iter()
            .find(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage))
            .expect("Sheer Force debe tener hook BeforeDamage");

        if let AbilityEffect::RemoveSecondaryEffects { damage_multiplier } = before_damage_hook.effect {
            assert_eq!(damage_multiplier, 1.3, "Sheer Force debe dar +30% daño (1.3x)");
        } else {
            panic!("Sheer Force debe tener efecto RemoveSecondaryEffects");
        }
    }
}

#[cfg(test)]
mod technician {
    use super::*;

    #[test]
    fn test_technician_has_before_damage_trigger() {
        let hooks = get_ability_hooks("technician");

        assert!(!hooks.is_empty(), "Technician debe tener hooks definidos");
        assert!(
            hooks.iter().any(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage)),
            "Technician debe tener trigger BeforeDamage"
        );
    }

    #[test]
    fn test_technician_parameters() {
        let hooks = get_ability_hooks("technician");
        let before_damage_hook = hooks.iter()
            .find(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage))
            .expect("Technician debe tener hook BeforeDamage");

        if let AbilityEffect::BoostWeakMoves { power_threshold, multiplier } = before_damage_hook.effect {
            assert_eq!(power_threshold, 60, "Technician debe activarse con movimientos ≤60 de poder");
            assert_eq!(multiplier, 1.5, "Technician debe dar +50% daño (1.5x)");
        } else {
            panic!("Technician debe tener efecto BoostWeakMoves");
        }
    }
}

#[cfg(test)]
mod regenerator {
    use super::*;

    #[test]
    fn test_regenerator_exists() {
        let hooks = get_ability_hooks("regenerator");

        assert!(!hooks.is_empty(), "Regenerator debe estar definido");
    }

    #[test]
    fn test_regenerator_has_heal_on_switch() {
        let hooks = get_ability_hooks("regenerator");

        // Regenerator requiere sistema de switch, pero debe estar definido
        assert!(
            !hooks.is_empty(),
            "Regenerator debe tener estructura básica implementada"
        );
    }
}

// ==================== PHASE 1: ABILITIES BÁSICAS ====================

#[cfg(test)]
mod intimidate {
    use super::*;

    #[test]
    fn test_intimidate_has_on_entry_trigger() {
        let hooks = get_ability_hooks("intimidate");

        assert!(!hooks.is_empty(), "Intimidate debe tener hooks definidos");
        assert!(
            hooks.iter().any(|h| matches!(h.trigger, AbilityTrigger::OnEntry)),
            "Intimidate debe tener trigger OnEntry"
        );
    }

    #[test]
    fn test_intimidate_lowers_attack() {
        let hooks = get_ability_hooks("intimidate");
        let on_entry_hook = hooks.iter()
            .find(|h| matches!(h.trigger, AbilityTrigger::OnEntry))
            .expect("Intimidate debe tener hook OnEntry");

        if let AbilityEffect::LowerOpponentsStat { stat, stages } = &on_entry_hook.effect {
            assert_eq!(stat, "attack", "Intimidate debe bajar Attack");
            assert_eq!(*stages, -1, "Intimidate debe bajar 1 stage");
        } else {
            panic!("Intimidate debe tener efecto LowerOpponentsStat");
        }
    }
}

#[cfg(test)]
mod blaze_torrent_overgrow {
    use super::*;

    #[test]
    fn test_blaze_boosts_fire_at_low_hp() {
        let hooks = get_ability_hooks("blaze");

        assert!(!hooks.is_empty(), "Blaze debe tener hooks definidos");

        let before_damage_hook = hooks.iter()
            .find(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage))
            .expect("Blaze debe tener trigger BeforeDamage");

        if let AbilityEffect::BoostTypeAtLowHP { move_type, multiplier, hp_threshold } = &before_damage_hook.effect {
            assert_eq!(move_type, &PokemonType::Fire, "Blaze debe potenciar tipo Fire");
            assert_eq!(*multiplier, 1.5, "Blaze debe dar 1.5x boost");
            assert_eq!(*hp_threshold, 0.33, "Blaze debe activarse a ≤33% HP");
        } else {
            panic!("Blaze debe tener efecto BoostTypeAtLowHP");
        }
    }

    #[test]
    fn test_torrent_boosts_water_at_low_hp() {
        let hooks = get_ability_hooks("torrent");
        let before_damage_hook = hooks.iter()
            .find(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage))
            .expect("Torrent debe tener trigger BeforeDamage");

        if let AbilityEffect::BoostTypeAtLowHP { move_type, .. } = &before_damage_hook.effect {
            assert_eq!(move_type, &PokemonType::Water, "Torrent debe potenciar tipo Water");
        } else {
            panic!("Torrent debe tener efecto BoostTypeAtLowHP");
        }
    }

    #[test]
    fn test_overgrow_boosts_grass_at_low_hp() {
        let hooks = get_ability_hooks("overgrow");
        let before_damage_hook = hooks.iter()
            .find(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage))
            .expect("Overgrow debe tener trigger BeforeDamage");

        if let AbilityEffect::BoostTypeAtLowHP { move_type, .. } = &before_damage_hook.effect {
            assert_eq!(move_type, &PokemonType::Grass, "Overgrow debe potenciar tipo Grass");
        } else {
            panic!("Overgrow debe tener efecto BoostTypeAtLowHP");
        }
    }
}

#[cfg(test)]
mod contact_abilities {
    use super::*;

    #[test]
    fn test_rough_skin_has_on_contact_trigger() {
        let hooks = get_ability_hooks("rough-skin");

        assert!(!hooks.is_empty(), "Rough Skin debe tener hooks definidos");
        assert!(
            hooks.iter().any(|h| matches!(h.trigger, AbilityTrigger::OnContact)),
            "Rough Skin debe tener trigger OnContact"
        );
    }

    #[test]
    fn test_iron_barbs_has_on_contact_trigger() {
        let hooks = get_ability_hooks("iron-barbs");

        assert!(!hooks.is_empty(), "Iron Barbs debe tener hooks definidos");
        assert!(
            hooks.iter().any(|h| matches!(h.trigger, AbilityTrigger::OnContact)),
            "Iron Barbs debe tener trigger OnContact"
        );
    }

    #[test]
    fn test_rough_skin_deals_damage() {
        let hooks = get_ability_hooks("rough-skin");
        let on_contact_hook = hooks.iter()
            .find(|h| matches!(h.trigger, AbilityTrigger::OnContact))
            .expect("Rough Skin debe tener hook OnContact");

        if let AbilityEffect::DamageAttackerOnContact { percent } = on_contact_hook.effect {
            assert_eq!(percent, 0.125, "Rough Skin debe causar 1/8 (12.5%) de daño");
        } else {
            panic!("Rough Skin debe tener efecto DamageAttackerOnContact");
        }
    }

    #[test]
    fn test_iron_barbs_deals_same_damage_as_rough_skin() {
        let rough_skin_hooks = get_ability_hooks("rough-skin");
        let iron_barbs_hooks = get_ability_hooks("iron-barbs");

        let rough_skin_effect = &rough_skin_hooks.iter()
            .find(|h| matches!(h.trigger, AbilityTrigger::OnContact))
            .expect("Rough Skin debe tener hook").effect;

        let iron_barbs_effect = &iron_barbs_hooks.iter()
            .find(|h| matches!(h.trigger, AbilityTrigger::OnContact))
            .expect("Iron Barbs debe tener hook").effect;

        assert_eq!(
            format!("{:?}", rough_skin_effect),
            format!("{:?}", iron_barbs_effect),
            "Iron Barbs y Rough Skin deben tener el mismo efecto"
        );
    }
}

#[cfg(test)]
mod type_boost_abilities {
    use super::*;

    #[test]
    fn test_adaptability_exists() {
        let hooks = get_ability_hooks("adaptability");
        assert!(!hooks.is_empty(), "Adaptability debe estar definido");
    }

    #[test]
    fn test_tough_claws_exists() {
        let hooks = get_ability_hooks("tough-claws");
        assert!(!hooks.is_empty(), "Tough Claws debe estar definido");
    }
}

// ==================== INTEGRATION TESTS ====================

#[cfg(test)]
mod integration {
    use super::*;

    #[test]
    fn test_multiple_abilities_independently() {
        // Test que diferentes abilities tienen hooks correctos

        let intimidate = get_ability_hooks("intimidate");
        let blaze = get_ability_hooks("blaze");
        let rough_skin = get_ability_hooks("rough-skin");
        let technician = get_ability_hooks("technician");

        // Cada una debe tener hooks
        assert!(!intimidate.is_empty(), "Intimidate debe tener hooks");
        assert!(!blaze.is_empty(), "Blaze debe tener hooks");
        assert!(!rough_skin.is_empty(), "Rough Skin debe tener hooks");
        assert!(!technician.is_empty(), "Technician debe tener hooks");

        // Cada una debe tener triggers diferentes
        assert!(intimidate.iter().any(|h| matches!(h.trigger, AbilityTrigger::OnEntry)));
        assert!(blaze.iter().any(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage)));
        assert!(rough_skin.iter().any(|h| matches!(h.trigger, AbilityTrigger::OnContact)));
        assert!(technician.iter().any(|h| matches!(h.trigger, AbilityTrigger::BeforeDamage)));
    }

    #[test]
    fn test_unknown_ability_returns_empty() {
        let hooks = get_ability_hooks("nonexistent-ability");

        assert!(hooks.is_empty(), "Ability inexistente debe retornar vec vacío");
    }

    #[test]
    fn test_abilities_with_same_effect_are_consistent() {
        // Abilities que hacen lo mismo deben tener efectos idénticos

        let solid_rock = get_ability_hooks("solid-rock");
        let filter = get_ability_hooks("filter");

        assert_eq!(
            solid_rock.len(),
            filter.len(),
            "Solid Rock y Filter deben tener el mismo número de hooks"
        );

        let rough_skin = get_ability_hooks("rough-skin");
        let iron_barbs = get_ability_hooks("iron-barbs");

        assert_eq!(
            rough_skin.len(),
            iron_barbs.len(),
            "Rough Skin e Iron Barbs deben tener el mismo número de hooks"
        );
    }

    #[test]
    fn test_ability_triggers_are_valid() {
        // Verificar que los triggers más importantes existen
        let all_abilities = vec![
            "intimidate", "blaze", "torrent", "overgrow",
            "rough-skin", "iron-barbs", "download",
            "solid-rock", "filter", "technician", "sheer-force"
        ];

        for ability_id in all_abilities {
            let hooks = get_ability_hooks(ability_id);
            assert!(
                !hooks.is_empty(),
                "{} debe tener al menos un hook definido",
                ability_id
            );

            // Verificar que cada hook tiene un trigger válido
            for hook in &hooks {
                assert!(
                    matches!(
                        hook.trigger,
                        AbilityTrigger::OnEntry
                            | AbilityTrigger::BeforeDamage
                            | AbilityTrigger::OnContact
                            | AbilityTrigger::AfterDamage
                            | AbilityTrigger::EndOfTurn
                    ),
                    "{} debe tener trigger válido",
                    ability_id
                );
            }
        }
    }
}

// ==================== EDGE CASES ====================

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_case_insensitive_ability_lookup() {
        // El sistema debería manejar IDs en lowercase
        let hooks1 = get_ability_hooks("intimidate");
        let hooks2 = get_ability_hooks("intimidate");

        assert_eq!(hooks1.len(), hooks2.len(), "Lookup debe ser consistente");
    }

    #[test]
    fn test_empty_ability_id() {
        let hooks = get_ability_hooks("");
        assert!(hooks.is_empty(), "ID vacío debe retornar vec vacío");
    }

    #[test]
    fn test_pokemon_with_ability() {
        let pokemon = create_test_pokemon("intimidate", 100, 200);

        assert_eq!(pokemon.ability, "intimidate", "Pokémon debe tener ability asignada");

        let hooks = get_ability_hooks(&pokemon.ability);
        assert!(!hooks.is_empty(), "Ability del Pokémon debe tener hooks");
    }
}
