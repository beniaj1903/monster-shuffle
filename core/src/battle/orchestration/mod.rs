//! Capa de orquestación del motor de batalla
//!
//! Este módulo contiene el código que coordina todos los sistemas de batalla.
//! El motor principal (BattleEngine) orquesta el flujo completo de un turno.

pub mod battle_engine;

// Re-exportar funciones principales
pub use battle_engine::{
    execute_turn,
    determine_player_outcome,
    determine_enemy_outcome,
    check_battle_state,
};

// TODO: Fase 4+ - Migrar más módulos de pipeline.rs
// pub mod turn_coordinator;     // Coordinador de turnos
// pub mod phase_manager;        // Gestor de fases del turno
