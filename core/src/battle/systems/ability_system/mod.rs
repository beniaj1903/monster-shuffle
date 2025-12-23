//! Sistema de habilidades Pokémon
//!
//! Este sistema es responsable de:
//! - Procesar hooks de habilidades
//! - Activar habilidades según triggers
//! - Modificar stats, prioridad, velocidad según habilidades

pub mod registry;
pub mod triggers;
pub mod processor;

#[cfg(test)]
mod tests;

// Re-exportar tipos y funciones principales del registry
pub use registry::{
    AbilityTrigger, AbilityEffect, AbilityHook, StatChangeTarget, HealCondition,
    get_ability_hooks,
};

// Re-exportar funciones del processor
pub use processor::{
    get_speed_with_abilities,
    get_priority_with_abilities,
};

// NOTA: handle_entry_hazards, apply_end_of_turn_abilities
// permanecen en pipeline.rs por ahora debido a su complejidad y dependencias
