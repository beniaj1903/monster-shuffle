//! Triggers que activan los efectos de items
//!
//! Define cuándo y bajo qué condiciones se activan los items

use crate::models::PokemonInstance;

/// Define los diferentes momentos/condiciones en que un item puede activarse
#[derive(Debug, Clone, PartialEq)]
pub enum ItemTrigger {
    /// Antes de calcular el daño de un movimiento usado por este Pokémon
    BeforeDamageDealt {
        move_data: String, // ID del movimiento
        is_first_use: bool, // Para Choice items (lock después del primer uso)
    },

    /// Después de calcular el daño de un movimiento usado por este Pokémon
    AfterDamageDealt {
        damage_dealt: u16,
        move_data: String,
    },

    /// Cuando este Pokémon recibe daño
    OnDamageTaken {
        damage: u16,
        attacker_id: String,
        is_super_effective: bool,
    },

    /// Cuando se aplica un status condition a este Pokémon
    OnStatusApplied {
        status: String,
    },

    /// Al final de cada turno
    EndOfTurn,

    /// Cuando el HP cae por debajo de un threshold
    OnHPThreshold {
        current_hp: u16,
        max_hp: u16,
        threshold: f32, // 0.5 = 50%, etc.
    },

    /// Cuando el Pokémon intenta usar un movimiento de status
    OnStatusMoveAttempt {
        move_data: String,
    },
}

/// Verifica si un item debería activarse según el trigger
pub fn check_item_trigger(
    item_id: &str,
    trigger: &ItemTrigger,
    pokemon: &PokemonInstance,
) -> bool {
    // Verificar que el Pokémon tenga el item equipado
    if let Some(held_item) = &pokemon.held_item {
        if held_item != item_id {
            return false;
        }
    } else {
        return false;
    }

    match item_id {
        // Choice Items: se activan antes de calcular daño
        "choice-band" | "choice-specs" | "choice-scarf" => {
            matches!(trigger, ItemTrigger::BeforeDamageDealt { .. })
        }

        // Life Orb: se activa después de causar daño
        "life-orb" => {
            matches!(trigger, ItemTrigger::AfterDamageDealt { damage_dealt, .. } if *damage_dealt > 0)
        }

        // Assault Vest: se activa cuando se intenta usar un status move
        "assault-vest" => {
            matches!(trigger, ItemTrigger::OnStatusMoveAttempt { .. })
        }

        // Sitrus Berry: se activa cuando HP < 50%
        "sitrus-berry" => {
            if let ItemTrigger::OnHPThreshold { current_hp, max_hp, .. } = trigger {
                (*current_hp as f32 / *max_hp as f32) <= 0.5
            } else {
                false
            }
        }

        // Lum Berry: se activa cuando se aplica un status
        "lum-berry" => {
            matches!(trigger, ItemTrigger::OnStatusApplied { .. })
        }

        // Weakness Policy: se activa al recibir golpe super efectivo
        "weakness-policy" => {
            matches!(trigger, ItemTrigger::OnDamageTaken { is_super_effective: true, .. })
        }

        _ => false,
    }
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
            current_hp: 100,
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
    fn test_choice_band_trigger() {
        let pokemon = create_test_pokemon(Some("choice-band".to_string()));
        let trigger = ItemTrigger::BeforeDamageDealt {
            move_data: "tackle".to_string(),
            is_first_use: true,
        };

        assert!(check_item_trigger("choice-band", &trigger, &pokemon));
    }

    #[test]
    fn test_life_orb_trigger() {
        let pokemon = create_test_pokemon(Some("life-orb".to_string()));
        let trigger = ItemTrigger::AfterDamageDealt {
            damage_dealt: 50,
            move_data: "tackle".to_string(),
        };

        assert!(check_item_trigger("life-orb", &trigger, &pokemon));
    }

    #[test]
    fn test_sitrus_berry_trigger_activates_below_50() {
        let pokemon = create_test_pokemon(Some("sitrus-berry".to_string()));
        let trigger = ItemTrigger::OnHPThreshold {
            current_hp: 90,
            max_hp: 200,
            threshold: 0.5,
        };

        assert!(check_item_trigger("sitrus-berry", &trigger, &pokemon));
    }

    #[test]
    fn test_sitrus_berry_trigger_does_not_activate_above_50() {
        let pokemon = create_test_pokemon(Some("sitrus-berry".to_string()));
        let trigger = ItemTrigger::OnHPThreshold {
            current_hp: 120,
            max_hp: 200,
            threshold: 0.5,
        };

        assert!(!check_item_trigger("sitrus-berry", &trigger, &pokemon));
    }

    #[test]
    fn test_lum_berry_trigger() {
        let pokemon = create_test_pokemon(Some("lum-berry".to_string()));
        let trigger = ItemTrigger::OnStatusApplied {
            status: "burn".to_string(),
        };

        assert!(check_item_trigger("lum-berry", &trigger, &pokemon));
    }

    #[test]
    fn test_weakness_policy_trigger() {
        let pokemon = create_test_pokemon(Some("weakness-policy".to_string()));
        let trigger = ItemTrigger::OnDamageTaken {
            damage: 50,
            attacker_id: "enemy-1".to_string(),
            is_super_effective: true,
        };

        assert!(check_item_trigger("weakness-policy", &trigger, &pokemon));
    }

    #[test]
    fn test_no_item_equipped() {
        let pokemon = create_test_pokemon(None);
        let trigger = ItemTrigger::BeforeDamageDealt {
            move_data: "tackle".to_string(),
            is_first_use: true,
        };

        assert!(!check_item_trigger("choice-band", &trigger, &pokemon));
    }
}
