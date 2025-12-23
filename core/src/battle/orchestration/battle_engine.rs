//! Motor principal de batalla
//!
//! Este módulo coordina todos los sistemas de batalla en un flujo estructurado:
//! 1. Preparación del turno
//! 2. Recopilación de acciones
//! 3. Ordenamiento por prioridad/velocidad
//! 4. Ejecución de acciones
//! 5. Resolución de fin de turno
//! 6. Verificación de victorias/derrotas

use crate::game::{BattleState, PlayerTeam};
use super::super::BattleOutcome;

// Re-exportar la función principal del pipeline por ahora
// TODO: Migrar gradualmente la lógica de pipeline.rs aquí
pub use super::super::pipeline::execute_turn;

/// Determina el resultado cuando el jugador se debilita
///
/// Verifica si hay más Pokémon del jugador disponibles
pub fn determine_player_outcome(player_team: &PlayerTeam, battle_state: &BattleState) -> BattleOutcome {
    // Buscar si hay otro Pokémon del jugador disponible
    let has_more_pokemon = player_team.active_members.iter()
        .enumerate()
        .any(|(i, p)| !battle_state.player_active_indices.contains(&i) && p.current_hp > 0);

    if has_more_pokemon {
        BattleOutcome::PlayerMustSwitch
    } else {
        BattleOutcome::PlayerLost
    }
}

/// Determina el resultado cuando el enemigo se debilita
///
/// Verifica si hay más enemigos disponibles en el equipo.
/// Si hay más enemigos, cambia al siguiente y retorna EnemySwitched.
/// Si no hay más, retorna PlayerWon.
pub fn determine_enemy_outcome(battle_state: &mut BattleState, logs: &mut Vec<String>) -> BattleOutcome {
    eprintln!("[BATTLE_END] determine_enemy_outcome - Verificando si hay más oponentes...");
    // Buscar si hay otro Pokémon enemigo disponible
    if battle_state.switch_to_next_opponent() {
        eprintln!("[BATTLE_END] Se encontró otro oponente, cambiando...");
        // El enemigo cambió de Pokémon
        let next_opponent = battle_state.get_opponent_active().clone();
        let opponent_name = battle_state.opponent_name.clone().unwrap_or_else(|| "El entrenador".to_string());
        logs.push(format!(
            "{} envió a {}!",
            opponent_name,
            next_opponent.species.display_name
        ));
        BattleOutcome::EnemySwitched
    } else {
        eprintln!("[BATTLE_END] No hay más oponentes, jugador ganó!");
        // No hay más Pokémon enemigos con vida
        BattleOutcome::PlayerWon
    }
}

/// Verifica el estado de la batalla después de cada acción
///
/// Retorna BattleOutcome apropiado dependiendo del estado de los combatientes
pub fn check_battle_state(
    battle_state: &mut BattleState,
    player_team: &PlayerTeam,
    opponent_team: &Vec<crate::models::PokemonInstance>,
    logs: &mut Vec<String>,
) -> BattleOutcome {
    eprintln!("[BATTLE_END] check_battle_state - Verificando estado de batalla");

    // Verificar si todos los Pokémon activos del jugador están debilitados
    let all_player_active_fainted = battle_state.player_active_indices.iter().all(|&idx| {
        let hp = player_team.active_members.get(idx).map(|p| p.current_hp).unwrap_or(0);
        eprintln!("[BATTLE_END] Jugador índice {}: HP = {}", idx, hp);
        hp == 0
    });
    eprintln!("[BATTLE_END] Todos los jugadores debilitados: {}", all_player_active_fainted);

    // Verificar si todos los Pokémon activos del oponente están debilitados
    // IMPORTANTE: Usar el opponent_team del parámetro, NO battle_state.opponent_team
    // porque el parámetro es el que se modifica durante execute_turn
    let all_opponent_active_fainted = battle_state.opponent_active_indices.iter().all(|&idx| {
        let hp = opponent_team.get(idx).map(|p| p.current_hp).unwrap_or(0);
        eprintln!("[BATTLE_END] Oponente índice {}: HP = {}", idx, hp);
        hp == 0
    });
    eprintln!("[BATTLE_END] Todos los oponentes debilitados: {}", all_opponent_active_fainted);

    // Si todos los oponentes activos están debilitados
    if all_opponent_active_fainted {
        eprintln!("[BATTLE_END] Todos los oponentes están debilitados, determinando resultado...");
        // Intentar cambiar a un nuevo oponente
        return determine_enemy_outcome(battle_state, logs);
    }

    // Si todos los jugadores activos están debilitados
    if all_player_active_fainted {
        // Verificar si hay más Pokémon disponibles
        return determine_player_outcome(player_team, battle_state);
    }

    // La batalla continúa
    BattleOutcome::Continue
}

// NOTA: Las siguientes funciones permanecen en pipeline.rs por ahora:
// - execute_turn: Función principal que ejecuta un turno completo
// - execute_single_action: Ejecuta una acción individual de un Pokémon
// - process_move_hit: Procesa un hit de movimiento
// - collect_action_candidates: Recopila todas las acciones de ambos equipos
// - sort_candidates: Ordena candidatos por prioridad y velocidad
// - handle_entry_hazards: Maneja hazards de entrada
// - apply_on_entry_stat_change: Aplica cambios de stats en entrada
// - apply_stat_stage_change: Aplica cambios de etapas de stats
// - apply_end_of_turn_abilities: Aplica habilidades de fin de turno
// - process_end_of_turn_residuals: Procesa efectos residuales

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Agregar tests unitarios para determine_player_outcome y determine_enemy_outcome
    // Una vez que se migren más funciones de pipeline.rs aquí
}
