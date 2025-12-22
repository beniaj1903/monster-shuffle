//! Sistema de ejecución de movimientos
//!
//! Este sistema es responsable de:
//! - Ejecutar movimientos individuales
//! - Resolver objetivos (targeting)
//! - Aplicar efectos de movimientos
//! - Gestionar PP

pub mod executor;
pub mod targeting;

// Re-exportar tipos y funciones principales
pub use executor::BattleContext;
pub use targeting::resolve_targets;

// NOTA: initialize_move_pp, consume_move_pp, has_moves_with_pp, create_struggle_move
// permanecen en checks.rs por ahora debido a compatibilidad de firmas
