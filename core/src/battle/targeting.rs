//! Módulo de compatibilidad para targeting
//!
//! Este módulo re-exporta todas las funciones del nuevo
//! systems/move_system/targeting.rs para mantener compatibilidad
//! con el código existente.
//!
//! MIGRADO: El código real ahora está en systems/move_system/targeting.rs

pub use super::systems::move_system::targeting::*;
