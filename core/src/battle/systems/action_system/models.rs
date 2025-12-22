//! Modelos del sistema de acciones
//!
//! Define las estructuras de datos utilizadas para representar
//! acciones de batalla en la cola de prioridad.

use crate::models::{FieldPosition, MoveData};

/// Candidato de acción para la cola de prioridad global
///
/// Representa un Pokémon que va a ejecutar una acción en este turno.
/// Se utiliza para ordenar las acciones por prioridad y velocidad.
#[derive(Debug, Clone)]
pub struct ActionCandidate {
    /// Posición del Pokémon en el campo
    pub position: FieldPosition,

    /// Índice en el array correspondiente (player_active_indices o opponent_active_indices)
    pub team_index: usize,

    /// Si es del jugador (true) o del oponente (false)
    pub is_player: bool,

    /// Velocidad efectiva del Pokémon (considerando stages y parálisis)
    pub speed: u16,

    /// Prioridad del movimiento seleccionado
    pub priority: i8,

    /// Datos del movimiento seleccionado
    pub move_data: MoveData,

    /// ID del template del movimiento (para consumo de PP)
    pub move_template_id: String,

    /// Objetivo seleccionado (si aplica, para movimientos "selected-pokemon")
    pub selected_target: Option<FieldPosition>,

    /// Nombre del Pokémon para logs
    pub pokemon_name: String,
}

impl ActionCandidate {
    /// Crea un nuevo candidato de acción
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        position: FieldPosition,
        team_index: usize,
        is_player: bool,
        speed: u16,
        priority: i8,
        move_data: MoveData,
        move_template_id: String,
        selected_target: Option<FieldPosition>,
        pokemon_name: String,
    ) -> Self {
        Self {
            position,
            team_index,
            is_player,
            speed,
            priority,
            move_data,
            move_template_id,
            selected_target,
            pokemon_name,
        }
    }
}
