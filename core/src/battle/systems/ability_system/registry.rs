/// Sistema de Habilidades (Abilities) para Pokémon
///
/// Este módulo implementa un sistema extensible de habilidades pasivas que afectan
/// diferentes aspectos del combate. Las habilidades se organizan en "hooks" que se
/// ejecutan en momentos específicos de la batalla.
///
/// # Arquitectura
///
/// - **AbilityTrigger**: Define CUÁNDO se activa una habilidad
/// - **AbilityEffect**: Define QUÉ hace la habilidad cuando se activa
/// - **get_ability_hooks**: Registry central que mapea ability_id -> lista de hooks
///
/// # Ejemplo de uso
///
/// ```rust
/// let hooks = get_ability_hooks("intimidate");
/// for hook in hooks {
///     if hook.trigger == AbilityTrigger::OnEntry {
///         // Aplicar el efecto
///     }
/// }
/// ```

use crate::models::{PokemonType, WeatherType, TerrainType, StatusCondition};

/// Momento en el que se activa una habilidad
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilityTrigger {
    /// Al entrar al campo (ej: Intimidate, Drought)
    OnEntry,
    /// Antes de calcular daño (ej: Huge Power, Levitate)
    BeforeDamage,
    /// Después de calcular daño (ej: Rough Skin, Iron Barbs)
    AfterDamage,
    /// Al recibir daño (ej: Stamina, Weak Armor)
    OnReceiveDamage,
    /// Al ser golpeado por contacto (ej: Static, Flame Body)
    OnContact,
    /// Al final del turno (ej: Speed Boost, Moody)
    EndOfTurn,
    /// Al cambiar de Pokémon (ej: Regenerator)
    OnSwitch,
    /// Modificación de prioridad (ej: Prankster, Gale Wings)
    ModifyPriority,
    /// Modificación de velocidad (ej: Chlorophyll, Swift Swim)
    ModifySpeed,
}

/// Tipo de objetivo para efectos de stat change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatChangeTarget {
    /// Afecta al usuario de la habilidad
    User,
    /// Afecta a todos los oponentes
    AllOpponents,
    /// Afecta a un oponente específico
    SingleOpponent,
    /// Afecta a los aliados
    Allies,
}

/// Efecto de una habilidad
#[derive(Debug, Clone)]
pub enum AbilityEffect {
    /// Establece un clima al entrar (Drought, Drizzle, Sand Stream, Snow Warning)
    SetWeather {
        weather: WeatherType,
        duration: u8, // 0 = infinito (con objeto)
    },

    /// Establece un terreno al entrar (Electric Surge, Grassy Surge, etc.)
    SetTerrain {
        terrain: TerrainType,
        duration: u8,
    },

    /// Modifica un stat al entrar (Intimidate, Download)
    ModifyStatOnEntry {
        stat: String, // "attack", "defense", "speed", etc.
        stages: i8,   // -1 para Intimidate, +1 para Download
        target: StatChangeTarget,
    },

    /// Inmunidad a un tipo + curación opcional (Levitate, Volt Absorb, Flash Fire)
    TypeImmunity {
        immune_type: PokemonType,
        heal_percent: Option<f32>, // None = inmunidad, Some(0.25) = cura 25%
        boost_on_absorb: Option<(String, i8)>, // Para Flash Fire: ("special_attack", 1)
    },

    /// Multiplica una stat pasivamente (Huge Power x2 Attack, Fur Coat x2 Defense)
    MultiplyBaseStat {
        stat: String,
        multiplier: f32,
    },

    /// Potencia movimientos de cierto tipo cuando HP < threshold (Blaze, Torrent, Overgrow)
    BoostTypeAtLowHP {
        move_type: PokemonType,
        multiplier: f32, // 1.5 para Blaze/Torrent/Overgrow
        hp_threshold: f32, // 0.33 = 33% HP
    },

    /// Potencia todos los movimientos de contacto (Tough Claws, Punching Glove)
    BoostContactMoves {
        multiplier: f32, // 1.3 para Tough Claws
    },

    /// Multiplica la velocidad bajo ciertas condiciones (Chlorophyll, Swift Swim, Slush Rush)
    MultiplySpeedInWeather {
        weather: WeatherType,
        multiplier: f32, // 2.0
    },

    /// Multiplica la velocidad bajo terreno (Surge Surfer)
    MultiplySpeedInTerrain {
        terrain: TerrainType,
        multiplier: f32,
    },

    /// Modifica la prioridad de movimientos de cierto tipo (Prankster, Gale Wings)
    ModifyMovePriority {
        move_type: Option<PokemonType>, // None = todos los movimientos
        priority_boost: i8, // +1 para Prankster
        condition: Option<PriorityCondition>, // Para Gale Wings: HP completo
    },

    /// Aumenta precisión de movimientos (Compound Eyes, Victory Star)
    ModifyAccuracy {
        multiplier: f32, // 1.3 para Compound Eyes
    },

    /// Aumenta tasa de crítico (Super Luck)
    ModifyCritRate {
        stages: i8, // +1 para Super Luck
    },

    /// Cambia stats al recibir daño (Stamina +1 Def, Weak Armor -1 Def +2 Spe)
    ModifyStatsOnHit {
        changes: Vec<(String, i8)>, // [("defense", 1)] para Stamina
    },

    /// Inflige estado al contacto (Static 30% parálisis, Flame Body 30% quemadura)
    InflictStatusOnContact {
        status: StatusCondition,
        chance: f32, // 0.3 = 30%
    },

    /// Daño residual al atacante en contacto (Rough Skin, Iron Barbs)
    DamageAttackerOnContact {
        damage_fraction: f32, // 1/8 = 0.125
    },

    /// Cura HP al final del turno (Rain Dish)
    HealEndOfTurn {
        fraction: f32, // 1/16 = 0.0625
        condition: Option<HealCondition>,
    },

    /// Previene reducción de stats (Clear Body, White Smoke, Hyper Cutter)
    PreventStatLoss {
        stats: Vec<String>, // Vec vacío = todas las stats
    },

    /// Previene estados de salud (Immunity, Limber, Oblivious, Own Tempo)
    PreventStatus {
        statuses: Vec<StatusCondition>, // Vec vacío = todos los estados
    },

    /// Regenera HP al cambiar de Pokémon (Regenerator)
    HealOnSwitch {
        fraction: f32, // 1/3 = 0.33
    },

    /// Aumenta stat al final del turno (Speed Boost, Moody)
    BoostStatEndOfTurn {
        stat: String,
        stages: i8,
    },

    /// Cambia tipo del Pokémon (Protean, Libero)
    ChangeTypeBeforeMove {
        to_move_type: bool, // true = cambia al tipo del movimiento usado
    },

    /// Ignora habilidades del oponente (Mold Breaker, Teravolt, Turboblaze)
    IgnoreOpponentAbility,

    /// Reduce daño super efectivo (Solid Rock, Filter)
    ReduceSuperEffectiveDamage {
        multiplier: f32, // 0.75 = reduce a 75% (25% de reducción)
    },

    /// Potencia movimientos débiles (Technician)
    BoostWeakMoves {
        power_threshold: u16, // 60 para Technician
        multiplier: f32,      // 1.5 para Technician
    },

    /// Elimina efectos secundarios y aumenta daño (Sheer Force)
    RemoveSecondaryEffects {
        damage_multiplier: f32, // 1.3 para Sheer Force
    },

    /// Lógica única personalizada (para habilidades muy complejas)
    Custom {
        ability_id: String,
    },
}

/// Condición para modificar prioridad
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriorityCondition {
    /// Solo funciona con HP completo (Gale Wings)
    FullHP,
    /// Solo funciona si el Pokémon está envenenado (Quick Draw - ejemplo)
    Poisoned,
}

/// Condición para curación
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealCondition {
    /// Solo bajo lluvia (Rain Dish)
    Weather(WeatherType),
    /// Solo bajo terreno
    Terrain(TerrainType),
}

/// Un "hook" de habilidad: cuándo se activa + qué hace
#[derive(Debug, Clone)]
pub struct AbilityHook {
    pub trigger: AbilityTrigger,
    pub effect: AbilityEffect,
}

impl AbilityHook {
    pub fn new(trigger: AbilityTrigger, effect: AbilityEffect) -> Self {
        Self { trigger, effect }
    }
}

/// Registry central: Mapea ability_id -> lista de hooks
///
/// Esta función es el corazón del sistema. Para añadir una nueva habilidad,
/// simplemente agrega un nuevo case aquí con sus hooks correspondientes.
///
/// # Ejemplo
///
/// ```rust
/// "intimidate" => vec![
///     AbilityHook::new(
///         AbilityTrigger::OnEntry,
///         AbilityEffect::ModifyStatOnEntry {
///             stat: "attack".to_string(),
///             stages: -1,
///             target: StatChangeTarget::AllOpponents,
///         },
///     ),
/// ],
/// ```
pub fn get_ability_hooks(ability_id: &str) -> Vec<AbilityHook> {
    match ability_id {
        // ============================================================
        // CLIMA Y TERRENO (Weather/Terrain Summon)
        // ============================================================
        "drought" => vec![AbilityHook::new(
            AbilityTrigger::OnEntry,
            AbilityEffect::SetWeather {
                weather: WeatherType::Sun,
                duration: 5,
            },
        )],

        "drizzle" => vec![AbilityHook::new(
            AbilityTrigger::OnEntry,
            AbilityEffect::SetWeather {
                weather: WeatherType::Rain,
                duration: 5,
            },
        )],

        "sand-stream" => vec![AbilityHook::new(
            AbilityTrigger::OnEntry,
            AbilityEffect::SetWeather {
                weather: WeatherType::Sandstorm,
                duration: 5,
            },
        )],

        "snow-warning" => vec![AbilityHook::new(
            AbilityTrigger::OnEntry,
            AbilityEffect::SetWeather {
                weather: WeatherType::Hail,
                duration: 5,
            },
        )],

        "electric-surge" => vec![AbilityHook::new(
            AbilityTrigger::OnEntry,
            AbilityEffect::SetTerrain {
                terrain: TerrainType::Electric,
                duration: 5,
            },
        )],

        "grassy-surge" => vec![AbilityHook::new(
            AbilityTrigger::OnEntry,
            AbilityEffect::SetTerrain {
                terrain: TerrainType::Grassy,
                duration: 5,
            },
        )],

        "misty-surge" => vec![AbilityHook::new(
            AbilityTrigger::OnEntry,
            AbilityEffect::SetTerrain {
                terrain: TerrainType::Misty,
                duration: 5,
            },
        )],

        "psychic-surge" => vec![AbilityHook::new(
            AbilityTrigger::OnEntry,
            AbilityEffect::SetTerrain {
                terrain: TerrainType::Psychic,
                duration: 5,
            },
        )],

        // ============================================================
        // STAT MODIFICATION ON ENTRY
        // ============================================================
        "intimidate" => vec![AbilityHook::new(
            AbilityTrigger::OnEntry,
            AbilityEffect::ModifyStatOnEntry {
                stat: "attack".to_string(),
                stages: -1,
                target: StatChangeTarget::AllOpponents,
            },
        )],

        // Download necesita lógica custom ya que debe comparar Defense vs Sp. Defense
        "download" => vec![AbilityHook::new(
            AbilityTrigger::OnEntry,
            AbilityEffect::Custom {
                ability_id: "download".to_string(),
            },
        )],

        // ============================================================
        // INMUNIDADES (Type Immunity)
        // ============================================================
        "levitate" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::TypeImmunity {
                immune_type: PokemonType::Ground,
                heal_percent: None,
                boost_on_absorb: None,
            },
        )],

        "volt-absorb" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::TypeImmunity {
                immune_type: PokemonType::Electric,
                heal_percent: Some(0.25),
                boost_on_absorb: None,
            },
        )],

        "water-absorb" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::TypeImmunity {
                immune_type: PokemonType::Water,
                heal_percent: Some(0.25),
                boost_on_absorb: None,
            },
        )],

        "flash-fire" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::TypeImmunity {
                immune_type: PokemonType::Fire,
                heal_percent: None,
                boost_on_absorb: Some(("special_attack".to_string(), 1)),
            },
        )],

        "sap-sipper" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::TypeImmunity {
                immune_type: PokemonType::Grass,
                heal_percent: None,
                boost_on_absorb: Some(("attack".to_string(), 1)),
            },
        )],

        // ============================================================
        // STAT MULTIPLIERS (Passive)
        // ============================================================
        "huge-power" | "pure-power" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::MultiplyBaseStat {
                stat: "attack".to_string(),
                multiplier: 2.0,
            },
        )],

        "fur-coat" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::MultiplyBaseStat {
                stat: "defense".to_string(),
                multiplier: 2.0,
            },
        )],

        // ============================================================
        // LOW HP BOOSTS (Blaze, Torrent, Overgrow)
        // ============================================================
        "blaze" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::BoostTypeAtLowHP {
                move_type: PokemonType::Fire,
                multiplier: 1.5,
                hp_threshold: 0.33,
            },
        )],

        "torrent" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::BoostTypeAtLowHP {
                move_type: PokemonType::Water,
                multiplier: 1.5,
                hp_threshold: 0.33,
            },
        )],

        "overgrow" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::BoostTypeAtLowHP {
                move_type: PokemonType::Grass,
                multiplier: 1.5,
                hp_threshold: 0.33,
            },
        )],

        "swarm" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::BoostTypeAtLowHP {
                move_type: PokemonType::Bug,
                multiplier: 1.5,
                hp_threshold: 0.33,
            },
        )],

        // ============================================================
        // CONTACT MOVE BOOSTS
        // ============================================================
        "tough-claws" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::BoostContactMoves { multiplier: 1.3 },
        )],

        // ============================================================
        // WEATHER SPEED BOOSTS
        // ============================================================
        "chlorophyll" => vec![AbilityHook::new(
            AbilityTrigger::ModifySpeed,
            AbilityEffect::MultiplySpeedInWeather {
                weather: WeatherType::Sun,
                multiplier: 2.0,
            },
        )],

        "swift-swim" => vec![AbilityHook::new(
            AbilityTrigger::ModifySpeed,
            AbilityEffect::MultiplySpeedInWeather {
                weather: WeatherType::Rain,
                multiplier: 2.0,
            },
        )],

        "sand-rush" => vec![AbilityHook::new(
            AbilityTrigger::ModifySpeed,
            AbilityEffect::MultiplySpeedInWeather {
                weather: WeatherType::Sandstorm,
                multiplier: 2.0,
            },
        )],

        "slush-rush" => vec![AbilityHook::new(
            AbilityTrigger::ModifySpeed,
            AbilityEffect::MultiplySpeedInWeather {
                weather: WeatherType::Hail,
                multiplier: 2.0,
            },
        )],

        // ============================================================
        // TERRAIN SPEED BOOSTS
        // ============================================================
        "surge-surfer" => vec![AbilityHook::new(
            AbilityTrigger::ModifySpeed,
            AbilityEffect::MultiplySpeedInTerrain {
                terrain: TerrainType::Electric,
                multiplier: 2.0,
            },
        )],

        // ============================================================
        // PRIORITY MODIFICATION
        // ============================================================
        "prankster" => vec![AbilityHook::new(
            AbilityTrigger::ModifyPriority,
            AbilityEffect::ModifyMovePriority {
                move_type: None, // Solo movimientos de estado (se verifica en otro lado)
                priority_boost: 1,
                condition: None,
            },
        )],

        "gale-wings" => vec![AbilityHook::new(
            AbilityTrigger::ModifyPriority,
            AbilityEffect::ModifyMovePriority {
                move_type: Some(PokemonType::Flying),
                priority_boost: 1,
                condition: Some(PriorityCondition::FullHP),
            },
        )],

        // ============================================================
        // ACCURACY/CRIT MODIFICATION
        // ============================================================
        "compound-eyes" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::ModifyAccuracy { multiplier: 1.3 },
        )],

        "super-luck" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::ModifyCritRate { stages: 1 },
        )],

        // ============================================================
        // ON-HIT EFFECTS
        // ============================================================
        "stamina" => vec![AbilityHook::new(
            AbilityTrigger::OnReceiveDamage,
            AbilityEffect::ModifyStatsOnHit {
                changes: vec![("defense".to_string(), 1)],
            },
        )],

        "weak-armor" => vec![AbilityHook::new(
            AbilityTrigger::OnReceiveDamage,
            AbilityEffect::ModifyStatsOnHit {
                changes: vec![
                    ("defense".to_string(), -1),
                    ("speed".to_string(), 2),
                ],
            },
        )],

        // ============================================================
        // ON-CONTACT EFFECTS
        // ============================================================
        "static" => vec![AbilityHook::new(
            AbilityTrigger::OnContact,
            AbilityEffect::InflictStatusOnContact {
                status: StatusCondition::Paralysis,
                chance: 0.3,
            },
        )],

        "flame-body" => vec![AbilityHook::new(
            AbilityTrigger::OnContact,
            AbilityEffect::InflictStatusOnContact {
                status: StatusCondition::Burn,
                chance: 0.3,
            },
        )],

        "rough-skin" | "iron-barbs" => vec![AbilityHook::new(
            AbilityTrigger::OnContact,
            AbilityEffect::DamageAttackerOnContact {
                damage_fraction: 0.125, // 1/8
            },
        )],

        // ============================================================
        // END OF TURN EFFECTS
        // ============================================================
        "speed-boost" => vec![AbilityHook::new(
            AbilityTrigger::EndOfTurn,
            AbilityEffect::BoostStatEndOfTurn {
                stat: "speed".to_string(),
                stages: 1,
            },
        )],

        "rain-dish" => vec![AbilityHook::new(
            AbilityTrigger::EndOfTurn,
            AbilityEffect::HealEndOfTurn {
                fraction: 0.0625, // 1/16
                condition: Some(HealCondition::Weather(WeatherType::Rain)),
            },
        )],

        // ============================================================
        // PROTECTION ABILITIES
        // ============================================================
        "clear-body" | "white-smoke" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::PreventStatLoss {
                stats: vec![], // Vacío = todas las stats
            },
        )],

        "hyper-cutter" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::PreventStatLoss {
                stats: vec!["attack".to_string()],
            },
        )],

        "immunity" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::PreventStatus {
                statuses: vec![StatusCondition::Poison, StatusCondition::BadPoison],
            },
        )],

        "limber" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::PreventStatus {
                statuses: vec![StatusCondition::Paralysis],
            },
        )],

        // ============================================================
        // ON-SWITCH EFFECTS
        // ============================================================
        "regenerator" => vec![AbilityHook::new(
            AbilityTrigger::OnSwitch,
            AbilityEffect::HealOnSwitch {
                fraction: 0.33, // 1/3
            },
        )],

        // ============================================================
        // SPECIAL ABILITIES
        // ============================================================
        "mold-breaker" | "teravolt" | "turboblaze" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::IgnoreOpponentAbility,
        )],

        // ============================================================
        // FASE 2.2: ABILITIES CRÍTICAS VGC
        // ============================================================

        // Solid Rock / Filter: Reduce daño super efectivo en 25%
        "solid-rock" | "filter" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::ReduceSuperEffectiveDamage {
                multiplier: 0.75, // 25% de reducción
            },
        )],

        // Technician: Potencia movimientos con poder ≤ 60 en 1.5x
        "technician" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::BoostWeakMoves {
                power_threshold: 60,
                multiplier: 1.5,
            },
        )],

        // Sheer Force: Elimina efectos secundarios y aumenta daño en 30%
        "sheer-force" => vec![AbilityHook::new(
            AbilityTrigger::BeforeDamage,
            AbilityEffect::RemoveSecondaryEffects {
                damage_multiplier: 1.3,
            },
        )],

        // ============================================================
        // DEFAULT: Sin habilidad o no implementada
        // ============================================================
        _ => vec![],
    }
}

/// Helper: Convierte un string de tipo de Pokémon a PokemonType enum
pub fn parse_pokemon_type(type_str: &str) -> Option<PokemonType> {
    match type_str.to_lowercase().as_str() {
        "normal" => Some(PokemonType::Normal),
        "fire" => Some(PokemonType::Fire),
        "water" => Some(PokemonType::Water),
        "grass" => Some(PokemonType::Grass),
        "electric" => Some(PokemonType::Electric),
        "ice" => Some(PokemonType::Ice),
        "fighting" => Some(PokemonType::Fighting),
        "poison" => Some(PokemonType::Poison),
        "ground" => Some(PokemonType::Ground),
        "flying" => Some(PokemonType::Flying),
        "psychic" => Some(PokemonType::Psychic),
        "bug" => Some(PokemonType::Bug),
        "rock" => Some(PokemonType::Rock),
        "ghost" => Some(PokemonType::Ghost),
        "dragon" => Some(PokemonType::Dragon),
        "dark" => Some(PokemonType::Dark),
        "steel" => Some(PokemonType::Steel),
        "fairy" => Some(PokemonType::Fairy),
        _ => None,
    }
}
