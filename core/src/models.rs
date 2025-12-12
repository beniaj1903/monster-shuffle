use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PokemonType {
    Normal,
    Fire,
    Water,
    Grass,
    Electric,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
    Unknown,
}

impl Default for PokemonType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stat {
    Hp,
    Attack,
    Defense,
    SpecialAttack,
    SpecialDefense,
    Speed,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Stats {
    pub hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub special_attack: u16,
    pub special_defense: u16,
    pub speed: u16,
}

impl Stats {
    pub const fn zero() -> Self {
        Self {
            hp: 0,
            attack: 0,
            defense: 0,
            special_attack: 0,
            special_defense: 0,
            speed: 0,
        }
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::zero()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StatAdjustments {
    pub hp: i16,
    pub attack: i16,
    pub defense: i16,
    pub special_attack: i16,
    pub special_defense: i16,
    pub speed: i16,
}

impl StatAdjustments {
    pub const fn zero() -> Self {
        Self {
            hp: 0,
            attack: 0,
            defense: 0,
            special_attack: 0,
            special_defense: 0,
            speed: 0,
        }
    }
}

impl Default for StatAdjustments {
    fn default() -> Self {
        Self::zero()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StatMultipliers {
    pub hp: f32,
    pub attack: f32,
    pub defense: f32,
    pub special_attack: f32,
    pub special_defense: f32,
    pub speed: f32,
}

impl StatMultipliers {
    pub const fn identity() -> Self {
        Self {
            hp: 1.0,
            attack: 1.0,
            defense: 1.0,
            special_attack: 1.0,
            special_defense: 1.0,
            speed: 1.0,
        }
    }
}

impl Default for StatMultipliers {
    fn default() -> Self {
        Self::identity()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct StatModifiers {
    pub additive: StatAdjustments,
    pub multipliers: StatMultipliers,
}

/// Representa los cambios temporales de stats en batalla (stages)
/// Los valores van de -6 a +6, donde 0 es el estado base
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct StatStages {
    pub attack: i8,          // -6 a +6
    pub defense: i8,
    pub special_attack: i8,
    pub special_defense: i8,
    pub speed: i8,
    pub accuracy: i8,
    pub evasion: i8,
}

impl StatStages {
    /// Crea un StatStages con todos los valores en 0 (estado base)
    pub const fn new() -> Self {
        Self {
            attack: 0,
            defense: 0,
            special_attack: 0,
            special_defense: 0,
            speed: 0,
            accuracy: 0,
            evasion: 0,
        }
    }

    /// Aplica un cambio de stat, limitando el rango a -6 a +6
    pub fn apply_change(&mut self, stat: &str, change: i8) {
        let clamped_change = change.clamp(-6, 6);
        match stat {
            "attack" => self.attack = (self.attack + clamped_change).clamp(-6, 6),
            "defense" => self.defense = (self.defense + clamped_change).clamp(-6, 6),
            "special_attack" => self.special_attack = (self.special_attack + clamped_change).clamp(-6, 6),
            "special_defense" => self.special_defense = (self.special_defense + clamped_change).clamp(-6, 6),
            "speed" => self.speed = (self.speed + clamped_change).clamp(-6, 6),
            "accuracy" => self.accuracy = (self.accuracy + clamped_change).clamp(-6, 6),
            "evasion" => self.evasion = (self.evasion + clamped_change).clamp(-6, 6),
            _ => {} // Ignorar stats desconocidos
        }
    }

    /// Calcula el multiplicador de stat basado en el stage
    /// Fórmula de Pokémon: (2 + stage) / 2 si stage >= 0, o 2 / (2 - stage) si stage < 0
    pub fn get_stat_multiplier(&self, stat: &str) -> f32 {
        let stage = match stat {
            "attack" => self.attack,
            "defense" => self.defense,
            "special_attack" => self.special_attack,
            "special_defense" => self.special_defense,
            "speed" => self.speed,
            "accuracy" => self.accuracy,
            "evasion" => self.evasion,
            _ => return 1.0,
        };

        if stage >= 0 {
            (2.0 + stage as f32) / 2.0
        } else {
            2.0 / (2.0 - stage as f32)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MoveTemplate {
    pub id: String,
    pub display_name: String,
    pub move_type: PokemonType,
    pub category: MoveCategory,
    pub base_power: Option<u16>,
    pub accuracy: Option<u8>,
    pub max_pp: u8,
}

/// Metadatos de efectos secundarios de un movimiento
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct MoveMeta {
    /// Estado alterado que puede aplicar: "burn", "paralysis", "none", etc.
    pub ailment: String,
    /// Probabilidad de aplicar el estado alterado (0-100)
    pub ailment_chance: u8,
    /// Tasa de crítico (0-4)
    pub crit_rate: u8,
    /// Porcentaje de daño que cura al usuario (drenaje)
    pub drain: i8,
    /// Probabilidad de causar retroceso (0-100)
    pub flinch_chance: u8,
    /// Probabilidad de que ocurran los cambios de stats (0-100)
    pub stat_chance: u8,
}

/// Cambio de stat que aplica un movimiento
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MoveStatChange {
    /// Nombre del stat afectado: "attack", "special_attack", "speed", etc.
    pub stat: String,
    /// Cambio en el stage (-6 a +6)
    pub change: i8,
}

/// Datos de un movimiento cargado desde moves.json
/// Estructura completa para el sistema de batalla avanzado (VGC)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MoveData {
    /// ID del movimiento (ej: "scratch", "tackle")
    pub id: String,
    /// Nombre para mostrar (ej: "Scratch", "Tackle")
    pub name: String,
    /// Tipo del movimiento (ej: "Normal", "Fire")
    pub r#type: String,
    /// Poder base del movimiento (null para movimientos de estado)
    pub power: Option<u16>,
    /// Precisión del movimiento (null para movimientos que siempre aciertan)
    pub accuracy: Option<u8>,
    /// Prioridad del movimiento (puede ser negativo para movimientos lentos)
    #[serde(default)]
    pub priority: i8,
    /// Puntos de poder (PP) del movimiento
    pub pp: u8,
    /// Categoría de daño: "physical", "special", "status"
    pub damage_class: String,
    /// Metadatos de efectos secundarios
    #[serde(default)]
    pub meta: MoveMeta,
    /// Cambios de stats que aplica el movimiento
    #[serde(default)]
    pub stat_changes: Vec<MoveStatChange>,
    /// Objetivo del movimiento: "selected-pokemon", "users-field", "user", etc.
    #[serde(default)]
    pub target: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct MoveInstance {
    /// Identifier that maps back to a template for lookups.
    pub template_id: String,
    /// Type changes can happen under randomlocke rules.
    pub randomized_type: Option<PokemonType>,
    /// Allows small positive/negative shifts applied to the base power.
    pub power_variation: Option<i16>,
    /// Percentage multiplier (e.g. 0.9 for -10%).
    pub accuracy_multiplier: Option<f32>,
}

/// Información sobre una evolución posible de un Pokémon
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EvolutionData {
    /// El ID de la especie objetivo (siguiente fase evolutiva)
    pub target_species_id: String,
    /// Nivel mínimo requerido para evolucionar (si aplica)
    pub min_level: Option<u8>,
    /// Trigger de la evolución: "level-up", "item", "trade", etc.
    pub trigger: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PokemonSpecies {
    pub species_id: String,
    pub display_name: String,
    pub generation: u8,
    pub primary_type: PokemonType,
    pub secondary_type: Option<PokemonType>,
    pub base_stats: Stats,
    /// List of move template identifiers available to the species.
    pub move_pool: Vec<String>,
    /// Indicates if this species is a candidate for starter selection (base of 3+ stage evolution chain)
    #[serde(default)]
    pub is_starter_candidate: bool,
    /// Lista de posibles evoluciones de esta especie
    #[serde(default)]
    pub evolutions: Vec<EvolutionData>,
}

impl PokemonSpecies {
    /// Calcula el Base Stat Total (BST) de la especie
    /// Retorna la suma de todos los stats base
    pub fn bst(&self) -> u32 {
        self.base_stats.hp as u32
            + self.base_stats.attack as u32
            + self.base_stats.defense as u32
            + self.base_stats.special_attack as u32
            + self.base_stats.special_defense as u32
            + self.base_stats.speed as u32
    }
}

// Agrega este Enum para condiciones de estado (Quemado, Congelado, etc.)
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCondition {
    Burn,
    Freeze,
    Paralysis,
    Poison,
    BadPoison, // Toxic
    Sleep,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct RandomizedProfile {
    pub rolled_primary_type: PokemonType,
    pub rolled_secondary_type: Option<PokemonType>,
    
    // --- NUEVO: Habilidad Aleatoria ---
    // Usamos String por ahora para simplificar, idealmente sería un Enum o ID
    pub rolled_ability_id: String, 
    
    /// Modificadores permanentes de esta "Run" (ej. buff global al tipo fuego)
    pub stat_modifiers: StatModifiers,
    pub moves: Vec<MoveInstance>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PokemonInstance {
    // Identificador único de esta instancia (UUID) para base de datos
    pub id: String, 
    pub species: PokemonSpecies,
    pub level: u8,
    
    // --- Estado actual ---
    pub current_hp: u16,
    pub status_condition: Option<StatusCondition>,
    
    /// Stages de stats en batalla (None fuera de batalla, Some con valores base al entrar)
    /// Representa cambios temporales de stats durante la batalla (-6 a +6)
    #[serde(default)]
    pub battle_stages: Option<StatStages>,
    
    pub individual_values: Stats,
    pub effort_values: Stats,
    
    /// Stats calculados (Base + IV + EV + Nivel) = VIDA MÁXIMA
    pub base_computed_stats: Stats,
    
    pub randomized_profile: RandomizedProfile,
}

impl PokemonInstance {
    /// Restaura completamente el Pokémon a su estado óptimo
    /// Restaura toda la vida y elimina cualquier condición de estado
    /// También resetea los battle_stages si están presentes
    pub fn full_restore(&mut self) {
        self.current_hp = self.base_computed_stats.hp;
        self.status_condition = None;
        if let Some(ref mut stages) = self.battle_stages {
            *stages = StatStages::new();
        }
    }

    /// Inicializa los battle_stages al entrar en batalla
    /// Debe llamarse cuando el Pokémon entra en combate
    pub fn init_battle_stages(&mut self) {
        self.battle_stages = Some(StatStages::new());
    }

    /// Resetea los battle_stages al salir de batalla
    /// Debe llamarse cuando el Pokémon sale de combate
    pub fn reset_battle_stages(&mut self) {
        self.battle_stages = None;
    }

    /// Ajusta el nivel del Pokémon y recalcula sus stats
    /// Mantiene los IVs, EVs y movimientos existentes
    pub fn set_level(&mut self, new_level: u8) {
        use crate::factory::compute_stats;
        
        self.level = new_level;
        // Recalcular stats con el nuevo nivel, manteniendo IVs y EVs
        self.base_computed_stats = compute_stats(
            &self.species.base_stats,
            &self.individual_values,
            &self.effort_values,
            new_level,
        );
        // Ajustar HP actual proporcionalmente si es necesario
        // Por ahora, simplemente lo ponemos al máximo
        self.current_hp = self.base_computed_stats.hp;
    }

    /// Aprende un nuevo movimiento y lo añade a la bolsa de movimientos
    /// La bolsa puede tener cualquier tamaño, pero solo los primeros 4 están activos en batalla
    pub fn learn_new_move(&mut self, new_move_id: String) {
        self.randomized_profile.moves.push(MoveInstance {
            template_id: new_move_id,
            randomized_type: None,
            power_variation: None,
            accuracy_multiplier: None,
        });
    }

    /// Obtiene los movimientos activos (primeros 4) para usar en batalla
    /// Retorna una referencia a los primeros 4 movimientos (o menos si hay menos de 4)
    pub fn get_active_moves(&self) -> &[MoveInstance] {
        let moves = &self.randomized_profile.moves;
        let active_count = moves.len().min(4);
        &moves[0..active_count]
    }
}

