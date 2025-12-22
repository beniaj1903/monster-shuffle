//! Sistema de acciones de batalla
//!
//! Este sistema es responsable de:
//! - Recopilar acciones de jugadores y oponentes
//! - Ordenar acciones por prioridad y velocidad
//! - Ejecutar acciones en el orden correcto

pub mod models;

// Re-exportar modelo principal
pub use models::ActionCandidate;

// NOTA: collect_action_candidates, sort_candidates y execute_single_action
// permanecen en pipeline.rs por ahora debido a dependencias con funciones
// de habilidades que aún no están migradas (get_speed_with_abilities, get_priority_with_abilities)
