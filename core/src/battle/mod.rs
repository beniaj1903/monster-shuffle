//! Módulo de batalla - Sistema de combate Pokémon
//! 
//! Este módulo contiene toda la lógica de batalla dividida en submódulos:
//! - `context`: Contexto de batalla para procesar ataques individuales
//! - `targeting`: Resolución de objetivos en combates Singles/Doubles
//! - `mechanics`: Cálculos matemáticos (daño, críticos, efectividad)
//! - `effects`: Aplicación de efectos (estados, stats, residuales)
//! - `pipeline`: Orquestación del flujo de turnos
//! - `checks`: Validaciones (PP, estados, grounded)

pub mod context;
pub mod targeting;
pub mod mechanics;
pub mod effects;
pub mod pipeline;
pub mod checks;

#[cfg(test)]
mod tests;

// Re-exportar tipos principales para compatibilidad
pub use context::BattleContext;
pub use targeting::resolve_targets;
pub use mechanics::{
    check_critical_hit,
    calculate_hit_count,
};
pub use effects::{
    trigger_on_entry_abilities,
    check_ability_immunity,
    is_grounded,
    modify_offensive_stat_by_ability,
    apply_weather_residuals,
    apply_residual_effects,
};
pub use checks::{
    check_ailment_success,
    initialize_move_pp,
    consume_move_pp,
    has_moves_with_pp,
    create_struggle_move,
};
pub use pipeline::{
    execute_turn,
    execute_enemy_attack,
};

use serde::{Deserialize, Serialize};

/// Resultado de la batalla después de un turno
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum BattleOutcome {
    /// Nadie murió, la batalla continúa
    Continue,
    /// Todos los enemigos debilitados - Jugador ganó
    PlayerWon,
    /// Todos los Pokémon del jugador debilitados - Jugador perdió
    PlayerLost,
    /// El Pokémon activo del jugador murió, pero le quedan otros vivos
    PlayerMustSwitch,
    /// El enemigo murió y sacó uno nuevo (útil para logs/animación)
    EnemySwitched,
}

/// Resultado de ejecutar un turno de batalla
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurnResult {
    /// Narración de lo que ocurrió en el turno
    pub logs: Vec<String>,
    /// Daño infligido por el jugador
    pub player_damage_dealt: u16,
    /// Daño infligido por el enemigo
    pub enemy_damage_dealt: u16,
    /// Resultado de la batalla después de este turno
    pub outcome: BattleOutcome,
}

impl TurnResult {
    /// Crea un nuevo TurnResult vacío
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            player_damage_dealt: 0,
            enemy_damage_dealt: 0,
            outcome: BattleOutcome::Continue,
        }
    }
}

impl Default for TurnResult {
    fn default() -> Self {
        Self::new()
    }
}

