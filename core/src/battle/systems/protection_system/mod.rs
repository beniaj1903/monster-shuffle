//! Sistema de Protecciones Avanzadas
//!
//! Este módulo implementa las mecánicas de protección avanzadas de VGC:
//! - Wide Guard: Protege al equipo de movimientos que golpean múltiples objetivos
//! - Quick Guard: Protege al equipo de movimientos con prioridad aumentada
//! - Mat Block: Protege al equipo de movimientos dañinos el primer turno
//! - Crafty Shield: Protege al equipo de movimientos de estado

pub mod processor;

#[cfg(test)]
mod tests;

pub use processor::*;
