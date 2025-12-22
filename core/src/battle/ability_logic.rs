//! Módulo de compatibilidad para ability_logic
//!
//! Este módulo re-exporta todas las funciones y tipos del nuevo
//! systems/ability_system/registry.rs para mantener compatibilidad
//! con el código existente.
//!
//! MIGRADO: El código real ahora está en systems/ability_system/registry.rs

pub use super::systems::ability_system::registry::*;
