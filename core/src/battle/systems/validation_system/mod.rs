//! Sistema de validaciones
//!
//! Este sistema es responsable de:
//! - Validar movimientos (PP, estados)
//! - Validar estados de Pokémon
//! - Resetear flags de turno

pub mod state_resetter;

// Re-exportar funciones principales
pub use state_resetter::reset_turn_flags;

// NOTA: can_pokemon_move, check_ailment_success, initialize_move_pp, consume_move_pp, has_moves_with_pp
// permanecen en checks.rs por ahora debido a diferencias en las firmas y lógica específica.
