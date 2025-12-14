//! Tests unitarios para el módulo de efectos de batalla
//! 
//! Este módulo contiene tests para verificar el correcto funcionamiento
//! de la aplicación de estados, cambios de stats, healing y otros efectos.

use rand::rngs::StdRng;
use rand::SeedableRng;
use crate::models::{MoveStatChange, StatusCondition};
use super::helpers::*;

#[cfg(test)]
mod status_tests {
    use super::*;

    #[test]
    fn test_apply_burn_status() {
        // Usa un movimiento con `ailment: "burn"`, `chance: 100`
        // Verifica que `defender.status_condition` cambie a `Some(Burn)`
        let mut attacker = create_mock_pokemon("charmander", vec!["Fire"], 100, 80);
        let mut defender = create_mock_pokemon("rattata", vec!["Normal"], 100, 60);
        
        // Crear movimiento con burn garantizado (100% chance)
        let burn_move = MoveBuilder::new("will-o-wisp", None, "status", "Fire")
            .with_ailment("burn", 100)
            .with_target("selected-pokemon")
            .build();
        
        let mut rng = StdRng::seed_from_u64(42);
        let mut context = create_battle_context(
            &mut attacker,
            &mut defender,
            &burn_move,
            &mut rng,
        );
        
        // Aplicar efectos (sin daño porque es movimiento de estado)
        context.apply_move_effects(0);
        
        // Verificar que el defensor tiene Burn (acceder a través del contexto)
        assert_eq!(
            context.defender.status_condition,
            Some(StatusCondition::Burn),
            "Defender should have Burn status after Will-O-Wisp"
        );
        
        // Verificar que se agregó un log
        assert!(
            context.logs.iter().any(|log| log.contains("quemó")),
            "Should have a log message about burning"
        );
    }

    #[test]
    fn test_burn_not_applied_if_already_has_status() {
        // Verifica que NO se aplique si ya tiene otro estado
        let mut attacker = create_mock_pokemon("charmander", vec!["Fire"], 100, 80);
        let mut defender = create_mock_pokemon("rattata", vec!["Normal"], 100, 60);
        
        // Aplicar un estado previo (Paralysis)
        defender.status_condition = Some(StatusCondition::Paralysis);
        let initial_status = defender.status_condition;
        
        // Crear movimiento con burn
        let burn_move = MoveBuilder::new("will-o-wisp", None, "status", "Fire")
            .with_ailment("burn", 100)
            .with_target("selected-pokemon")
            .build();
        
        let mut rng = StdRng::seed_from_u64(42);
        let mut context = create_battle_context(
            &mut attacker,
            &mut defender,
            &burn_move,
            &mut rng,
        );
        
        // Aplicar efectos
        context.apply_move_effects(0);
        
        // Verificar que el estado NO cambió (sigue siendo Paralysis)
        assert_eq!(
            context.defender.status_condition,
            initial_status,
            "Defender should keep existing status, not apply new Burn"
        );
    }

    #[test]
    fn test_burn_immune_fire_type() {
        // Verifica que los tipos Fire son inmunes a Burn
        let mut attacker = create_mock_pokemon("charmander", vec!["Fire"], 100, 80);
        let mut defender = create_mock_pokemon("charmander", vec!["Fire"], 100, 60);
        
        // Crear movimiento con burn
        let burn_move = MoveBuilder::new("will-o-wisp", None, "status", "Fire")
            .with_ailment("burn", 100)
            .with_target("selected-pokemon")
            .build();
        
        let mut rng = StdRng::seed_from_u64(42);
        let mut context = create_battle_context(
            &mut attacker,
            &mut defender,
            &burn_move,
            &mut rng,
        );
        
        // Aplicar efectos
        context.apply_move_effects(0);
        
        // Verificar que el defensor NO tiene Burn (es inmune)
        assert_eq!(
            context.defender.status_condition,
            None,
            "Fire type should be immune to Burn"
        );
    }
}

#[cfg(test)]
mod stat_changes_tests {
    use super::*;

    #[test]
    fn test_swords_dance_self_stat_change() {
        // Usa "Swords Dance" (`target: "user"`, `change: +2 Attack`)
        // Verifica que `attacker.battle_stages.attack` suba a 2
        let mut attacker = create_mock_pokemon("scyther", vec!["Bug", "Flying"], 100, 100);
        let mut defender = create_mock_pokemon("rattata", vec!["Normal"], 100, 60);
        
        // Crear Swords Dance: +2 Attack al usuario
        let swords_dance = MoveBuilder::new("swords-dance", None, "status", "Normal")
            .with_target("user")
            .with_stat_changes(vec![MoveStatChange {
                stat: "attack".to_string(),
                change: 2,
            }])
            .build();
        
        let mut rng = StdRng::seed_from_u64(42);
        let mut context = create_battle_context(
            &mut attacker,
            &mut defender,
            &swords_dance,
            &mut rng,
        );
        
        // Aplicar efectos
        context.apply_move_effects(0);
        
        // Verificar que el atacante tiene +2 Attack (acceder a través del contexto)
        assert!(
            context.attacker.battle_stages.is_some(),
            "Attacker should have battle_stages initialized"
        );
        
        if let Some(ref stages) = context.attacker.battle_stages {
            assert_eq!(
                stages.attack, 2,
                "Attacker should have +2 Attack after Swords Dance"
            );
        }
        
        // Verificar que se agregó un log
        assert!(
            context.logs.iter().any(|log| log.contains("ataque") && log.contains("subió")),
            "Should have a log message about attack increasing"
        );
    }

    #[test]
    fn test_growl_enemy_stat_change() {
        // Usa "Growl" (`target: "selected-pokemon"`, `change: -1 Attack`)
        // Verifica que `defender.battle_stages.attack` baje a -1
        let mut attacker = create_mock_pokemon("pikachu", vec!["Electric"], 100, 90);
        let mut defender = create_mock_pokemon("rattata", vec!["Normal"], 100, 60);
        
        // Crear Growl: -1 Attack al enemigo
        let growl = MoveBuilder::new("growl", None, "status", "Normal")
            .with_target("selected-pokemon")
            .with_stat_changes(vec![MoveStatChange {
                stat: "attack".to_string(),
                change: -1,
            }])
            .build();
        
        let mut rng = StdRng::seed_from_u64(42);
        let mut context = create_battle_context(
            &mut attacker,
            &mut defender,
            &growl,
            &mut rng,
        );
        
        // Aplicar efectos
        context.apply_move_effects(0);
        
        // Verificar que el defensor tiene -1 Attack (acceder a través del contexto)
        assert!(
            context.defender.battle_stages.is_some(),
            "Defender should have battle_stages initialized"
        );
        
        if let Some(ref stages) = context.defender.battle_stages {
            assert_eq!(
                stages.attack, -1,
                "Defender should have -1 Attack after Growl"
            );
        }
        
        // Verificar que se agregó un log
        assert!(
            context.logs.iter().any(|log| log.contains("ataque") && log.contains("bajó")),
            "Should have a log message about attack decreasing"
        );
    }

    #[test]
    fn test_stat_changes_clamp_to_limits() {
        // Verifica que los stat stages se limiten a -6 y +6
        let mut attacker = create_mock_pokemon("scyther", vec!["Bug", "Flying"], 100, 100);
        let mut defender = create_mock_pokemon("rattata", vec!["Normal"], 100, 60);
        
        // Inicializar battle_stages con +5 Attack
        attacker.init_battle_stages();
        if let Some(ref mut stages) = attacker.battle_stages {
            stages.attack = 5;
        }
        
        // Crear Swords Dance: +2 Attack (debería llegar a +6, no +7)
        let swords_dance = MoveBuilder::new("swords-dance", None, "status", "Normal")
            .with_target("user")
            .with_stat_changes(vec![MoveStatChange {
                stat: "attack".to_string(),
                change: 2,
            }])
            .build();
        
        let mut rng = StdRng::seed_from_u64(42);
        let mut context = create_battle_context(
            &mut attacker,
            &mut defender,
            &swords_dance,
            &mut rng,
        );
        
        // Aplicar efectos
        context.apply_move_effects(0);
        
        // Verificar que el atacante tiene +6 Attack (clamped, no +7)
        if let Some(ref stages) = context.attacker.battle_stages {
            assert_eq!(
                stages.attack, 6,
                "Attack should be clamped to +6, not +7"
            );
        }
    }
}

#[cfg(test)]
mod healing_tests {
    use super::*;

    #[test]
    fn test_recover_healing() {
        // Baja el HP del usuario a 10/100
        // Usa "Recover" (`healing: 50`)
        // Verifica que el HP suba correctamente (sin pasar el MaxHP)
        let mut attacker = create_mock_pokemon("chansey", vec!["Normal"], 100, 50);
        let mut defender = create_mock_pokemon("rattata", vec!["Normal"], 100, 60);
        
        // Bajar HP a 10
        attacker.current_hp = 10;
        let max_hp = attacker.base_computed_stats.hp;
        let initial_hp = attacker.current_hp;
        
        // Crear Recover: cura 50% del HP máximo
        let recover = MoveBuilder::new("recover", None, "status", "Normal")
            .with_target("user")
            .build();
        
        // Necesitamos modificar el meta directamente para healing
        let mut recover_move = recover;
        recover_move.meta.healing = 50; // 50% del HP máximo
        
        let mut rng = StdRng::seed_from_u64(42);
        let mut context = create_battle_context(
            &mut attacker,
            &mut defender,
            &recover_move,
            &mut rng,
        );
        
        // Aplicar efectos
        context.apply_move_effects(0);
        
        // Calcular HP esperado: 10 + (max_hp * 0.5), pero no más que max_hp
        let expected_heal = (max_hp as f32 * 0.5) as u16;
        let expected_hp = (initial_hp as u32 + expected_heal as u32).min(max_hp as u32) as u16;
        
        // Verificar que el HP subió correctamente (acceder a través del contexto)
        assert_eq!(
            context.attacker.current_hp, expected_hp,
            "HP should increase by 50% of max HP. Initial: {}, Max: {}, Expected: {}, Actual: {}",
            initial_hp,
            max_hp,
            expected_hp,
            context.attacker.current_hp
        );
        
        // Verificar que el HP no excedió el máximo
        assert!(
            context.attacker.current_hp <= max_hp,
            "HP should not exceed max HP. Current: {}, Max: {}",
            context.attacker.current_hp,
            max_hp
        );
        
        // Verificar que se agregó un log
        assert!(
            context.logs.iter().any(|log| log.contains("recuperó")),
            "Should have a log message about healing"
        );
    }

    #[test]
    fn test_recover_healing_at_max_hp() {
        // Verifica que Recover no haga nada si el Pokémon ya está al máximo
        let mut attacker = create_mock_pokemon("chansey", vec!["Normal"], 100, 50);
        let mut defender = create_mock_pokemon("rattata", vec!["Normal"], 100, 60);
        
        // HP al máximo
        let max_hp = attacker.base_computed_stats.hp;
        attacker.current_hp = max_hp;
        let initial_hp = attacker.current_hp;
        
        // Crear Recover
        let recover = MoveBuilder::new("recover", None, "status", "Normal")
            .with_target("user")
            .build();
        
        let mut recover_move = recover;
        recover_move.meta.healing = 50;
        
        let mut rng = StdRng::seed_from_u64(42);
        let mut context = create_battle_context(
            &mut attacker,
            &mut defender,
            &recover_move,
            &mut rng,
        );
        
        // Aplicar efectos
        context.apply_move_effects(0);
        
        // Verificar que el HP no cambió (ya estaba al máximo)
        assert_eq!(
            context.attacker.current_hp, initial_hp,
            "HP should not change when already at max. Current: {}, Max: {}",
            context.attacker.current_hp,
            max_hp
        );
        
        // No debería haber log de curación si no se curó nada
        // (aunque el código actual podría agregar un log, esto es aceptable)
    }

    #[test]
    fn test_recover_healing_partial() {
        // Verifica que Recover cure correctamente cuando el HP está cerca del máximo
        let mut attacker = create_mock_pokemon("chansey", vec!["Normal"], 100, 50);
        let mut defender = create_mock_pokemon("rattata", vec!["Normal"], 100, 60);
        
        // HP a 80 de 100 (max) - pero necesitamos asegurarnos de que max_hp sea al menos 100
        let max_hp = attacker.base_computed_stats.hp;
        // Asegurar que el HP inicial no exceda el máximo
        attacker.current_hp = 80.min(max_hp);
        let initial_hp = attacker.current_hp;
        
        // Crear Recover (50% = 50 HP)
        let recover = MoveBuilder::new("recover", None, "status", "Normal")
            .with_target("user")
            .build();
        
        let mut recover_move = recover;
        recover_move.meta.healing = 50;
        
        let mut rng = StdRng::seed_from_u64(42);
        let mut context = create_battle_context(
            &mut attacker,
            &mut defender,
            &recover_move,
            &mut rng,
        );
        
        // Aplicar efectos
        context.apply_move_effects(0);
        
        // Debería curar 50% del HP máximo, pero solo hasta el máximo
        // Calcular HP esperado: min(initial_hp + heal_amount, max_hp)
        let heal_amount = (max_hp as f32 * 0.5) as u16;
        let expected_hp = (initial_hp as u32 + heal_amount as u32).min(max_hp as u32) as u16;
        
        assert_eq!(
            context.attacker.current_hp, expected_hp,
            "HP should be healed to max, not beyond. Initial: {}, Max: {}, Heal: {}, Expected: {}, Actual: {}",
            initial_hp,
            max_hp,
            heal_amount,
            expected_hp,
            context.attacker.current_hp
        );
        
        // Verificar que el HP no excedió el máximo
        assert!(
            context.attacker.current_hp <= max_hp,
            "HP should not exceed max HP. Current: {}, Max: {}",
            context.attacker.current_hp,
            max_hp
        );
    }
}

