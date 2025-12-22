//! Procesador de items que integra con el pipeline de batalla
//!
//! Este módulo provee funciones de alto nivel para procesar items
//! en diferentes puntos del pipeline de batalla

use crate::models::PokemonInstance;
use super::item_effects::{apply_item_effect, ItemEffectResult};
use super::item_triggers::{check_item_trigger, ItemTrigger};

/// Procesador principal de items
pub struct ItemProcessor;

impl ItemProcessor {
    /// Procesa items antes de calcular daño
    pub fn process_before_damage(
        pokemon: &mut PokemonInstance,
        move_id: &str,
        is_first_use: bool,
    ) -> ItemEffectResult {
        if let Some(ref item_id) = pokemon.held_item.clone() {
            let trigger = ItemTrigger::BeforeDamageDealt {
                move_data: move_id.to_string(),
                is_first_use,
            };

            if check_item_trigger(item_id, &trigger, pokemon) {
                return apply_item_effect(item_id, pokemon, Some(move_id), None);
            }
        }

        ItemEffectResult::default()
    }

    /// Procesa items después de causar daño
    pub fn process_after_damage(
        pokemon: &mut PokemonInstance,
        move_id: &str,
        damage_dealt: u16,
    ) -> ItemEffectResult {
        if let Some(ref item_id) = pokemon.held_item.clone() {
            let trigger = ItemTrigger::AfterDamageDealt {
                damage_dealt,
                move_data: move_id.to_string(),
            };

            if check_item_trigger(item_id, &trigger, pokemon) {
                return apply_item_effect(item_id, pokemon, Some(move_id), Some(damage_dealt));
            }
        }

        ItemEffectResult::default()
    }

    /// Procesa items al recibir daño
    pub fn process_on_damage_taken(
        pokemon: &mut PokemonInstance,
        damage: u16,
        attacker_id: &str,
        is_super_effective: bool,
    ) -> ItemEffectResult {
        if let Some(ref item_id) = pokemon.held_item.clone() {
            let trigger = ItemTrigger::OnDamageTaken {
                damage,
                attacker_id: attacker_id.to_string(),
                is_super_effective,
            };

            if check_item_trigger(item_id, &trigger, pokemon) {
                return apply_item_effect(item_id, pokemon, None, None);
            }
        }

        ItemEffectResult::default()
    }

    /// Procesa items cuando se aplica un status
    pub fn process_on_status_applied(
        pokemon: &mut PokemonInstance,
        status: &str,
    ) -> ItemEffectResult {
        if let Some(ref item_id) = pokemon.held_item.clone() {
            let trigger = ItemTrigger::OnStatusApplied {
                status: status.to_string(),
            };

            if check_item_trigger(item_id, &trigger, pokemon) {
                return apply_item_effect(item_id, pokemon, None, None);
            }
        }

        ItemEffectResult::default()
    }

    /// Procesa items al final del turno
    pub fn process_end_of_turn(
        pokemon: &mut PokemonInstance,
    ) -> ItemEffectResult {
        // Verificar threshold de HP para berries
        let current_hp = pokemon.current_hp;
        let max_hp = pokemon.base_computed_stats.hp;

        if let Some(ref item_id) = pokemon.held_item.clone() {
            let trigger = ItemTrigger::OnHPThreshold {
                current_hp,
                max_hp,
                threshold: 0.5,
            };

            if check_item_trigger(item_id, &trigger, pokemon) {
                return apply_item_effect(item_id, pokemon, None, None);
            }
        }

        ItemEffectResult::default()
    }

    /// Verifica si un item bloquea el uso de un movimiento de status
    pub fn blocks_status_move(
        pokemon: &PokemonInstance,
        _move_id: &str,
    ) -> bool {
        if let Some(ref item_id) = pokemon.held_item {
            if item_id == "assault-vest" {
                // TODO: Verificar que el movimiento sea de status consultando MoveData
                // Por ahora asumimos que se verificará en el validation system
                return true;
            }
        }
        false
    }

    /// Obtiene el multiplicador de daño de un item (para Choice items y Life Orb)
    pub fn get_damage_multiplier(
        pokemon: &PokemonInstance,
        move_category: &str, // "physical" o "special"
    ) -> f32 {
        if let Some(ref item_id) = pokemon.held_item {
            match item_id.as_str() {
                "choice-band" if move_category == "physical" => 1.5,
                "choice-specs" if move_category == "special" => 1.5,
                "life-orb" => 1.3,
                _ => 1.0,
            }
        } else {
            1.0
        }
    }

    /// Obtiene el multiplicador de Speed de Choice Scarf
    pub fn get_speed_multiplier(pokemon: &PokemonInstance) -> f32 {
        if let Some(ref item_id) = pokemon.held_item {
            if item_id == "choice-scarf" {
                return 1.5;
            }
        }
        1.0
    }

    /// Obtiene el multiplicador de Sp. Defense de Assault Vest
    pub fn get_sp_defense_multiplier(pokemon: &PokemonInstance) -> f32 {
        if let Some(ref item_id) = pokemon.held_item {
            if item_id == "assault-vest" {
                return 1.5;
            }
        }
        1.0
    }
}

/// Función de conveniencia para procesar items antes de calcular daño
pub fn process_items_before_damage(
    pokemon: &mut PokemonInstance,
    move_id: &str,
    is_first_use: bool,
) -> ItemEffectResult {
    ItemProcessor::process_before_damage(pokemon, move_id, is_first_use)
}

/// Función de conveniencia para procesar items después de causar daño
pub fn process_items_after_damage(
    pokemon: &mut PokemonInstance,
    move_id: &str,
    damage_dealt: u16,
) -> ItemEffectResult {
    ItemProcessor::process_after_damage(pokemon, move_id, damage_dealt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PokemonSpecies, Stats, StatStages};

    fn create_test_pokemon(item: Option<String>) -> PokemonInstance {
        PokemonInstance {
            id: "test-1".to_string(),
            species: PokemonSpecies {
                id: "pikachu".to_string(),
                display_name: "Pikachu".to_string(),
                types: vec!["Electric".to_string()],
                base_stats: Default::default(),
            },
            level: 50,
            current_hp: 200,
            status_condition: None,
            held_item: item,
            ability: "static".to_string(),
            nature: "adamant".to_string(),
            evs: Default::default(),
            ivs: Default::default(),
            learned_moves: vec![],
            base_computed_stats: Stats {
                hp: 200,
                attack: 100,
                defense: 100,
                sp_attack: 100,
                sp_defense: 100,
                speed: 100,
            },
            stat_stages: Some(StatStages::default()),
            volatile_status: None,
        }
    }

    #[test]
    fn test_choice_band_damage_multiplier() {
        let pokemon = create_test_pokemon(Some("choice-band".to_string()));
        let multiplier = ItemProcessor::get_damage_multiplier(&pokemon, "physical");

        assert_eq!(multiplier, 1.5);
    }

    #[test]
    fn test_choice_specs_damage_multiplier() {
        let pokemon = create_test_pokemon(Some("choice-specs".to_string()));
        let multiplier = ItemProcessor::get_damage_multiplier(&pokemon, "special");

        assert_eq!(multiplier, 1.5);
    }

    #[test]
    fn test_life_orb_damage_multiplier() {
        let pokemon = create_test_pokemon(Some("life-orb".to_string()));
        let multiplier = ItemProcessor::get_damage_multiplier(&pokemon, "physical");

        assert_eq!(multiplier, 1.3);
    }

    #[test]
    fn test_choice_scarf_speed_multiplier() {
        let pokemon = create_test_pokemon(Some("choice-scarf".to_string()));
        let multiplier = ItemProcessor::get_speed_multiplier(&pokemon);

        assert_eq!(multiplier, 1.5);
    }

    #[test]
    fn test_assault_vest_sp_defense_multiplier() {
        let pokemon = create_test_pokemon(Some("assault-vest".to_string()));
        let multiplier = ItemProcessor::get_sp_defense_multiplier(&pokemon);

        assert_eq!(multiplier, 1.5);
    }

    #[test]
    fn test_no_item_multipliers() {
        let pokemon = create_test_pokemon(None);

        assert_eq!(ItemProcessor::get_damage_multiplier(&pokemon, "physical"), 1.0);
        assert_eq!(ItemProcessor::get_speed_multiplier(&pokemon), 1.0);
        assert_eq!(ItemProcessor::get_sp_defense_multiplier(&pokemon), 1.0);
    }

    #[test]
    fn test_process_before_damage_choice_band() {
        let mut pokemon = create_test_pokemon(Some("choice-band".to_string()));
        let result = process_items_before_damage(&mut pokemon, "tackle", true);

        assert_eq!(result.damage_multiplier, 1.5);
        assert_eq!(result.move_locked, Some("tackle".to_string()));
    }

    #[test]
    fn test_process_after_damage_life_orb() {
        let mut pokemon = create_test_pokemon(Some("life-orb".to_string()));
        let result = process_items_after_damage(&mut pokemon, "tackle", 50);

        assert_eq!(result.damage_multiplier, 1.3);
        assert_eq!(result.recoil_damage, 20); // 10% of 200 HP
        assert_eq!(pokemon.current_hp, 180);
    }
}
