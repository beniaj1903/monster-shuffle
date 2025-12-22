//! Sistema de Inteligencia Artificial
//!
//! Este sistema es responsable de:
//! - Seleccionar movimientos para oponentes
//! - Implementar estrategias de IA
//! - Tomar decisiones de cambio de Pokémon

pub mod selector;

// Re-exportar función principal
pub use selector::select_ai_move;
