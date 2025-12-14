//! Helpers para crear objetos mock para testing
//! 
//! Este módulo proporciona funciones utilitarias para crear instancias
//! de Pokémon, movimientos y contextos de batalla sin depender de la
//! base de datos real, facilitando la escritura de tests unitarios.

use rand::rngs::StdRng;
use crate::models::{
    MoveData, MoveMeta, MoveStatChange, PokemonInstance, PokemonSpecies, PokemonType,
    RandomizedProfile, StatAdjustments, StatModifiers, StatMultipliers, StatStages, Stats, VolatileStatus,
};
use crate::battle::context::BattleContext;
use crate::models::{WeatherState, TerrainState};

/// Crea un Pokémon mock para testing
/// 
/// # Arguments
/// 
/// * `species` - ID de la especie (ej: "pikachu")
/// * `types` - Vector de tipos (ej: vec!["Electric"])
/// * `hp` - HP máximo del Pokémon
/// * `speed` - Velocidad del Pokémon
/// 
/// # Returns
/// 
/// Una instancia de `PokemonInstance` con stats base configurados manualmente,
/// battle_stages y volatile_status inicializados en valores por defecto.
pub fn create_mock_pokemon(
    species: &str,
    types: Vec<&str>,
    hp: u16,
    speed: u16,
) -> PokemonInstance {
    // Parsear tipos
    let primary_type = parse_type(types.get(0).copied().unwrap_or("Normal"));
    let secondary_type = types.get(1).map(|t| parse_type(t));

    // Crear stats base (usando valores razonables para un Pokémon de nivel 50)
    // Los stats se calculan automáticamente, pero necesitamos valores base
    let base_stats = Stats {
        hp: hp.saturating_sub(110), // Ajustar para que el HP calculado sea aproximadamente hp
        attack: 50,
        defense: 50,
        special_attack: 50,
        special_defense: 50,
        speed: speed.saturating_sub(5), // Ajustar para que la velocidad calculada sea aproximadamente speed
    };

    // Crear la especie mock
    let species_data = PokemonSpecies {
        species_id: species.to_string(),
        display_name: species.to_string(),
        generation: 1,
        primary_type,
        secondary_type,
        base_stats,
        move_pool: Vec::new(),
        possible_abilities: vec!["none".to_string()],
        is_starter_candidate: false,
        evolutions: Vec::new(),
    };

    // Calcular stats finales (nivel 50, IVs 31, EVs 0)
    let ivs = Stats {
        hp: 31,
        attack: 31,
        defense: 31,
        special_attack: 31,
        special_defense: 31,
        speed: 31,
    };
    let evs = Stats::zero();
    let computed_stats = crate::factory::compute_stats(&species_data.base_stats, &ivs, &evs, 50);

    // Crear el perfil randomizado
    let randomized_profile = RandomizedProfile {
        rolled_primary_type: primary_type,
        rolled_secondary_type: secondary_type,
        rolled_ability_id: "none".to_string(),
        stat_modifiers: StatModifiers {
            additive: StatAdjustments::zero(),
            multipliers: StatMultipliers::identity(),
        },
        learned_moves: Vec::new(),
        moves: Vec::new(),
    };

    // Crear la instancia
    let mut instance = PokemonInstance {
        id: format!("test-{}", species),
        species: species_data,
        level: 50,
        current_hp: computed_stats.hp,
        status_condition: None,
        ability: "none".to_string(),
        held_item: None,
        battle_stages: Some(StatStages::new()),
        volatile_status: Some(VolatileStatus::new()),
        individual_values: ivs,
        effort_values: evs,
        base_computed_stats: computed_stats,
        randomized_profile,
    };

    // Ajustar HP y velocidad si es necesario
    instance.current_hp = hp.min(instance.base_computed_stats.hp);
    // La velocidad ya está ajustada en base_stats

    instance
}

/// Crea un movimiento dummy para testing
/// 
/// # Arguments
/// 
/// * `id` - ID del movimiento (ej: "tackle")
/// * `power` - Poder del movimiento (None para movimientos de estado)
/// * `category` - Categoría: "physical", "special", o "status"
/// * `type_name` - Tipo del movimiento (ej: "Normal", "Fire")
/// 
/// # Returns
/// 
/// Una instancia de `MoveData` con valores por defecto razonables.
/// Se puede configurar `priority`, `target`, y `meta` usando el builder pattern.
pub fn create_dummy_move(
    id: &str,
    power: Option<u16>,
    category: &str,
    type_name: &str,
) -> MoveData {
    MoveData {
        id: id.to_string(),
        name: id.to_string(),
        r#type: type_name.to_string(),
        power,
        accuracy: Some(100),
        priority: 0,
        pp: 20,
        damage_class: category.to_string(),
        meta: MoveMeta::default(),
        stat_changes: Vec::new(),
        target: "selected-pokemon".to_string(),
    }
}

/// Builder para crear movimientos con configuración avanzada
pub struct MoveBuilder {
    move_data: MoveData,
}

impl MoveBuilder {
    /// Crea un nuevo builder con valores base
    pub fn new(id: &str, power: Option<u16>, category: &str, type_name: &str) -> Self {
        Self {
            move_data: create_dummy_move(id, power, category, type_name),
        }
    }

    /// Configura la prioridad del movimiento
    pub fn with_priority(mut self, priority: i8) -> Self {
        self.move_data.priority = priority;
        self
    }

    /// Configura el objetivo del movimiento
    pub fn with_target(mut self, target: &str) -> Self {
        self.move_data.target = target.to_string();
        self
    }

    /// Configura la precisión del movimiento
    pub fn with_accuracy(mut self, accuracy: u8) -> Self {
        self.move_data.accuracy = Some(accuracy);
        self
    }

    /// Configura el PP del movimiento
    pub fn with_pp(mut self, pp: u8) -> Self {
        self.move_data.pp = pp;
        self
    }

    /// Configura el ailment del movimiento
    pub fn with_ailment(mut self, ailment: &str, chance: u8) -> Self {
        self.move_data.meta.ailment = ailment.to_string();
        self.move_data.meta.ailment_chance = chance;
        self
    }

    /// Configura cambios de stats del movimiento
    pub fn with_stat_changes(mut self, stat_changes: Vec<MoveStatChange>) -> Self {
        self.move_data.stat_changes = stat_changes;
        self
    }

    /// Configura el crit_rate del movimiento
    pub fn with_crit_rate(mut self, crit_rate: u8) -> Self {
        self.move_data.meta.crit_rate = crit_rate;
        self
    }

    /// Configura el drain del movimiento (porcentaje de daño absorbido)
    pub fn with_drain(mut self, drain: i8) -> Self {
        self.move_data.meta.drain = drain;
        self
    }

    /// Configura la probabilidad de flinch
    pub fn with_flinch_chance(mut self, chance: u8) -> Self {
        self.move_data.meta.flinch_chance = chance;
        self
    }

    /// Configura movimientos multi-hit
    pub fn with_hit_range(mut self, min_hits: u8, max_hits: u8) -> Self {
        self.move_data.meta.min_hits = Some(min_hits);
        self.move_data.meta.max_hits = Some(max_hits);
        self
    }

    /// Configura movimientos con carga (ej: Solar Beam)
    pub fn with_charge_turns(mut self, min_turns: u8, max_turns: u8) -> Self {
        self.move_data.meta.min_turns = Some(min_turns);
        self.move_data.meta.max_turns = Some(max_turns);
        self
    }

    /// Construye el MoveData final
    pub fn build(self) -> MoveData {
        self.move_data
    }
}

/// Crea un contexto de batalla para testing
/// 
/// # Arguments
/// 
/// * `attacker` - Pokémon atacante (mutable)
/// * `defender` - Pokémon defensor (mutable)
/// * `move_data` - Datos del movimiento a ejecutar
/// * `rng` - RNG mutable para usar en el contexto
/// 
/// # Returns
/// 
/// Un `BattleContext` listo para probar funciones aisladas.
/// 
/// # Example
/// 
/// ```rust,no_run
/// use rand::rngs::StdRng;
/// use rand::SeedableRng;
/// let mut rng = StdRng::seed_from_u64(42);
/// let mut attacker = create_mock_pokemon("pikachu", vec!["Electric"], 100, 90);
/// let mut defender = create_mock_pokemon("charmander", vec!["Fire"], 80, 70);
/// let move_data = create_dummy_move("thunderbolt", Some(90), "special", "Electric");
/// let context = create_battle_context(&mut attacker, &mut defender, &move_data, &mut rng);
/// ```
pub fn create_battle_context<'a>(
    attacker: &'a mut PokemonInstance,
    defender: &'a mut PokemonInstance,
    move_data: &'a MoveData,
    rng: &'a mut StdRng,
) -> BattleContext<'a> {
    let attacker_name = attacker.species.display_name.clone();
    let defender_name = defender.species.display_name.clone();

    BattleContext::new(
        attacker,
        defender,
        move_data,
        attacker_name,
        defender_name,
        rng,
        None, // weather
        None, // terrain
    )
}

/// Crea un contexto de batalla con clima y terreno
/// 
/// Similar a `create_battle_context`, pero permite especificar clima y terreno.
pub fn create_battle_context_with_weather_terrain<'a>(
    attacker: &'a mut PokemonInstance,
    defender: &'a mut PokemonInstance,
    move_data: &'a MoveData,
    rng: &'a mut StdRng,
    weather: Option<&'a WeatherState>,
    terrain: Option<&'a TerrainState>,
) -> BattleContext<'a> {
    let attacker_name = attacker.species.display_name.clone();
    let defender_name = defender.species.display_name.clone();

    BattleContext::new(
        attacker,
        defender,
        move_data,
        attacker_name,
        defender_name,
        rng,
        weather,
        terrain,
    )
}

/// Helper para parsear un string de tipo a PokemonType
fn parse_type(type_str: &str) -> PokemonType {
    match type_str.to_lowercase().as_str() {
        "normal" => PokemonType::Normal,
        "fire" => PokemonType::Fire,
        "water" => PokemonType::Water,
        "grass" => PokemonType::Grass,
        "electric" => PokemonType::Electric,
        "ice" => PokemonType::Ice,
        "fighting" => PokemonType::Fighting,
        "poison" => PokemonType::Poison,
        "ground" => PokemonType::Ground,
        "flying" => PokemonType::Flying,
        "psychic" => PokemonType::Psychic,
        "bug" => PokemonType::Bug,
        "rock" => PokemonType::Rock,
        "ghost" => PokemonType::Ghost,
        "dragon" => PokemonType::Dragon,
        "dark" => PokemonType::Dark,
        "steel" => PokemonType::Steel,
        "fairy" => PokemonType::Fairy,
        _ => PokemonType::Unknown,
    }
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_create_mock_pokemon() {
        let pikachu = create_mock_pokemon("pikachu", vec!["Electric"], 100, 90);
        
        assert_eq!(pikachu.species.species_id, "pikachu");
        assert_eq!(pikachu.species.primary_type, PokemonType::Electric);
        assert!(pikachu.battle_stages.is_some());
        assert!(pikachu.volatile_status.is_some());
        assert!(pikachu.current_hp > 0);
    }

    #[test]
    fn test_create_dummy_move() {
        let tackle = create_dummy_move("tackle", Some(40), "physical", "Normal");
        
        assert_eq!(tackle.id, "tackle");
        assert_eq!(tackle.power, Some(40));
        assert_eq!(tackle.damage_class, "physical");
        assert_eq!(tackle.r#type, "Normal");
    }

    #[test]
    fn test_move_builder() {
        let fire_blast = MoveBuilder::new("fire-blast", Some(110), "special", "Fire")
            .with_accuracy(85)
            .with_pp(5)
            .with_ailment("burn", 10)
            .with_crit_rate(1)
            .build();
        
        assert_eq!(fire_blast.accuracy, Some(85));
        assert_eq!(fire_blast.pp, 5);
        assert_eq!(fire_blast.meta.ailment, "burn");
        assert_eq!(fire_blast.meta.ailment_chance, 10);
        assert_eq!(fire_blast.meta.crit_rate, 1);
    }

    #[test]
    fn test_create_battle_context() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        
        let mut attacker = create_mock_pokemon("pikachu", vec!["Electric"], 100, 90);
        let mut defender = create_mock_pokemon("charmander", vec!["Fire"], 80, 70);
        let move_data = create_dummy_move("thunderbolt", Some(90), "special", "Electric");
        let mut rng = StdRng::seed_from_u64(123);
        
        let context = create_battle_context(
            &mut attacker,
            &mut defender,
            &move_data,
            &mut rng,
        );
        
        assert_eq!(context.attacker.species.species_id, "pikachu");
        assert_eq!(context.defender.species.species_id, "charmander");
        assert_eq!(context.move_data.id, "thunderbolt");
    }
}

