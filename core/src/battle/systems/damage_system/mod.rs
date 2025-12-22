//! Sistema de cálculo de daño
//!
//! Este sistema es responsable de:
//! - Calcular daño base
//! - Aplicar modificadores (STAB, weather, terrain, abilities)
//! - Calcular críticos
//! - Calcular efectividad de tipos

pub mod calculator;

// Re-exportar funciones principales
pub use calculator::{
    calculate_damage,
    check_critical_hit,
    calculate_hit_count,
    get_type_effectiveness,
    get_effective_speed,
};
