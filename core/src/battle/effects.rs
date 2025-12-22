//! Módulo de compatibilidad para effects
//!
//! Este módulo re-exporta todas las funciones del nuevo
//! systems/effect_system/effects_handler.rs para mantener compatibilidad
//! con el código existente.
//!
//! MIGRADO: El código real ahora está en systems/effect_system/effects_handler.rs

pub use super::systems::effect_system::effects_handler::*;
