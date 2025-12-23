//! Sistema de redirección de ataques
//!
//! Este módulo maneja movimientos que redirigen ataques en batalla doubles:
//! - Follow Me: Redirige todos los single-target moves del oponente
//! - Rage Powder: Como Follow Me pero no afecta Grass types
//! - Spotlight: Marca un objetivo específico para redirección
//! - Ally Switch: Intercambia posiciones con el aliado

pub mod processor;

#[cfg(test)]
mod tests;

pub use processor::{
    apply_redirection,
    set_follow_me,
    set_rage_powder,
    set_spotlight,
    ally_switch,
    clear_redirection,
};
