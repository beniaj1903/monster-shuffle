//! Módulo de compatibilidad para context
//!
//! Este módulo re-exporta todas las funciones del nuevo
//! systems/move_system/executor.rs para mantener compatibilidad
//! con el código existente.
//!
//! MIGRADO: El código real ahora está en systems/move_system/executor.rs

pub use super::systems::move_system::executor::*;
