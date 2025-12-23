//! Sistema de Items
//!
//! Este módulo maneja todos los efectos de items equipados en batalla.
//! Incluye items como Choice Band, Life Orb, berries, etc.

pub mod item_effects;
pub mod item_triggers;
pub mod item_processor;

#[cfg(test)]
mod tests;

// Re-exportar tipos principales
pub use item_effects::{ItemEffect, apply_item_effect};
pub use item_triggers::{ItemTrigger, check_item_trigger};
pub use item_processor::{ItemProcessor, process_items_before_damage, process_items_after_damage};
