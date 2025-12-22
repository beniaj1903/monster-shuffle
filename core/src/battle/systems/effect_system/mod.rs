//! Sistema de efectos de campo y estados
//!
//! Este sistema es responsable de:
//! - Gestionar efectos de clima (weather)
//! - Gestionar efectos de terreno (terrain)
//! - Aplicar efectos de estados alterados
//! - Procesar efectos residuales

pub mod effects_handler;

// Re-exportar funciones principales
pub use effects_handler::{
    apply_weather_residuals,
    apply_residual_effects,
    is_grounded,
    check_ability_immunity,
    modify_offensive_stat_by_ability,
    trigger_on_entry_abilities,
};

// NOTA: process_end_of_turn_residuals permanece en pipeline.rs por ahora
