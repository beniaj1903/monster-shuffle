//! Módulo de compatibilidad para mechanics
//!
//! Este módulo re-exporta todas las funciones del nuevo
//! systems/damage_system/calculator.rs para mantener compatibilidad
//! con el código existente.
//!
//! MIGRADO: El código real ahora está en systems/damage_system/calculator.rs

pub use super::systems::damage_system::calculator::*;
