//! Helpers de acceso a Pokémon en batalla
//!
//! Este módulo proporciona funciones para acceder a Pokémon en el campo de batalla,
//! ya sea de forma inmutable o mutable, basándose en su posición.

use crate::models::{PokemonInstance, FieldPosition};
use crate::game::{BattleState, PlayerTeam};

/// Obtiene una referencia inmutable a un Pokémon basándose en su posición
///
/// # Argumentos
/// * `position` - Posición del Pokémon en el campo
/// * `team_index` - Índice del Pokémon en su equipo
/// * `battle_state` - Estado actual de la batalla
/// * `player_team` - Equipo del jugador
/// * `opponent_team` - Equipo del oponente
///
/// # Retorna
/// `Some(&PokemonInstance)` si el Pokémon existe, `None` en caso contrario
pub fn get_pokemon<'a>(
    position: FieldPosition,
    team_index: usize,
    battle_state: &'a BattleState,
    player_team: &'a PlayerTeam,
    opponent_team: &'a Vec<PokemonInstance>,
) -> Option<&'a PokemonInstance> {
    match position {
        FieldPosition::PlayerLeft | FieldPosition::PlayerRight => {
            player_team.active_members.get(team_index)
        }
        FieldPosition::OpponentLeft | FieldPosition::OpponentRight => {
            if battle_state.is_trainer_battle {
                opponent_team.get(team_index)
            } else {
                Some(&battle_state.opponent_instance)
            }
        }
    }
}

/// Obtiene una referencia mutable a un Pokémon basándose en su posición
///
/// # Argumentos
/// * `position` - Posición del Pokémon en el campo
/// * `team_index` - Índice del Pokémon en su equipo
/// * `battle_state` - Estado actual de la batalla (mutable)
/// * `player_team` - Equipo del jugador (mutable)
/// * `opponent_team` - Equipo del oponente (mutable)
///
/// # Retorna
/// `Some(&mut PokemonInstance)` si el Pokémon existe, `None` en caso contrario
pub fn get_pokemon_mut<'a>(
    position: FieldPosition,
    team_index: usize,
    battle_state: &'a mut BattleState,
    player_team: &'a mut PlayerTeam,
    opponent_team: &'a mut Vec<PokemonInstance>,
) -> Option<&'a mut PokemonInstance> {
    match position {
        FieldPosition::PlayerLeft | FieldPosition::PlayerRight => {
            player_team.active_members.get_mut(team_index)
        }
        FieldPosition::OpponentLeft | FieldPosition::OpponentRight => {
            if battle_state.is_trainer_battle {
                opponent_team.get_mut(team_index)
            } else {
                Some(&mut battle_state.opponent_instance)
            }
        }
    }
}

/// Obtiene el índice del equipo para una posición dada
///
/// # Argumentos
/// * `position` - Posición en el campo
/// * `battle_state` - Estado actual de la batalla
///
/// # Retorna
/// `Some(usize)` con el índice si existe, `None` en caso contrario
pub fn get_team_index(position: FieldPosition, battle_state: &BattleState) -> Option<usize> {
    match position {
        FieldPosition::PlayerLeft => battle_state.player_active_indices.get(0).copied(),
        FieldPosition::PlayerRight => battle_state.player_active_indices.get(1).copied(),
        FieldPosition::OpponentLeft => {
            if battle_state.is_trainer_battle {
                battle_state.opponent_active_indices.get(0).copied()
            } else {
                Some(0) // Wild battle - siempre índice 0
            }
        }
        FieldPosition::OpponentRight => battle_state.opponent_active_indices.get(1).copied(),
    }
}

/// Verifica si un Pokémon en una posición específica está vivo
///
/// # Argumentos
/// * `position` - Posición del Pokémon en el campo
/// * `team_index` - Índice del Pokémon en su equipo
/// * `battle_state` - Estado actual de la batalla
/// * `player_team` - Equipo del jugador
/// * `opponent_team` - Equipo del oponente
///
/// # Retorna
/// `true` si el Pokémon existe y tiene HP > 0, `false` en caso contrario
pub fn is_pokemon_alive(
    position: FieldPosition,
    team_index: usize,
    battle_state: &BattleState,
    player_team: &PlayerTeam,
    opponent_team: &Vec<PokemonInstance>,
) -> bool {
    get_pokemon(position, team_index, battle_state, player_team, opponent_team)
        .map(|p| p.current_hp > 0)
        .unwrap_or(false)
}
