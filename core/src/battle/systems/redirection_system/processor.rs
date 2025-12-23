//! Procesador de redirección
//!
//! Funciones para aplicar y manejar redirección de ataques

use crate::game::{BattleState, RedirectionState};
use crate::models::{FieldPosition, PokemonInstance, MoveData};

/// Aplica redirección si está activa y el movimiento es elegible
///
/// Retorna la posición del objetivo redirigido, o None si no hay redirección
pub fn apply_redirection(
    original_target: FieldPosition,
    attacker_position: FieldPosition,
    attacker: &PokemonInstance,
    move_data: &MoveData,
    battle_state: &BattleState,
) -> Option<FieldPosition> {
    // Solo aplicar redirección en batallas dobles
    if battle_state.format != crate::models::BattleFormat::Double {
        return None;
    }

    // Verificar si hay redirección activa
    let redirection = battle_state.redirection.as_ref()?;

    // Verificar si el movimiento es single-target
    // Los movimientos que afectan a todos no pueden ser redirigidos
    if !is_single_target_move(move_data) {
        return None;
    }

    // Verificar si el atacante es del equipo contrario al redirector
    let redirector_is_player = matches!(
        redirection.redirector_position,
        FieldPosition::PlayerLeft | FieldPosition::PlayerRight
    );
    let attacker_is_player = matches!(
        attacker_position,
        FieldPosition::PlayerLeft | FieldPosition::PlayerRight
    );

    // Si opponent_only es true, solo redirigir ataques del equipo contrario
    if redirection.opponent_only && redirector_is_player == attacker_is_player {
        return None;
    }

    // Rage Powder: No afecta a tipos Grass
    if redirection.redirection_type == "rage-powder" {
        if attacker.randomized_profile.rolled_primary_type == crate::models::PokemonType::Grass
            || attacker.randomized_profile.rolled_secondary_type == Some(crate::models::PokemonType::Grass)
        {
            return None;
        }
    }

    // Si el objetivo original ya es el redirector, no cambiar
    if original_target == redirection.redirector_position {
        return None;
    }

    // Redirigir al redirector
    Some(redirection.redirector_position)
}

/// Verifica si un movimiento es single-target
fn is_single_target_move(move_data: &MoveData) -> bool {
    match move_data.target.as_str() {
        "selected-pokemon" => true,         // Single target
        "selected-pokemon-me-first" => true, // Single target (Me First)
        "opponents-field" => false,          // Afecta a todos los oponentes
        "user-or-ally" => false,             // Afecta al usuario o aliado
        "users-field" => false,              // Afecta al campo del usuario
        "user-and-allies" => false,          // Afecta al usuario y aliados
        "all-other-pokemon" => false,        // Afecta a todos excepto usuario
        "all-opponents" => false,            // Afecta a todos los oponentes
        "entire-field" => false,             // Afecta a todo el campo
        "user" => false,                     // Afecta solo al usuario
        "random-opponent" => true,           // Single target (random)
        "all-pokemon" => false,              // Afecta a todos
        "all-allies" => false,               // Afecta a todos los aliados
        "specific-move" => false,            // Depende del movimiento
        _ => true,                           // Por defecto, asumir single-target
    }
}

/// Activa Follow Me en el campo
pub fn set_follow_me(
    battle_state: &mut BattleState,
    user_position: FieldPosition,
) {
    battle_state.redirection = Some(RedirectionState {
        redirector_position: user_position,
        redirection_type: "follow-me".to_string(),
        opponent_only: true, // Solo afecta ataques del oponente
    });
}

/// Activa Rage Powder en el campo
pub fn set_rage_powder(
    battle_state: &mut BattleState,
    user_position: FieldPosition,
) {
    battle_state.redirection = Some(RedirectionState {
        redirector_position: user_position,
        redirection_type: "rage-powder".to_string(),
        opponent_only: true, // Solo afecta ataques del oponente
    });
}

/// Activa Spotlight en el campo
pub fn set_spotlight(
    battle_state: &mut BattleState,
    target_position: FieldPosition,
) {
    battle_state.redirection = Some(RedirectionState {
        redirector_position: target_position,
        redirection_type: "spotlight".to_string(),
        opponent_only: false, // Afecta todos los ataques
    });
}

/// Intercambia las posiciones de dos Pokémon aliados (Ally Switch)
///
/// Retorna true si el intercambio fue exitoso
pub fn ally_switch(
    battle_state: &mut BattleState,
    user_position: FieldPosition,
) -> bool {
    // Solo funciona en batallas dobles
    if battle_state.format != crate::models::BattleFormat::Double {
        return false;
    }

    // Determinar qué equipo y cuál es la otra posición
    let (indices, _other_position) = match user_position {
        FieldPosition::PlayerLeft => {
            (&mut battle_state.player_active_indices, FieldPosition::PlayerRight)
        }
        FieldPosition::PlayerRight => {
            (&mut battle_state.player_active_indices, FieldPosition::PlayerLeft)
        }
        FieldPosition::OpponentLeft => {
            (&mut battle_state.opponent_active_indices, FieldPosition::OpponentRight)
        }
        FieldPosition::OpponentRight => {
            (&mut battle_state.opponent_active_indices, FieldPosition::OpponentLeft)
        }
    };

    // Verificar que hay 2 Pokémon activos
    if indices.len() != 2 {
        return false;
    }

    // Intercambiar los índices
    indices.swap(0, 1);

    true
}

/// Limpia el estado de redirección al final del turno
pub fn clear_redirection(battle_state: &mut BattleState) {
    battle_state.redirection = None;
}
