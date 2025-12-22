//! Efectos que producen los items cuando se activan
//!
//! Define qué hace cada item cuando su trigger se cumple

use crate::models::{PokemonInstance, StatusCondition};

/// Resultado de aplicar un efecto de item
#[derive(Debug, Clone)]
pub struct ItemEffectResult {
    /// Logs generados por el efecto
    pub logs: Vec<String>,

    /// Si el item se consumió (berries de un solo uso)
    pub consumed: bool,

    /// Modificadores aplicados
    pub damage_multiplier: f32,
    pub stat_boosts: Vec<(String, i8)>, // (stat_name, stages)
    pub healed_hp: u16,
    pub recoil_damage: u16,
    pub status_cured: bool,
    pub move_locked: Option<String>, // Para Choice items
}

impl Default for ItemEffectResult {
    fn default() -> Self {
        Self {
            logs: Vec::new(),
            consumed: false,
            damage_multiplier: 1.0,
            stat_boosts: Vec::new(),
            healed_hp: 0,
            recoil_damage: 0,
            status_cured: false,
            move_locked: None,
        }
    }
}

/// Define los efectos que puede producir un item
#[derive(Debug, Clone, PartialEq)]
pub enum ItemEffect {
    /// Multiplica el daño de un movimiento
    BoostDamage { multiplier: f32 },

    /// Aumenta una stat en stages
    BoostStat { stat: String, stages: i8 },

    /// Bloquea al Pokémon a usar solo un movimiento
    LockMove { move_id: String },

    /// Cura status conditions
    CureStatus,

    /// Restaura HP (porcentaje del HP máximo)
    RestoreHP { percent: f32 },

    /// Causa recoil damage (porcentaje del HP máximo)
    RecoilDamage { percent: f32 },

    /// Bloquea movimientos de status
    BlockStatusMoves,
}

/// Aplica el efecto de un item al Pokémon
pub fn apply_item_effect(
    item_id: &str,
    pokemon: &mut PokemonInstance,
    move_id: Option<&str>,
    damage_dealt: Option<u16>,
) -> ItemEffectResult {
    let mut result = ItemEffectResult::default();

    match item_id {
        // Choice Band: +50% Attack en movimientos físicos
        "choice-band" => {
            if let Some(mv) = move_id {
                result.damage_multiplier = 1.5;
                result.move_locked = Some(mv.to_string());
                result.logs.push(format!(
                    "{}'s Choice Band boosted its attack!",
                    pokemon.species.display_name
                ));
            }
        }

        // Choice Specs: +50% Sp. Attack en movimientos especiales
        "choice-specs" => {
            if let Some(mv) = move_id {
                result.damage_multiplier = 1.5;
                result.move_locked = Some(mv.to_string());
                result.logs.push(format!(
                    "{}'s Choice Specs boosted its special attack!",
                    pokemon.species.display_name
                ));
            }
        }

        // Choice Scarf: +50% Speed (aplicado como stat boost permanente mientras esté equipado)
        "choice-scarf" => {
            if let Some(mv) = move_id {
                result.move_locked = Some(mv.to_string());
                // El boost de speed se maneja en el cálculo de velocidad directamente
                result.logs.push(format!(
                    "{}'s Choice Scarf boosted its speed!",
                    pokemon.species.display_name
                ));
            }
        }

        // Life Orb: +30% damage, -10% HP de recoil
        "life-orb" => {
            if damage_dealt.is_some() {
                result.damage_multiplier = 1.3;
                let max_hp = pokemon.base_computed_stats.hp;
                result.recoil_damage = max_hp / 10;

                // Aplicar recoil inmediatamente
                pokemon.current_hp = pokemon.current_hp.saturating_sub(result.recoil_damage);

                result.logs.push(format!(
                    "{} lost some HP due to Life Orb!",
                    pokemon.species.display_name
                ));
            }
        }

        // Assault Vest: +50% Sp. Defense (manejado en cálculo de daño)
        // El bloqueo de status moves se maneja en el validation system
        "assault-vest" => {
            // Este efecto es pasivo, no genera resultado aquí
        }

        // Sitrus Berry: Cura 25% HP
        "sitrus-berry" => {
            let max_hp = pokemon.base_computed_stats.hp;
            let heal_amount = max_hp / 4;
            let old_hp = pokemon.current_hp;
            pokemon.current_hp = (pokemon.current_hp + heal_amount).min(max_hp);
            result.healed_hp = pokemon.current_hp - old_hp;
            result.consumed = true;

            result.logs.push(format!(
                "{} restored HP using its Sitrus Berry!",
                pokemon.species.display_name
            ));

            // Consumir el item
            pokemon.held_item = None;
        }

        // Lum Berry: Cura todos los status conditions
        "lum-berry" => {
            if pokemon.status_condition.is_some() {
                let status_name = match pokemon.status_condition.as_ref().unwrap() {
                    StatusCondition::Burn => "burn",
                    StatusCondition::Freeze => "freeze",
                    StatusCondition::Paralysis => "paralysis",
                    StatusCondition::Poison => "poison",
                    StatusCondition::BadPoison => "bad poison",
                    StatusCondition::Sleep { .. } => "sleep",
                };

                pokemon.status_condition = None;
                result.status_cured = true;
                result.consumed = true;

                result.logs.push(format!(
                    "{}'s Lum Berry cured its {}!",
                    pokemon.species.display_name,
                    status_name
                ));

                // Consumir el item
                pokemon.held_item = None;
            }
        }

        // Weakness Policy: +2 Attack y Sp. Attack cuando recibe golpe super efectivo
        "weakness-policy" => {
            result.stat_boosts.push(("attack".to_string(), 2));
            result.stat_boosts.push(("sp_attack".to_string(), 2));
            result.consumed = true;

            result.logs.push(format!(
                "{}'s Weakness Policy sharply raised its offenses!",
                pokemon.species.display_name
            ));

            // Consumir el item
            pokemon.held_item = None;
        }

        _ => {
            // Item no reconocido o no implementado
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{PokemonSpecies, Stats, StatStages};

    fn create_test_pokemon(item: Option<String>, hp: u16) -> PokemonInstance {
        PokemonInstance {
            id: "test-1".to_string(),
            species: PokemonSpecies {
                id: "pikachu".to_string(),
                display_name: "Pikachu".to_string(),
                types: vec!["Electric".to_string()],
                base_stats: Default::default(),
            },
            level: 50,
            current_hp: hp,
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
    fn test_choice_band_effect() {
        let mut pokemon = create_test_pokemon(Some("choice-band".to_string()), 200);
        let result = apply_item_effect("choice-band", &mut pokemon, Some("tackle"), None);

        assert_eq!(result.damage_multiplier, 1.5);
        assert_eq!(result.move_locked, Some("tackle".to_string()));
        assert!(!result.consumed);
    }

    #[test]
    fn test_life_orb_effect() {
        let mut pokemon = create_test_pokemon(Some("life-orb".to_string()), 200);
        let result = apply_item_effect("life-orb", &mut pokemon, Some("tackle"), Some(50));

        assert_eq!(result.damage_multiplier, 1.3);
        assert_eq!(result.recoil_damage, 20); // 10% of 200
        assert_eq!(pokemon.current_hp, 180); // 200 - 20
        assert!(!result.consumed);
    }

    #[test]
    fn test_sitrus_berry_effect() {
        let mut pokemon = create_test_pokemon(Some("sitrus-berry".to_string()), 90);
        let result = apply_item_effect("sitrus-berry", &mut pokemon, None, None);

        assert_eq!(result.healed_hp, 50); // 25% of 200
        assert_eq!(pokemon.current_hp, 140); // 90 + 50
        assert!(result.consumed);
        assert!(pokemon.held_item.is_none()); // Berry consumed
    }

    #[test]
    fn test_lum_berry_effect() {
        let mut pokemon = create_test_pokemon(Some("lum-berry".to_string()), 200);
        pokemon.status_condition = Some(StatusCondition::Burn);

        let result = apply_item_effect("lum-berry", &mut pokemon, None, None);

        assert!(result.status_cured);
        assert!(result.consumed);
        assert!(pokemon.status_condition.is_none());
        assert!(pokemon.held_item.is_none()); // Berry consumed
    }

    #[test]
    fn test_weakness_policy_effect() {
        let mut pokemon = create_test_pokemon(Some("weakness-policy".to_string()), 200);
        let result = apply_item_effect("weakness-policy", &mut pokemon, None, None);

        assert_eq!(result.stat_boosts.len(), 2);
        assert!(result.stat_boosts.contains(&("attack".to_string(), 2)));
        assert!(result.stat_boosts.contains(&("sp_attack".to_string(), 2)));
        assert!(result.consumed);
        assert!(pokemon.held_item.is_none()); // Policy consumed
    }
}
