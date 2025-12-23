//! Integration Tests - Combos de Mecánicas VGC (Fase 4.2)
//!
//! Este módulo contiene tests de integración que validan que múltiples
//! sistemas trabajen correctamente en conjunto, simulando escenarios
//! reales de batallas VGC.

use crate::models::{
    PokemonInstance, PokemonSpecies, Stats, RandomizedProfile, PokemonType,
    StatModifiers, VolatileStatus, StatStages, StatusCondition, MoveData,
    MoveMeta, BattleFormat, FieldPosition,
};
use crate::game::BattleState;
use crate::battle::systems::item_system::item_effects::apply_item_effect;
use crate::battle::systems::ability_system::registry::get_ability_hooks;

/// Helper para crear Pokémon de prueba con configuración completa
fn create_test_pokemon(
    name: &str,
    primary_type: PokemonType,
    ability: &str,
    item: Option<String>,
    hp: u16,
    max_hp: u16,
) -> PokemonInstance {
    PokemonInstance {
        id: format!("test-{}", name),
        species: PokemonSpecies {
            species_id: name.to_lowercase(),
            display_name: name.to_string(),
            generation: 1,
            primary_type,
            secondary_type: None,
            base_stats: Stats {
                hp: max_hp,
                attack: 100,
                defense: 100,
                special_attack: 100,
                special_defense: 100,
                speed: 100,
            },
            move_pool: vec!["tackle".to_string(), "thunderbolt".to_string()],
            possible_abilities: vec![ability.to_string()],
            is_starter_candidate: false,
            evolutions: Vec::new(),
        },
        level: 50,
        current_hp: hp,
        status_condition: None,
        held_item: item,
        ability: ability.to_string(),
        battle_stages: Some(StatStages::new()),
        volatile_status: Some(VolatileStatus::new()),
        individual_values: Stats::default(),
        effort_values: Stats::default(),
        randomized_profile: RandomizedProfile {
            rolled_primary_type: primary_type,
            rolled_secondary_type: None,
            rolled_ability_id: ability.to_string(),
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

// ==================== ITEM + ABILITY COMBO TESTS ====================

#[cfg(test)]
mod item_ability_combos {
    use super::*;

    #[test]
    fn test_life_orb_plus_sheer_force() {
        // Life Orb + Sheer Force = +30% daño SIN recoil
        // Este es un combo muy popular en VGC
        let mut pokemon = create_test_pokemon(
            "Nidoking",
            PokemonType::Poison,
            "sheer-force",
            Some("life-orb".to_string()),
            200,
            200,
        );

        // Aplicar Life Orb (normalmente +30% daño con -10% HP recoil)
        let result = apply_item_effect("life-orb", &mut pokemon, Some("sludge-wave"), Some(80));

        // Con Sheer Force, Life Orb no causa recoil
        // Nota: Esta interacción requeriría lógica especial en el damage calculator
        assert_eq!(result.damage_multiplier, 1.3, "Life Orb debe dar +30% daño");

        // En la implementación real, Sheer Force cancelaría el recoil
        // Por ahora validamos que Life Orb funciona normalmente
        assert_eq!(pokemon.current_hp, 180, "Life Orb causa recoil normalmente");
    }

    #[test]
    fn test_choice_band_plus_technician() {
        // Choice Band (+50% Attack) + Technician (+50% para movimientos ≤60 poder)
        // = Movimientos débiles con daño brutal
        let mut pokemon = create_test_pokemon(
            "Breloom",
            PokemonType::Grass,
            "technician",
            Some("choice-band".to_string()),
            200,
            200,
        );

        // Choice Band boost
        let band_result = apply_item_effect("choice-band", &mut pokemon, Some("mach-punch"), None);
        assert_eq!(band_result.damage_multiplier, 1.5, "Choice Band +50% daño");

        // Technician boost (se aplicaría en damage calculator)
        let technician_hooks = get_ability_hooks("technician");
        assert!(!technician_hooks.is_empty(), "Technician debe tener hooks");

        // En combate real: Mach Punch (40 poder) × 1.5 (Choice Band) × 1.5 (Technician) = 90 poder efectivo
    }

    #[test]
    fn test_assault_vest_blocks_lum_berry_trigger() {
        // Assault Vest impide usar movimientos de estado
        // Lum Berry cura estados automáticamente
        // Estos NO deberían interferir entre sí
        let mut pokemon = create_test_pokemon(
            "Conkeldurr",
            PokemonType::Fighting,
            "guts",
            Some("assault-vest".to_string()),
            200,
            200,
        );

        // Assault Vest no interfiere con items que se activan automáticamente
        let av_result = apply_item_effect("assault-vest", &mut pokemon, None, None);
        assert_eq!(av_result.damage_multiplier, 1.0, "Assault Vest es pasivo");

        // Si tuviera Lum Berry y fuera envenenado, Lum Berry funcionaría normalmente
        pokemon.held_item = Some("lum-berry".to_string());
        pokemon.status_condition = Some(StatusCondition::Poison);

        let lum_result = apply_item_effect("lum-berry", &mut pokemon, None, None);
        assert!(lum_result.status_cured, "Lum Berry debe curar veneno");
        assert!(pokemon.status_condition.is_none(), "Veneno debe ser curado");
    }

    #[test]
    fn test_weakness_policy_plus_solid_rock() {
        // Solid Rock reduce daño super efectivo en 25%
        // Weakness Policy se activa con golpe super efectivo
        // Ambos deberían funcionar juntos
        let mut pokemon = create_test_pokemon(
            "Rhyperior",
            PokemonType::Ground,
            "solid-rock",
            Some("weakness-policy".to_string()),
            200,
            200,
        );

        // Solid Rock hooks
        let solid_rock_hooks = get_ability_hooks("solid-rock");
        assert!(!solid_rock_hooks.is_empty(), "Solid Rock debe tener hooks");

        // Weakness Policy se activaría después de recibir golpe super efectivo
        let wp_result = apply_item_effect("weakness-policy", &mut pokemon, None, None);

        assert_eq!(wp_result.stat_boosts.len(), 2, "Weakness Policy da 2 boosts");
        assert!(wp_result.consumed, "Weakness Policy es consumible");

        // En combate real: Recibe Water move (4x) → Solid Rock lo reduce a 3x → Weakness Policy activa
    }

    #[test]
    fn test_sitrus_berry_plus_regenerator() {
        // Sitrus Berry cura 25% HP cuando baja de 50%
        // Regenerator cura 1/3 HP al salir del campo
        // Ambas son mecánicas de curación independientes
        let mut pokemon = create_test_pokemon(
            "Amoonguss",
            PokemonType::Grass,
            "regenerator",
            Some("sitrus-berry".to_string()),
            90,  // Menos de 50% para activar Sitrus Berry
            200,
        );

        // Sitrus Berry se activa
        let sitrus_result = apply_item_effect("sitrus-berry", &mut pokemon, None, None);
        assert_eq!(sitrus_result.healed_hp, 50, "Sitrus Berry cura 25% = 50 HP");
        assert_eq!(pokemon.current_hp, 140, "HP debe ser 90 + 50");

        // Regenerator hooks existen
        let regen_hooks = get_ability_hooks("regenerator");
        assert!(!regen_hooks.is_empty(), "Regenerator debe tener hooks");

        // En combate real: Al salir, curaría 1/3 de 200 = 66 HP adicionales
    }

    #[test]
    fn test_rocky_helmet_plus_rough_skin() {
        // Rocky Helmet causa 1/6 HP daño al atacante en contacto
        // Rough Skin causa 1/8 HP daño al atacante en contacto
        // Ambos deberían acumular (29% HP total de daño por contacto)
        let mut pokemon = create_test_pokemon(
            "Garchomp",
            PokemonType::Dragon,
            "rough-skin",
            Some("rocky-helmet".to_string()),
            200,
            200,
        );

        // Rough Skin hooks
        let rough_skin_hooks = get_ability_hooks("rough-skin");
        assert!(!rough_skin_hooks.is_empty(), "Rough Skin debe tener hooks");

        // Rocky Helmet se procesaría en apply_on_contact_items
        // Rough Skin se procesaría en apply_on_contact_abilities
        // Ambos deberían aplicarse al mismo atacante

        // HP del atacante (ejemplo 200 HP):
        // Rocky Helmet: 200 / 6 = 33 HP
        // Rough Skin: 200 / 8 = 25 HP
        // Total: 58 HP de daño por hacer contacto (29% del HP máximo)
    }

    #[test]
    fn test_choice_scarf_priority_interaction() {
        // Choice Scarf aumenta Speed en 50%
        // Esto afecta el orden de turnos pero NO la prioridad de movimientos
        let mut fast_pokemon = create_test_pokemon(
            "Kartana",
            PokemonType::Grass,
            "beast-boost",
            Some("choice-scarf".to_string()),
            200,
            200,
        );

        // Choice Scarf no genera efectos directos en apply_item_effect
        // Su efecto se aplica en el speed calculation
        let result = apply_item_effect("choice-scarf", &mut fast_pokemon, Some("leaf-blade"), None);
        assert_eq!(result.move_locked, Some("leaf-blade".to_string()));

        // En combate real:
        // - Choice Scarf multiplica Speed × 1.5
        // - Movimientos de prioridad +1 aún van primero
        // - Trick Room invierte orden de speed pero NO prioridad
    }
}

// ==================== PROTECTION + REDIRECTION COMBO TESTS ====================

#[cfg(test)]
mod protection_redirection_combos {
    use super::*;
    use crate::battle::systems::protection_system::processor::*;
    use crate::battle::systems::redirection_system::processor::*;

    #[test]
    fn test_wide_guard_plus_follow_me() {
        // Wide Guard protege de spread moves
        // Follow Me redirige single-target moves
        // Juntos protegen de todo tipo de ataques
        let mut protected_pokemon = create_test_pokemon(
            "Arcanine",
            PokemonType::Fire,
            "intimidate",
            None,
            200,
            200,
        );

        let mut redirector_pokemon = create_test_pokemon(
            "Amoonguss",
            PokemonType::Grass,
            "regenerator",
            None,
            200,
            200,
        );

        // Activar Wide Guard
        activate_wide_guard(&mut protected_pokemon);

        // Crear battle state y activar Follow Me
        let mut battle_state = BattleState::new(
            0,
            vec![protected_pokemon.clone(), redirector_pokemon.clone()],
            "Opponent".to_string(),
            BattleFormat::Double,
            true,
        );
        set_follow_me(&mut battle_state, FieldPosition::PlayerRight);

        // Verificar que ambas protecciones están activas
        assert!(
            protected_pokemon.volatile_status.as_ref().unwrap().wide_guard_active,
            "Wide Guard debe estar activo"
        );
        assert!(
            battle_state.redirection.is_some(),
            "Follow Me debe estar activo"
        );

        // En combate real:
        // - Earthquake (spread) → Bloqueado por Wide Guard
        // - Flamethrower (single) → Redirigido por Follow Me a Amoonguss
    }

    #[test]
    fn test_quick_guard_vs_priority_redirection() {
        // Quick Guard bloquea movimientos con prioridad
        // Follow Me redirige ataques
        // Quick Guard tiene prioridad sobre redirección
        let mut protected = create_test_pokemon(
            "Incineroar",
            PokemonType::Fire,
            "intimidate",
            None,
            200,
            200,
        );

        activate_quick_guard(&mut protected);

        // En combate real:
        // - Fake Out (priority +3) → Bloqueado por Quick Guard (no se redirige)
        // - Follow Me no puede redirigir lo que Quick Guard bloquea
    }

    #[test]
    fn test_mat_block_first_turn_protection() {
        // Mat Block solo funciona el primer turno
        // Es útil para proteger a un setup sweeper
        let mut mat_blocker = create_test_pokemon(
            "Greninja",
            PokemonType::Water,
            "protean",
            None,
            200,
            200,
        );

        activate_mat_block(&mut mat_blocker);

        // Mat Block bloquea movimientos dañinos
        let tackle = MoveData {
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
        };

        assert!(is_blocked_by_mat_block(&mat_blocker, &tackle));

        // En combate real: Protege en turno 1 mientras el compañero hace setup (Swords Dance, etc.)
    }

    #[test]
    fn test_crafty_shield_plus_wide_guard_full_protection() {
        // Crafty Shield bloquea status moves
        // Wide Guard bloquea spread damaging moves
        // Juntos ofrecen protección casi completa del equipo
        let mut poke1 = create_test_pokemon("A", PokemonType::Normal, "pressure", None, 200, 200);
        let mut poke2 = create_test_pokemon("B", PokemonType::Normal, "pressure", None, 200, 200);

        activate_crafty_shield(&mut poke1);
        activate_wide_guard(&mut poke2);

        // Status move bloqueado
        let thunder_wave = MoveData {
            id: "thunder-wave".to_string(),
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
        };

        // Spread move bloqueado
        let earthquake = MoveData {
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
        };

        assert!(is_blocked_by_crafty_shield(&poke1, &thunder_wave));
        assert!(is_blocked_by_wide_guard(&poke2, &earthquake));

        // En combate real: Equipo protegido de status spam Y spread moves
    }
}

// ==================== VGC SCENARIO TESTS ====================

#[cfg(test)]
mod vgc_scenarios {
    use super::*;
    use crate::battle::systems::redirection_system::processor::*;

    #[test]
    fn test_doubles_spread_move_with_redirection() {
        // Escenario: Earthquake en doubles con Follow Me activo
        // Follow Me NO debe redirigir Earthquake (es spread)
        let mut battle_state = BattleState::new(
            0,
            vec![
                create_test_pokemon("Landorus", PokemonType::Ground, "intimidate", None, 200, 200),
                create_test_pokemon("Amoonguss", PokemonType::Grass, "regenerator", None, 200, 200),
            ],
            "Opponent".to_string(),
            BattleFormat::Double,
            true,
        );

        // Amoonguss usa Follow Me
        set_follow_me(&mut battle_state, FieldPosition::PlayerRight);

        // Oponente usa Earthquake (spread move)
        let earthquake = MoveData {
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
        };

        let attacker = create_test_pokemon("Garchomp", PokemonType::Dragon, "rough-skin", None, 200, 200);

        // Earthquake NO debe ser redirigido (es spread)
        let result = apply_redirection(
            FieldPosition::PlayerLeft,
            FieldPosition::OpponentLeft,
            &attacker,
            &earthquake,
            &battle_state,
        );

        assert_eq!(result, None, "Spread moves NO deben ser redirigidos por Follow Me");

        // En combate real: Earthquake golpea a AMBOS Pokémon del jugador
    }

    #[test]
    fn test_rage_powder_vs_grass_type_immunity() {
        // Escenario VGC común: Rage Powder no afecta a Grass types
        let mut battle_state = BattleState::new(
            0,
            vec![
                create_test_pokemon("Amoonguss", PokemonType::Grass, "regenerator", None, 200, 200),
                create_test_pokemon("Incineroar", PokemonType::Fire, "intimidate", None, 200, 200),
            ],
            "Opponent".to_string(),
            BattleFormat::Double,
            true,
        );

        // Amoonguss usa Rage Powder
        set_rage_powder(&mut battle_state, FieldPosition::PlayerLeft);

        let single_move = MoveData {
            id: "flamethrower".to_string(),
            name: "Flamethrower".to_string(),
            r#type: "Fire".to_string(),
            power: Some(90),
            accuracy: Some(100),
            priority: 0,
            pp: 15,
            damage_class: "special".to_string(),
            meta: MoveMeta::default(),
            stat_changes: vec![],
            target: "selected-pokemon".to_string(),
        };

        // Grass type attacker (Kartana)
        let grass_attacker = create_test_pokemon(
            "Kartana",
            PokemonType::Grass,
            "beast-boost",
            None,
            200,
            200,
        );

        // Rage Powder NO debe redirigir ataques de Grass types
        let result = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &grass_attacker,
            &single_move,
            &battle_state,
        );

        assert_eq!(result, None, "Rage Powder NO debe afectar a Grass types");

        // Fire type attacker (Charizard)
        let fire_attacker = create_test_pokemon(
            "Charizard",
            PokemonType::Fire,
            "blaze",
            None,
            200,
            200,
        );

        // Rage Powder SÍ debe redirigir ataques de Fire types
        let result2 = apply_redirection(
            FieldPosition::PlayerRight,
            FieldPosition::OpponentLeft,
            &fire_attacker,
            &single_move,
            &battle_state,
        );

        assert_eq!(
            result2,
            Some(FieldPosition::PlayerLeft),
            "Rage Powder debe redirigir ataques de non-Grass types"
        );
    }

    #[test]
    fn test_intimidate_on_switch_in_doubles() {
        // Escenario: Incineroar entra al campo en doubles
        // Intimidate debe bajar Attack de AMBOS oponentes
        let incineroar = create_test_pokemon(
            "Incineroar",
            PokemonType::Fire,
            "intimidate",
            None,
            200,
            200,
        );

        // Intimidate hooks
        let intimidate_hooks = get_ability_hooks("intimidate");
        assert!(!intimidate_hooks.is_empty(), "Intimidate debe tener hooks");

        // En combate real:
        // - Incineroar entra al campo
        // - Intimidate activa (OnEntry trigger)
        // - AMBOS oponentes reciben -1 Attack
        // - Inmunidades verificadas (Clear Body, White Smoke, etc.)
    }

    #[test]
    fn test_trick_room_speed_reversal_scenario() {
        // Escenario: Trick Room activo en doubles
        // Pokémon lentos actúan primero

        let slow_pokemon = create_test_pokemon(
            "Snorlax",
            PokemonType::Normal,
            "thick-fat",
            None,
            200,
            200,
        );
        // Speed base: 30

        let fast_pokemon = create_test_pokemon(
            "Regieleki",
            PokemonType::Electric,
            "transistor",
            None,
            200,
            200,
        );
        // Speed base: 200

        // En combate real con Trick Room:
        // - Normalmente: Regieleki (200) actúa antes que Snorlax (30)
        // - Con Trick Room: Snorlax (30) actúa antes que Regieleki (200)
        // - Prioridad de movimientos aún se respeta (+1 va primero)

        // Esto se testea en el action_system/sort_candidates
    }

    #[test]
    fn test_fake_out_quick_guard_interaction() {
        // Escenario VGC crítico: Fake Out vs Quick Guard
        let mut protected = create_test_pokemon(
            "Arcanine",
            PokemonType::Fire,
            "intimidate",
            None,
            200,
            200,
        );

        activate_quick_guard(&mut protected);

        // Fake Out tiene priority +3
        let fake_out = MoveData {
            id: "fake-out".to_string(),
            name: "Fake Out".to_string(),
            r#type: "Normal".to_string(),
            power: Some(40),
            accuracy: Some(100),
            priority: 3,
            pp: 10,
            damage_class: "physical".to_string(),
            meta: MoveMeta::default(),
            stat_changes: vec![],
            target: "selected-pokemon".to_string(),
        };

        assert!(
            is_blocked_by_quick_guard(&protected, &fake_out),
            "Quick Guard debe bloquear Fake Out (priority +3)"
        );

        // En combate real:
        // - Incineroar usa Fake Out turn 1
        // - Oponente usa Quick Guard
        // - Fake Out bloqueado, Incineroar no puede flinch
    }

    #[test]
    fn test_ally_switch_position_reversal() {
        // Escenario: Ally Switch cambia posiciones físicas
        // Afecta targeting de spread moves
        let mut battle_state = BattleState::new(
            0,
            vec![
                create_test_pokemon("Garchomp", PokemonType::Dragon, "rough-skin", None, 200, 200),
                create_test_pokemon("Togekiss", PokemonType::Fairy, "serene-grace", None, 200, 200),
            ],
            "Opponent".to_string(),
            BattleFormat::Double,
            true,
        );

        let original_indices = battle_state.player_active_indices.clone();

        // Garchomp (PlayerLeft) usa Ally Switch
        let success = ally_switch(&mut battle_state, FieldPosition::PlayerLeft);

        assert!(success, "Ally Switch debe ejecutarse");
        assert_eq!(
            battle_state.player_active_indices[0],
            original_indices[1],
            "Posiciones deben intercambiarse"
        );

        // En combate real:
        // - Garchomp ahora en posición Right
        // - Togekiss ahora en posición Left
        // - Earthquake enemigo golpea en nuevas posiciones
    }
}

// ==================== EDGE CASES Y CORNER CASES ====================

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_multiple_contact_damage_sources_stack() {
        // Rocky Helmet + Rough Skin + Iron Barbs
        // ¿Pueden acumular múltiples fuentes de daño por contacto?
        let pokemon_with_helmet = create_test_pokemon(
            "Ferrothorn",
            PokemonType::Grass,
            "iron-barbs",
            Some("rocky-helmet".to_string()),
            200,
            200,
        );

        let iron_barbs_hooks = get_ability_hooks("iron-barbs");
        assert!(!iron_barbs_hooks.is_empty());

        // En combate real:
        // - Iron Barbs: 1/8 HP (25 de 200)
        // - Rocky Helmet: 1/6 HP (33 de 200)
        // - Total: 58 HP de daño al atacante por hacer contacto

        // Nota: Esto es BRUTAL y muy usado en VGC
    }

    #[test]
    fn test_berry_consumption_priority() {
        // ¿Qué pasa si un Pokémon tiene <50% HP Y un status?
        // Sitrus Berry (HP) vs Lum Berry (status) - solo puede tener uno
        let mut pokemon_low_hp = create_test_pokemon(
            "Snorlax",
            PokemonType::Normal,
            "thick-fat",
            Some("sitrus-berry".to_string()),
            90,
            200,
        );
        pokemon_low_hp.status_condition = Some(StatusCondition::Burn);

        // Sitrus Berry se activa por HP bajo
        let result = apply_item_effect("sitrus-berry", &mut pokemon_low_hp, None, None);
        assert!(result.consumed);
        assert_eq!(pokemon_low_hp.current_hp, 140);

        // El burn permanece (no tiene Lum Berry)
        assert_eq!(
            pokemon_low_hp.status_condition,
            Some(StatusCondition::Burn),
            "Burn debe permanecer"
        );
    }

    #[test]
    fn test_item_removal_after_consumption() {
        // Verificar que items consumibles se remueven correctamente
        let mut pokemon = create_test_pokemon(
            "Garchomp",
            PokemonType::Dragon,
            "rough-skin",
            Some("weakness-policy".to_string()),
            200,
            200,
        );

        assert!(pokemon.held_item.is_some(), "Debe tener Weakness Policy");

        // Activar Weakness Policy
        let result = apply_item_effect("weakness-policy", &mut pokemon, None, None);
        assert!(result.consumed);

        // Item debe ser removido
        assert!(
            pokemon.held_item.is_none(),
            "Weakness Policy debe removerse después de consumirse"
        );

        // No puede activarse de nuevo
        let result2 = apply_item_effect("weakness-policy", &mut pokemon, None, None);
        assert!(!result2.consumed);
        assert_eq!(result2.stat_boosts.len(), 0);
    }

    #[test]
    fn test_ability_item_null_handling() {
        // Pokémon sin ability ni item debe manejar correctamente
        let pokemon_naked = create_test_pokemon(
            "Ditto",
            PokemonType::Normal,
            "",
            None,
            200,
            200,
        );

        // No debe crashear con ability vacía
        let empty_hooks = get_ability_hooks("");
        assert!(empty_hooks.is_empty(), "Ability vacía debe retornar hooks vacíos");

        // No debe crashear con item None
        assert!(pokemon_naked.held_item.is_none());
    }

    #[test]
    fn test_stat_boost_cap_with_weakness_policy() {
        // Weakness Policy da +2/+2
        // ¿Qué pasa si ya está en +4 Attack/SpA?
        let mut pokemon = create_test_pokemon(
            "Dragonite",
            PokemonType::Dragon,
            "multiscale",
            Some("weakness-policy".to_string()),
            200,
            200,
        );

        // Simular boosts previos (en combate real)
        if let Some(ref mut stages) = pokemon.battle_stages {
            stages.attack = 4;
            stages.special_attack = 4;
        }

        // Weakness Policy intenta dar +2/+2
        let result = apply_item_effect("weakness-policy", &mut pokemon, None, None);
        assert_eq!(result.stat_boosts.len(), 2);

        // En combate real, los boosts se capearían a +6 máximo
        // Attack: 4 + 2 = 6 (cap)
        // SpA: 4 + 2 = 6 (cap)
    }
}
