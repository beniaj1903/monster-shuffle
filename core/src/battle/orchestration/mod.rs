//! Capa de orquestación del motor de batalla
//!
//! Este módulo contiene el código que coordina todos los sistemas de batalla.
//! El motor principal (BattleEngine) orquesta el flujo completo de un turno.

// TODO: Fase 3 - Crear estos módulos
// pub mod battle_engine;        // Motor principal (versión simplificada de pipeline.rs)
// pub mod turn_coordinator;     // Coordinador de turnos
// pub mod phase_manager;        // Gestor de fases del turno
// pub mod outcome_resolver;     // Resolución de victorias/derrotas

// Por ahora, re-exportamos la función execute_turn del pipeline actual
// para mantener la compatibilidad
pub use super::pipeline::execute_turn;
