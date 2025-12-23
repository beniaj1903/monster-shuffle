//! Procesador de protecciones avanzadas
//!
//! Funciones para verificar y aplicar protecciones de equipo

use crate::models::{MoveData, PokemonInstance};

/// Verifica si un movimiento es bloqueado por Wide Guard
///
/// Wide Guard protege al equipo de movimientos que golpean múltiples objetivos:
/// - all-opponents (ej: Earthquake, Rock Slide, Surf)
/// - all-other-pokemon (ej: Explosion)
/// - all-pokemon (ej: algunos movimientos especiales)
pub fn is_blocked_by_wide_guard(
    defender: &PokemonInstance,
    move_data: &MoveData,
) -> bool {
    // Verificar si Wide Guard está activo
    let Some(ref volatile) = defender.volatile_status else {
        return false;
    };

    if !volatile.wide_guard_active {
        return false;
    }

    // Wide Guard solo protege de movimientos que golpean múltiples objetivos
    is_spread_move(move_data)
}

/// Verifica si un movimiento es bloqueado por Quick Guard
///
/// Quick Guard protege al equipo de movimientos con prioridad aumentada (priority > 0)
pub fn is_blocked_by_quick_guard(
    defender: &PokemonInstance,
    move_data: &MoveData,
) -> bool {
    // Verificar si Quick Guard está activo
    let Some(ref volatile) = defender.volatile_status else {
        return false;
    };

    if !volatile.quick_guard_active {
        return false;
    }

    // Quick Guard solo protege de movimientos con prioridad > 0
    move_data.priority > 0
}

/// Verifica si un movimiento es bloqueado por Mat Block
///
/// Mat Block protege al equipo de movimientos dañinos, pero solo funciona
/// el primer turno (cuando el usuario acaba de salir al campo)
pub fn is_blocked_by_mat_block(
    defender: &PokemonInstance,
    move_data: &MoveData,
) -> bool {
    // Verificar si Mat Block está activo
    let Some(ref volatile) = defender.volatile_status else {
        return false;
    };

    if !volatile.mat_block_active {
        return false;
    }

    // Mat Block solo protege de movimientos dañinos (que tienen poder)
    move_data.power.is_some()
}

/// Verifica si un movimiento es bloqueado por Crafty Shield
///
/// Crafty Shield protege al equipo de movimientos de estado (no dañinos)
pub fn is_blocked_by_crafty_shield(
    defender: &PokemonInstance,
    move_data: &MoveData,
) -> bool {
    // Verificar si Crafty Shield está activo
    let Some(ref volatile) = defender.volatile_status else {
        return false;
    };

    if !volatile.crafty_shield_active {
        return false;
    }

    // Crafty Shield solo protege de movimientos sin poder (status moves)
    move_data.power.is_none() && move_data.damage_class == "status"
}

/// Verifica si un movimiento es de tipo "spread" (golpea múltiples objetivos)
fn is_spread_move(move_data: &MoveData) -> bool {
    matches!(
        move_data.target.as_str(),
        "all-opponents"       // Earthquake, Rock Slide, Surf, etc.
        | "all-other-pokemon" // Explosion, Self-Destruct
        | "all-pokemon"       // Algunos movimientos especiales
        | "entire-field"      // Campo entero (raro)
    )
}

/// Verifica todas las protecciones y retorna el tipo de protección que bloquea el movimiento
///
/// Retorna Some(mensaje) si el movimiento es bloqueado, None si no
pub fn check_advanced_protections(
    defender: &PokemonInstance,
    move_data: &MoveData,
) -> Option<String> {
    // Verificar en orden de prioridad
    if is_blocked_by_wide_guard(defender, move_data) {
        return Some(format!(
            "¡Wide Guard protegió a {}!",
            defender.species.display_name
        ));
    }

    if is_blocked_by_quick_guard(defender, move_data) {
        return Some(format!(
            "¡Quick Guard protegió a {}!",
            defender.species.display_name
        ));
    }

    if is_blocked_by_mat_block(defender, move_data) {
        return Some(format!(
            "¡Mat Block protegió a {}!",
            defender.species.display_name
        ));
    }

    if is_blocked_by_crafty_shield(defender, move_data) {
        return Some(format!(
            "¡Crafty Shield protegió a {}!",
            defender.species.display_name
        ));
    }

    None
}

/// Activa Wide Guard en el Pokémon
pub fn activate_wide_guard(pokemon: &mut PokemonInstance) {
    if pokemon.volatile_status.is_none() {
        pokemon.init_battle_stages();
    }
    if let Some(ref mut volatile) = pokemon.volatile_status {
        volatile.wide_guard_active = true;
    }
}

/// Activa Quick Guard en el Pokémon
pub fn activate_quick_guard(pokemon: &mut PokemonInstance) {
    if pokemon.volatile_status.is_none() {
        pokemon.init_battle_stages();
    }
    if let Some(ref mut volatile) = pokemon.volatile_status {
        volatile.quick_guard_active = true;
    }
}

/// Activa Mat Block en el Pokémon
pub fn activate_mat_block(pokemon: &mut PokemonInstance) {
    if pokemon.volatile_status.is_none() {
        pokemon.init_battle_stages();
    }
    if let Some(ref mut volatile) = pokemon.volatile_status {
        volatile.mat_block_active = true;
    }
}

/// Activa Crafty Shield en el Pokémon
pub fn activate_crafty_shield(pokemon: &mut PokemonInstance) {
    if pokemon.volatile_status.is_none() {
        pokemon.init_battle_stages();
    }
    if let Some(ref mut volatile) = pokemon.volatile_status {
        volatile.crafty_shield_active = true;
    }
}

/// Limpia todas las protecciones avanzadas al inicio de cada turno
pub fn clear_advanced_protections(pokemon: &mut PokemonInstance) {
    if let Some(ref mut volatile) = pokemon.volatile_status {
        volatile.wide_guard_active = false;
        volatile.quick_guard_active = false;
        volatile.mat_block_active = false;
        volatile.crafty_shield_active = false;
    }
}
