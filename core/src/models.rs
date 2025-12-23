use serde::{Deserialize, Serialize};

/// Formato de batalla (Single o Double)
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleFormat {
    /// Batalla 1v1 (formato estándar)
    Single,
    /// Batalla 2v2 (formato doble)
    Double,
}

impl Default for BattleFormat {
    fn default() -> Self {
        Self::Single
    }
}

/// Posición de un Pokémon en el campo de batalla
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldPosition {
    /// Posición izquierda del jugador (en combates dobles)
    PlayerLeft,
    /// Posición derecha del jugador (en combates dobles)
    PlayerRight,
    /// Posición izquierda del oponente (en combates dobles)
    OpponentLeft,
    /// Posición derecha del oponente (en combates dobles)
    OpponentRight,
}

/// Tipo de clima en batalla
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherType {
    /// Harsh Sunlight (Sol intenso)
    Sun,
    /// Rain (Lluvia)
    Rain,
    /// Sandstorm (Tormenta de arena)
    Sandstorm,
    /// Hail / Snow (Granizo / Nieve)
    Hail,
    /// Sin clima activo
    None,
}

impl Default for WeatherType {
    fn default() -> Self {
        Self::None
    }
}

/// Estado del clima en batalla
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WeatherState {
    /// Tipo de clima activo
    pub weather_type: WeatherType,
    /// Turnos restantes del clima (por defecto 5)
    pub turns_remaining: u8,
}

impl WeatherState {
    /// Crea un nuevo estado de clima con duración por defecto (5 turnos)
    pub fn new(weather_type: WeatherType) -> Self {
        Self {
            weather_type,
            turns_remaining: 5,
        }
    }

    /// Crea un nuevo estado de clima con duración personalizada
    pub fn with_duration(weather_type: WeatherType, turns_remaining: u8) -> Self {
        Self {
            weather_type,
            turns_remaining,
        }
    }
}

/// Tipo de terreno en batalla
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    /// Electric Terrain (Campo Eléctrico)
    Electric,
    /// Grassy Terrain (Campo de Hierba)
    Grassy,
    /// Misty Terrain (Campo de Niebla)
    Misty,
    /// Psychic Terrain (Campo Psíquico)
    Psychic,
}

/// Estado del terreno en batalla
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TerrainState {
    /// Tipo de terreno activo
    pub terrain_type: TerrainType,
    /// Turnos restantes del terreno (por defecto 5)
    pub turns_remaining: u8,
}

impl TerrainState {
    /// Crea un nuevo estado de terreno con duración por defecto (5 turnos)
    pub fn new(terrain_type: TerrainType) -> Self {
        Self {
            terrain_type,
            turns_remaining: 5,
        }
    }

    /// Crea un nuevo estado de terreno con duración personalizada
    pub fn with_duration(terrain_type: TerrainType, turns_remaining: u8) -> Self {
        Self {
            terrain_type,
            turns_remaining,
        }
    }
}

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

/// Estados volátiles que se resetean al final de cada turno o al salir de batalla
/// Estos estados son temporales y no persisten entre turnos
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct VolatileStatus {
    /// Si el Pokémon retrocedió este turno (flinch)
    pub flinched: bool,
    /// Si el Pokémon está confundido (para futura implementación)
    pub confused: bool,
    /// Stage de crítico (para Focus Energy, etc.)
    pub crit_stage: u8,
    /// Si el Pokémon está protegido este turno (Protect/Detect)
    pub protected: bool,
    /// Contador de protección para disminuir probabilidad de éxito consecutivo
    pub protect_counter: u8,
    /// Si el Pokémon debe recargar energía (Hyper Beam)
    pub must_recharge: bool,
    /// ID del movimiento que está cargando (Solar Beam, etc.)
    pub charging_move: Option<String>,
    /// Contador de turnos con BadPoison para daño escalante (Toxic)
    #[serde(default)]
    pub badly_poisoned_turns: u8,

    // --- Fase 1.3: Volatile Status Avanzado ---

    /// ID del Pokémon que causó infatuation (Attract)
    /// Si está presente, hay 50% chance de no poder atacar
    #[serde(default)]
    pub infatuated_by: Option<String>,

    /// Si el Pokémon tiene Leech Seed activo
    /// Pierde 1/8 HP al final del turno y cura al usuario
    #[serde(default)]
    pub leech_seeded: bool,

    /// ID del Pokémon que usó Leech Seed (para curación)
    #[serde(default)]
    pub leech_seed_source: Option<String>,

    /// HP del Substitute activo (0 = no hay substitute)
    /// Absorbe daño hasta que se rompa
    #[serde(default)]
    pub substitute_hp: u16,

    /// Contador de Perish Song (None = no activo, Some(n) = turnos restantes)
    /// Cuando llega a 0, el Pokémon es debilitado
    #[serde(default)]
    pub perish_count: Option<u8>,

    // --- Fase 2.1: Protecciones Avanzadas ---

    /// Si Wide Guard está activo este turno (protege del equipo de spread moves)
    #[serde(default)]
    pub wide_guard_active: bool,

    /// Si Quick Guard está activo este turno (protege del equipo de priority moves)
    #[serde(default)]
    pub quick_guard_active: bool,

    /// Si Mat Block está activo este turno (protege del equipo, solo funciona en turno 1)
    #[serde(default)]
    pub mat_block_active: bool,

    /// Si Crafty Shield está activo este turno (protege del equipo de status moves)
    #[serde(default)]
    pub crafty_shield_active: bool,

    // --- Fase 2.3: Movimientos de Switch Forzado ---

    /// Si este Pokémon fue marcado para switch forzado este turno (Dragon Tail, Roar, etc.)
    #[serde(default)]
    pub forced_switch: bool,
}

impl VolatileStatus {
    /// Crea un nuevo VolatileStatus con todos los valores en false/0
    pub const fn new() -> Self {
        Self {
            flinched: false,
            confused: false,
            crit_stage: 0,
            protected: false,
            protect_counter: 0,
            must_recharge: false,
            charging_move: None,
            badly_poisoned_turns: 0,
            infatuated_by: None,
            leech_seeded: false,
            leech_seed_source: None,
            substitute_hp: 0,
            perish_count: None,
            wide_guard_active: false,
            quick_guard_active: false,
            mat_block_active: false,
            crafty_shield_active: false,
            forced_switch: false,
        }
    }

    /// Resetea todos los estados volátiles al inicio de un nuevo turno
    /// protected siempre se resetea a false al inicio del turno
    pub fn reset_turn(&mut self) {
        self.flinched = false;
        self.protected = false;

        // Resetear protecciones avanzadas (Fase 2.1)
        self.wide_guard_active = false;
        self.quick_guard_active = false;
        self.mat_block_active = false;
        self.crafty_shield_active = false;

        // Resetear switch forzado (Fase 2.3)
        self.forced_switch = false;

        // confused, crit_stage, protect_counter, must_recharge y charging_move persisten entre turnos
    }

    /// Resetea completamente todos los estados volátiles
    pub fn reset_all(&mut self) {
        self.flinched = false;
        self.confused = false;
        self.crit_stage = 0;
        self.protected = false;
        self.protect_counter = 0;
        self.must_recharge = false;
        self.charging_move = None;
    }
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MoveMeta {
    /// Estado alterado que puede aplicar: "burn", "paralysis", "none", etc.
    #[serde(default = "default_ailment")]
    pub ailment: String,
    /// Probabilidad de aplicar el estado alterado (0-100)
    #[serde(default)]
    pub ailment_chance: u8,
    /// Tasa de crítico (0-4)
    #[serde(default)]
    pub crit_rate: u8,
    /// Porcentaje de daño que cura al usuario (drenaje). Negativo es recoil.
    #[serde(default)]
    pub drain: i8,
    /// Probabilidad de causar retroceso (0-100)
    #[serde(default)]
    pub flinch_chance: u8,
    /// Probabilidad de que ocurran los cambios de stats (0-100)
    #[serde(default)]
    pub stat_chance: u8,
    /// Porcentaje de curación (0-100)
    #[serde(default)]
    pub healing: i8,
    /// Número mínimo de golpes (para movimientos multi-hit como Double Slap)
    #[serde(default)]
    pub min_hits: Option<u8>,
    /// Número máximo de golpes
    #[serde(default)]
    pub max_hits: Option<u8>,
    /// Turnos mínimos (para movimientos con carga como Solar Beam)
    #[serde(default)]
    pub min_turns: Option<u8>,
    /// Turnos máximos
    #[serde(default)]
    pub max_turns: Option<u8>,
    /// Si el movimiento hace contacto físico (afecta habilidades como Rough Skin, Static, Tough Claws)
    #[serde(default)]
    pub makes_contact: bool,
    /// Si el movimiento fuerza al objetivo a cambiar (Dragon Tail, Roar, etc.)
    #[serde(default)]
    pub forces_switch: bool,
}

impl Default for MoveMeta {
    fn default() -> Self {
        Self {
            ailment: "none".to_string(),
            ailment_chance: 0,
            crit_rate: 0,
            drain: 0,
            flinch_chance: 0,
            stat_chance: 0,
            healing: 0,
            min_hits: None,
            max_hits: None,
            min_turns: None,
            max_turns: None,
            makes_contact: false,
            forces_switch: false,
        }
    }
}

fn default_ailment() -> String {
    "none".to_string()
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

/// Movimiento aprendido por un Pokémon con su PP actual y máximo
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LearnedMove {
    /// ID del movimiento (ej: "tackle", "scratch")
    pub move_id: String,
    /// Puntos de poder actuales (PP restantes)
    pub current_pp: u8,
    /// Puntos de poder máximos (se obtiene de MoveData al aprenderlo)
    pub max_pp: u8,
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
    /// List of possible ability IDs for this species (e.g., "blaze", "intimidate")
    #[serde(default)]
    pub possible_abilities: Vec<String>,
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
    /// Movimientos aprendidos con su información de PP
    pub learned_moves: Vec<LearnedMove>,
    /// Instancias de movimientos (para randomización de tipos/poder)
    /// Mantenemos esto para compatibilidad con el sistema de randomización
    #[serde(default)]
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
    /// ID de la habilidad activa de este Pokémon (e.g., "blaze", "intimidate")
    pub ability: String,
    /// ID del objeto que sostiene este Pokémon (e.g., "leftovers", "life-orb")
    #[serde(default)]
    pub held_item: Option<String>,
    
    /// Stages de stats en batalla (None fuera de batalla, Some con valores base al entrar)
    /// Representa cambios temporales de stats durante la batalla (-6 a +6)
    #[serde(default)]
    pub battle_stages: Option<StatStages>,
    
    /// Estados volátiles en batalla (None fuera de batalla, Some al entrar)
    /// Estos estados se resetean al final de cada turno o al salir de batalla
    #[serde(default)]
    pub volatile_status: Option<VolatileStatus>,
    
    pub individual_values: Stats,
    pub effort_values: Stats,
    
    /// Stats calculados (Base + IV + EV + Nivel) = VIDA MÁXIMA
    pub base_computed_stats: Stats,
    
    pub randomized_profile: RandomizedProfile,
}

impl PokemonInstance {
    /// Restaura completamente el Pokémon a su estado óptimo
    /// Restaura toda la vida y elimina cualquier condición de estado
    /// También resetea los battle_stages y volatile_status si están presentes
    pub fn full_restore(&mut self) {
        self.current_hp = self.base_computed_stats.hp;
        self.status_condition = None;
        if let Some(ref mut stages) = self.battle_stages {
            *stages = StatStages::new();
        }
        if let Some(ref mut volatile) = self.volatile_status {
            volatile.reset_all();
        }
    }

    /// Inicializa los battle_stages y volatile_status al entrar en batalla
    /// Debe llamarse cuando el Pokémon entra en combate
    pub fn init_battle_stages(&mut self) {
        self.battle_stages = Some(StatStages::new());
        self.volatile_status = Some(VolatileStatus::new());
    }

    /// Resetea los battle_stages y volatile_status al salir de batalla
    /// Debe llamarse cuando el Pokémon sale de combate
    pub fn reset_battle_stages(&mut self) {
        self.battle_stages = None;
        self.volatile_status = None;
    }

    /// Resetea los estados volátiles al inicio de un nuevo turno
    /// Debe llamarse al inicio de cada turno para resetear flinch, etc.
    pub fn reset_turn_volatiles(&mut self) {
        if let Some(ref mut volatile) = self.volatile_status {
            volatile.reset_turn();
        }
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
    /// Requiere el max_pp del movimiento para inicializar correctamente
    pub fn learn_new_move(&mut self, new_move_id: String, max_pp: u8) {
        self.randomized_profile.learned_moves.push(LearnedMove {
            move_id: new_move_id.clone(),
            current_pp: max_pp,
            max_pp,
        });
        // También mantener la lista de MoveInstance para compatibilidad
        self.randomized_profile.moves.push(MoveInstance {
            template_id: new_move_id,
            randomized_type: None,
            power_variation: None,
            accuracy_multiplier: None,
        });
    }

    /// Obtiene los movimientos activos aprendidos (primeros 4) para usar en batalla
    /// Retorna una referencia a los primeros 4 movimientos (o menos si hay menos de 4)
    pub fn get_active_learned_moves(&self) -> &[LearnedMove] {
        let moves = &self.randomized_profile.learned_moves;
        let active_count = moves.len().min(4);
        &moves[0..active_count]
    }

    /// Obtiene los movimientos activos (primeros 4) para usar en batalla
    /// Retorna una referencia a los primeros 4 movimientos (o menos si hay menos de 4)
    /// @deprecated: Usar get_active_learned_moves en su lugar
    pub fn get_active_moves(&self) -> &[MoveInstance] {
        let moves = &self.randomized_profile.moves;
        let active_count = moves.len().min(4);
        &moves[0..active_count]
    }
    
    /// Obtiene un movimiento aprendido por su ID
    pub fn get_learned_move(&mut self, move_id: &str) -> Option<&mut LearnedMove> {
        self.randomized_profile.learned_moves.iter_mut()
            .find(|m| m.move_id == move_id)
    }
}

