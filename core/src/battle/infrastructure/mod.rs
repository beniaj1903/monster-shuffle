//! Capa de infraestructura
//!
//! Este módulo contiene utilidades y helpers que dan soporte a los sistemas de batalla.

pub mod pokemon_accessor;
pub mod move_data_loader;

// Re-exportar funciones comunes
pub use pokemon_accessor::{get_pokemon, get_pokemon_mut, get_team_index, is_pokemon_alive};
pub use move_data_loader::resolve_move_data;
