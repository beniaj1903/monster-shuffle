use rand::rngs::StdRng;
use rand::{seq::SliceRandom, Rng, SeedableRng};

use crate::models::{
    LearnedMove, MoveInstance, PokemonInstance, PokemonSpecies, RandomizedProfile, StatModifiers, Stats,
};

// NOTA: Se eliminaron las constantes y funciones de randomización de tipos (Vanilla behavior)

fn roll_ivs(rng: &mut StdRng) -> Stats {
    Stats {
        hp: rng.gen_range(0u16..=31),
        attack: rng.gen_range(0u16..=31),
        defense: rng.gen_range(0u16..=31),
        special_attack: rng.gen_range(0u16..=31),
        special_defense: rng.gen_range(0u16..=31),
        speed: rng.gen_range(0u16..=31),
    }
}

fn compute_stat(base: u16, iv: u16, ev: u16, level: u8, is_hp: bool) -> u16 {
    // Casteamos a u32 para evitar overflow durante la multiplicación
    let base_term = ((base as u32 * 2) + iv as u32 + (ev as u32 / 4)) * level as u32 / 100;

    let result = if is_hp {
        base_term + level as u32 + 10
    } else {
        base_term + 5
    };

    result.min(u16::MAX as u32) as u16
}

/// Calcula los stats finales basándose en stats base, IVs, EVs y nivel
/// Esta función es pública para que pueda ser usada por otros módulos
pub fn compute_stats(base: &Stats, ivs: &Stats, evs: &Stats, level: u8) -> Stats {
    Stats {
        hp: compute_stat(base.hp, ivs.hp, evs.hp, level, true),
        attack: compute_stat(base.attack, ivs.attack, evs.attack, level, false),
        defense: compute_stat(base.defense, ivs.defense, evs.defense, level, false),
        special_attack: compute_stat(
            base.special_attack,
            ivs.special_attack,
            evs.special_attack,
            level,
            false,
        ),
        special_defense: compute_stat(
            base.special_defense,
            ivs.special_defense,
            evs.special_defense,
            level,
            false,
        ),
        speed: compute_stat(base.speed, ivs.speed, evs.speed, level, false),
    }
}

fn roll_moves(rng: &mut StdRng, move_pool: &[String]) -> Vec<MoveInstance> {
    let mut pool = move_pool.to_vec();
    pool.shuffle(rng);
    pool.into_iter()
        .take(4)
        .map(|template_id| MoveInstance {
            template_id,
            randomized_type: None,      // Vanilla: Sin cambio de tipo
            power_variation: None,      // Vanilla: Sin cambio de poder
            accuracy_multiplier: None,  // Vanilla: Sin cambio de precisión
        })
        .collect()
}

/// Genera movimientos aleatorios del pool global (modo Chaos)
/// Selecciona 6 movimientos únicos aleatorios del pool global
fn roll_chaos_moves(rng: &mut StdRng, global_move_pool: &[String]) -> Vec<MoveInstance> {
    if global_move_pool.is_empty() {
        return Vec::new();
    }
    
    let mut pool = global_move_pool.to_vec();
    pool.shuffle(rng);
    
    // Seleccionar 6 movimientos únicos
    let mut selected = Vec::new();
    let mut used = std::collections::HashSet::new();
    
    for move_id in pool {
        if !used.contains(&move_id) {
            selected.push(MoveInstance {
                template_id: move_id.clone(),
                randomized_type: None,
                power_variation: None,
                accuracy_multiplier: None,
            });
            used.insert(move_id);
            if selected.len() >= 6 {
                break;
            }
        }
    }
    
    selected
}

/// Selecciona una habilidad aleatoria de las posibles habilidades de la especie
/// Prioriza habilidades no-hidden (slot 1 o 2), pero puede incluir hidden si no hay otras
fn roll_ability_id(species: &PokemonSpecies, rng: &mut StdRng) -> String {
    if species.possible_abilities.is_empty() {
        // Fallback si no hay habilidades definidas
        return "none".to_string();
    }
    
    // Seleccionar una habilidad aleatoria de las disponibles
    use rand::seq::SliceRandom;
    species.possible_abilities
        .choose(rng)
        .cloned()
        .unwrap_or_else(|| "none".to_string())
}

fn roll_instance_id(rng: &mut StdRng) -> String {
    let high: u64 = rng.gen();
    let low: u64 = rng.gen();
    format!("{:016x}{:016x}", high, low)
}

/// Crea una instancia de Pokémon determinista a partir de una semilla.
/// Comportamiento Vanilla: Tipos y Stats base originales.
/// 
/// Si `chaos_mode` es true, los movimientos se seleccionan aleatoriamente del `global_move_pool`.
/// Si es false, se usan los movimientos del `species.move_pool` (comportamiento vanilla).
/// 
/// `moves_data` es un HashMap opcional con los datos de los movimientos para inicializar el PP.
/// Si es None, los movimientos se crearán con PP 0 y se inicializarán cuando se usen.
pub fn create_pokemon_instance(
    species: &PokemonSpecies,
    level: u8,
    seed: u64,
    chaos_mode: bool,
    global_move_pool: &[String],
    moves_data: Option<&std::collections::HashMap<String, crate::models::MoveData>>,
) -> PokemonInstance {
    let mut rng = StdRng::seed_from_u64(seed);

    let ivs = roll_ivs(&mut rng);
    let evs = Stats::zero();

    // Stats calculados sobre la base original
    let base_computed_stats = compute_stats(&species.base_stats, &ivs, &evs, level);

    // Seleccionar movimientos según el modo
    let moves = if chaos_mode {
        roll_chaos_moves(&mut rng, global_move_pool)
    } else {
        roll_moves(&mut rng, &species.move_pool)
    };
    
    // Crear learned_moves con PP inicializado si tenemos los datos
    let learned_moves: Vec<LearnedMove> = moves.iter()
        .map(|move_instance| {
            let max_pp = moves_data
                .and_then(|data| data.get(&move_instance.template_id))
                .map(|move_data| move_data.pp)
                .unwrap_or(0); // Si no tenemos los datos, inicializar con 0 (se inicializará después)
            
            LearnedMove {
                move_id: move_instance.template_id.clone(),
                current_pp: max_pp,
                max_pp,
            }
        })
        .collect();
    
    let rolled_ability_id = roll_ability_id(species, &mut rng);
    let ability = rolled_ability_id.clone(); // La habilidad activa es la misma que la rolled
    let id = roll_instance_id(&mut rng);

    // Perfil "Randomizado" coincide con el original
    let randomized_profile = RandomizedProfile {
        rolled_primary_type: species.primary_type,
        rolled_secondary_type: species.secondary_type,
        rolled_ability_id,
        stat_modifiers: StatModifiers::default(),
        learned_moves,
        moves,
    };

    PokemonInstance {
        id,
        species: species.clone(),
        level,
        current_hp: base_computed_stats.hp,
        status_condition: None,
        ability,
        held_item: None, // Por defecto sin objeto, se puede asignar después
        battle_stages: None,
        volatile_status: None, // Se inicializa cuando entra en batalla
        individual_values: ivs,
        effort_values: evs,
        base_computed_stats,
        randomized_profile,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PokemonType;

    fn sample_species() -> PokemonSpecies {
        PokemonSpecies {
            species_id: "001".into(),
            display_name: "Bulbasaur".into(),
            generation: 1,
            primary_type: PokemonType::Grass,
            secondary_type: Some(PokemonType::Poison),
            base_stats: Stats {
                hp: 45,
                attack: 49,
                defense: 49,
                special_attack: 65,
                special_defense: 65,
                speed: 45,
            },
            move_pool: vec![
                "tackle".into(),
                "growl".into(),
                "vine-whip".into(),
                "razor-leaf".into(),
                "sleep-powder".into(),
            ],
            possible_abilities: vec!["overgrow".into(), "chlorophyll".into()],
            is_starter_candidate: false,
            evolutions: Vec::new(),
        }
    }

    fn compute_expected_hp(base: u16, iv: u16, ev: u16, level: u8) -> u16 {
        let core = ((2 * base as u32 + iv as u32 + (ev as u32 / 4)) * level as u32) / 100;
        (core + level as u32 + 10) as u16
    }

    #[test]
    fn deterministic_with_same_seed() {
        let species = sample_species();
        let empty_pool: Vec<String> = Vec::new();
        let a = create_pokemon_instance(&species, 30, 12345, false, &empty_pool, None);
        let b = create_pokemon_instance(&species, 30, 12345, false, &empty_pool, None);
        assert_eq!(a, b, "Same seed should produce identical instance");
    }

    #[test]
    fn different_seed_changes_output() {
        let species = sample_species();
        let empty_pool: Vec<String> = Vec::new();
        let a = create_pokemon_instance(&species, 30, 12345, false, &empty_pool, None);
        let b = create_pokemon_instance(&species, 30, 54321, false, &empty_pool, None);
        assert_ne!(a, b, "Different seed should change the generated instance");
    }

    #[test]
    fn types_remain_vanilla() {
        let species = sample_species();
        let empty_pool: Vec<String> = Vec::new();
        let instance = create_pokemon_instance(&species, 30, 999, false, &empty_pool, None);
        assert_eq!(instance.randomized_profile.rolled_primary_type, species.primary_type);
        assert_eq!(instance.randomized_profile.rolled_secondary_type, species.secondary_type);
    }

    #[test]
    fn moves_come_from_pool_and_limit_four() {
        let species = sample_species();
        let empty_pool: Vec<String> = Vec::new();
        let instance = create_pokemon_instance(&species, 30, 42, false, &empty_pool, None);
        assert!(instance.randomized_profile.moves.len() <= 4);
        for mv in &instance.randomized_profile.moves {
            assert!(
                species.move_pool.contains(&mv.template_id),
                "Move {} must come from species pool",
                mv.template_id
            );
        }
    }

    #[test]
    fn ability_selected_from_species_pool() {
        let species = sample_species();
        let empty_pool: Vec<String> = Vec::new();
        let instance = create_pokemon_instance(&species, 30, 7, false, &empty_pool, None);
        // La habilidad debe venir de las posibles habilidades de la especie
        assert!(
            species.possible_abilities.contains(&instance.ability),
            "Ability '{}' should be one of the species' possible abilities: {:?}",
            instance.ability,
            species.possible_abilities
        );
        // También debe coincidir con rolled_ability_id
        assert_eq!(instance.ability, instance.randomized_profile.rolled_ability_id);
    }

    #[test]
    fn hp_calculated_with_standard_formula() {
        let species = sample_species();
        let level = 50;
        let empty_pool: Vec<String> = Vec::new();
        let instance = create_pokemon_instance(&species, level, 2024, false, &empty_pool, None);
        let expected_hp = compute_expected_hp(
            species.base_stats.hp,
            instance.individual_values.hp,
            instance.effort_values.hp,
            level,
        );
        assert_eq!(instance.base_computed_stats.hp, expected_hp);
        assert_eq!(instance.current_hp, expected_hp, "Current HP starts at max HP");
    }
}

