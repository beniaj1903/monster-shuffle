//! Sistema de validaciones
//!
//! Este sistema es responsable de:
//! - Validar movimientos (PP, estados)
//! - Validar estados de Pokémon
//! - Resetear flags de turno
//! - Gestionar PP de movimientos

pub mod state_resetter;
pub mod pp_manager;

// Re-exportar funciones principales
pub use state_resetter::reset_turn_flags;
pub use pp_manager::{
    initialize_move_pp,
    consume_move_pp,
    has_moves_with_pp,
    create_struggle_move,
};

// NOTA: can_pokemon_move y check_ailment_success permanecen en checks.rs
// debido a que requieren acceso a StdRng y lógica de batalla específica
