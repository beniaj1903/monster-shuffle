//! Tests unitarios para el módulo de mecánicas de batalla
//! 
//! Este módulo contiene tests para verificar el correcto funcionamiento
//! de los cálculos de daño, efectividad de tipos, STAB, críticos y stat stages.

use rand::rngs::StdRng;
use rand::SeedableRng;
use crate::battle::mechanics::calculate_damage;
use crate::models::StatStages;
use super::helpers::*;

/// Helper para calcular el daño promedio de múltiples ejecuciones
/// Útil para reducir el impacto del factor aleatorio (0.85-1.0)
fn calculate_average_damage(
    attacker: &mut crate::models::PokemonInstance,
    defender: &mut crate::models::PokemonInstance,
    move_data: &crate::models::MoveData,
    is_critical: bool,
    iterations: usize,
) -> f32 {
    let mut total_damage = 0u64;
    
    for _ in 0..iterations {
        let mut rng = StdRng::seed_from_u64(42);
        let (damage, _, _) = calculate_damage(
            attacker,
            defender,
            move_data,
            is_critical,
            &mut rng,
            None,
            None,
            None,
        );
        total_damage += damage as u64;
    }
    
    total_damage as f32 / iterations as f32
}

#[cfg(test)]
mod effectiveness_tests {
    use super::*;

    #[test]
    fn test_fire_vs_grass_super_effective() {
        // Fuego vs Planta -> Debe ser Super Effective (> 1.0)
        let mut attacker = create_mock_pokemon("charmander", vec!["Fire"], 100, 80);
        let mut defender = create_mock_pokemon("bulbasaur", vec!["Grass"], 100, 60);
        
        let fire_move = create_dummy_move("ember", Some(40), "special", "Fire");
        
        // Calcular daño promedio para reducir variación aleatoria
        let avg_damage = calculate_average_damage(&mut attacker, &mut defender, &fire_move, false, 100);
        
        // Crear un movimiento Normal con el mismo poder para comparar
        let normal_move = create_dummy_move("tackle", Some(40), "physical", "Normal");
        let avg_normal_damage = calculate_average_damage(&mut attacker, &mut defender, &normal_move, false, 100);
        
        // El daño de Fuego vs Planta debe ser significativamente mayor (2x efectividad)
        // Considerando el factor aleatorio, esperamos al menos 1.5x más daño
        assert!(
            avg_damage > avg_normal_damage * 1.5,
            "Fire vs Grass should be super effective. Fire damage: {}, Normal damage: {}",
            avg_damage,
            avg_normal_damage
        );
    }

    #[test]
    fn test_fire_vs_water_not_very_effective() {
        // Fuego vs Agua -> Debe ser Not Very Effective (< 1.0)
        // Usamos un atacante Normal para que ninguno de los movimientos tenga STAB
        let mut attacker = create_mock_pokemon("rattata", vec!["Normal"], 100, 80);
        let mut defender = create_mock_pokemon("squirtle", vec!["Water"], 100, 60);
        
        let fire_move = create_dummy_move("ember", Some(40), "special", "Fire");
        
        // Calcular daño promedio
        let avg_damage = calculate_average_damage(&mut attacker, &mut defender, &fire_move, false, 100);
        
        // Crear un movimiento Normal ESPECIAL con el mismo poder para comparar (mismo tipo de daño)
        let normal_move = create_dummy_move("swift", Some(40), "special", "Normal");
        let avg_normal_damage = calculate_average_damage(&mut attacker, &mut defender, &normal_move, false, 100);
        
        // El daño de Fuego vs Agua debe ser significativamente menor (0.5x efectividad)
        // Considerando el factor aleatorio y que ambos tienen el mismo STAB (ninguno), 
        // esperamos menos de 0.6x del daño normal
        assert!(
            avg_damage < avg_normal_damage * 0.6,
            "Fire vs Water should be not very effective. Fire damage: {}, Normal damage: {}, Ratio: {}",
            avg_damage,
            avg_normal_damage,
            avg_damage / avg_normal_damage
        );
    }

    #[test]
    fn test_ground_vs_flying_immune() {
        // Tierra vs Volador -> Debe ser Inmune (0.0)
        let attacker = create_mock_pokemon("sandshrew", vec!["Ground"], 100, 80);
        let defender = create_mock_pokemon("pidgey", vec!["Flying"], 100, 60);
        
        let ground_move = create_dummy_move("earthquake", Some(100), "physical", "Ground");
        
        let mut rng = StdRng::seed_from_u64(42);
        let (damage, _effectiveness_msg, _) = calculate_damage(
            &attacker,
            &defender,
            &ground_move,
            false,
            &mut rng,
            None,
            None,
            None,
        );
        
        // El daño debe ser exactamente 0 (inmune)
        assert_eq!(
            damage, 0,
            "Ground vs Flying should be immune. Damage: {}",
            damage
        );
    }
}

#[cfg(test)]
mod stab_tests {
    use super::*;

    #[test]
    fn test_stab_fire_move_on_fire_pokemon() {
        // Charmander (Fuego) usando Ember (Fuego) debe hacer más daño que Scratch (Normal)
        // con la misma potencia
        let mut attacker = create_mock_pokemon("charmander", vec!["Fire"], 100, 80);
        let mut defender = create_mock_pokemon("rattata", vec!["Normal"], 100, 60);
        
        // Movimiento de tipo Fuego (mismo tipo que el atacante) - tiene STAB
        let fire_move = create_dummy_move("ember", Some(40), "special", "Fire");
        let avg_fire_damage = calculate_average_damage(&mut attacker, &mut defender, &fire_move, false, 100);
        
        // Movimiento Normal (diferente tipo) - sin STAB
        let normal_move = create_dummy_move("scratch", Some(40), "physical", "Normal");
        let avg_normal_damage = calculate_average_damage(&mut attacker, &mut defender, &normal_move, false, 100);
        
        // El daño con STAB debe ser aproximadamente 1.5x mayor
        // Considerando variación aleatoria, esperamos al menos 1.3x más daño
        assert!(
            avg_fire_damage > avg_normal_damage * 1.3,
            "STAB Fire move should deal more damage. Fire (STAB): {}, Normal (no STAB): {}",
            avg_fire_damage,
            avg_normal_damage
        );
    }
}

#[cfg(test)]
mod critical_hit_tests {
    use super::*;

    #[test]
    fn test_critical_hit_damage_multiplier() {
        // Un crítico debe hacer aproximadamente 1.5x más daño
        let mut attacker = create_mock_pokemon("pikachu", vec!["Electric"], 100, 90);
        let mut defender = create_mock_pokemon("charmander", vec!["Fire"], 100, 80);
        
        let move_data = create_dummy_move("thunderbolt", Some(90), "special", "Electric");
        
        // Calcular daño promedio sin crítico
        let avg_normal_damage = calculate_average_damage(&mut attacker, &mut defender, &move_data, false, 100);
        
        // Calcular daño promedio con crítico forzado
        let avg_critical_damage = calculate_average_damage(&mut attacker, &mut defender, &move_data, true, 100);
        
        // El daño crítico debe ser aproximadamente 1.5x mayor
        // Considerando variación aleatoria, esperamos entre 1.4x y 1.6x
        let ratio = avg_critical_damage / avg_normal_damage;
        assert!(
            ratio >= 1.4 && ratio <= 1.6,
            "Critical hit should deal ~1.5x damage. Normal: {}, Critical: {}, Ratio: {}",
            avg_normal_damage,
            avg_critical_damage,
            ratio
        );
    }
}

#[cfg(test)]
mod stat_stages_tests {
    use super::*;

    #[test]
    fn test_attack_stage_plus_two_doubles_damage() {
        // Un Pokémon con attack_stage: +2 debe hacer aproximadamente el doble de daño físico
        // que uno con stage: 0
        let mut attacker_base = create_mock_pokemon("machop", vec!["Fighting"], 100, 70);
        let mut attacker_boosted = create_mock_pokemon("machop", vec!["Fighting"], 100, 70);
        let mut defender = create_mock_pokemon("rattata", vec!["Normal"], 100, 60);
        
        // Configurar battle_stages para el atacante con boost
        attacker_boosted.battle_stages = Some(StatStages {
            attack: 2,  // +2 attack stage
            defense: 0,
            special_attack: 0,
            special_defense: 0,
            speed: 0,
            accuracy: 0,
            evasion: 0,
        });
        
        let move_data = create_dummy_move("karate-chop", Some(50), "physical", "Fighting");
        
        // Calcular daño promedio sin boost
        let avg_base_damage = calculate_average_damage(&mut attacker_base, &mut defender, &move_data, false, 100);
        
        // Calcular daño promedio con +2 attack
        let avg_boosted_damage = calculate_average_damage(&mut attacker_boosted, &mut defender, &move_data, false, 100);
        
        // Con +2 attack stage, el multiplicador es (2 + 2) / 2 = 2.0x
        // Esperamos aproximadamente el doble de daño
        let ratio = avg_boosted_damage / avg_base_damage;
        assert!(
            ratio >= 1.8 && ratio <= 2.2,
            "+2 Attack stage should double damage. Base: {}, Boosted: {}, Ratio: {}",
            avg_base_damage,
            avg_boosted_damage,
            ratio
        );
    }

    #[test]
    fn test_defense_stage_minus_two_takes_more_damage() {
        // Un Pokémon con defense_stage: -2 debe recibir más daño
        let mut attacker = create_mock_pokemon("machop", vec!["Fighting"], 100, 70);
        let mut defender_base = create_mock_pokemon("rattata", vec!["Normal"], 100, 60);
        let mut defender_lowered = create_mock_pokemon("rattata", vec!["Normal"], 100, 60);
        
        // Configurar battle_stages para el defensor con debuff
        defender_lowered.battle_stages = Some(StatStages {
            attack: 0,
            defense: -2,  // -2 defense stage
            special_attack: 0,
            special_defense: 0,
            speed: 0,
            accuracy: 0,
            evasion: 0,
        });
        
        let move_data = create_dummy_move("tackle", Some(40), "physical", "Normal");
        
        // Calcular daño promedio sin debuff
        let avg_base_damage = calculate_average_damage(&mut attacker, &mut defender_base, &move_data, false, 100);
        
        // Calcular daño promedio con -2 defense
        let avg_lowered_damage = calculate_average_damage(&mut attacker, &mut defender_lowered, &move_data, false, 100);
        
        // Con -2 defense stage, el multiplicador es 2 / (2 - (-2)) = 2 / 4 = 0.5x para la defensa
        // Esto significa que el daño recibido es 2.0x mayor
        let ratio = avg_lowered_damage / avg_base_damage;
        assert!(
            ratio >= 1.8 && ratio <= 2.2,
            "-2 Defense stage should double damage taken. Base: {}, Lowered: {}, Ratio: {}",
            avg_base_damage,
            avg_lowered_damage,
            ratio
        );
    }

    #[test]
    fn test_special_attack_stage_plus_one() {
        // Un Pokémon con special_attack_stage: +1 debe hacer más daño especial
        let mut attacker_base = create_mock_pokemon("abra", vec!["Psychic"], 100, 90);
        let mut attacker_boosted = create_mock_pokemon("abra", vec!["Psychic"], 100, 90);
        let mut defender = create_mock_pokemon("rattata", vec!["Normal"], 100, 60);
        
        // Configurar battle_stages para el atacante con boost
        attacker_boosted.battle_stages = Some(StatStages {
            attack: 0,
            defense: 0,
            special_attack: 1,  // +1 special attack stage
            special_defense: 0,
            speed: 0,
            accuracy: 0,
            evasion: 0,
        });
        
        let move_data = create_dummy_move("psychic", Some(90), "special", "Psychic");
        
        // Calcular daño promedio sin boost
        let avg_base_damage = calculate_average_damage(&mut attacker_base, &mut defender, &move_data, false, 100);
        
        // Calcular daño promedio con +1 special attack
        let avg_boosted_damage = calculate_average_damage(&mut attacker_boosted, &mut defender, &move_data, false, 100);
        
        // Con +1 special attack stage, el multiplicador es (2 + 1) / 2 = 1.5x
        let ratio = avg_boosted_damage / avg_base_damage;
        assert!(
            ratio >= 1.4 && ratio <= 1.6,
            "+1 Special Attack stage should increase damage by 1.5x. Base: {}, Boosted: {}, Ratio: {}",
            avg_base_damage,
            avg_boosted_damage,
            ratio
        );
    }
}

