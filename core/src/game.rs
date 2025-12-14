use serde::{Deserialize, Serialize};
use crate::models::{PokemonInstance, WeatherState, TerrainState, BattleFormat, FieldPosition};

/// Acción pendiente de un Pokémon del jugador
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PendingPlayerAction {
    /// Índice del Pokémon del jugador que ejecuta la acción
    pub user_index: usize,
    /// ID del movimiento seleccionado
    pub move_id: String,
    /// Posición del objetivo (si aplica)
    pub target_position: Option<FieldPosition>,
}

/// Estado actual del juego (pantalla/contexto en el que está el usuario)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum GameState {
    /// El usuario debe elegir 1 de 3 starters
    StarterSelection,
    /// El usuario está viendo las rutas del mapa
    Map,
    /// El usuario tiene 5 opciones ocultas frente a él (encuentro)
    EncounterSelection,
    /// El usuario está gestionando su equipo
    TeamManagement,
    /// El usuario está en una batalla
    Battle,
    /// El usuario está en una batalla contra un líder de gimnasio
    GymBattle,
    /// El usuario debe elegir un objeto de recompensa tras vencer a un Gym Leader
    LootSelection,
    /// El usuario ha completado todos los encuentros (esperando PvP)
    Completed,
}

/// Configuración de la partida
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameConfig {
    /// Cada cuántos encuentros aparece un líder de gimnasio
    pub gym_interval: u32,
    /// Total de encuentros necesarios para completar la partida
    pub total_encounters: u32,
    /// Modo Chaos: Los movimientos se seleccionan aleatoriamente del pool global
    #[serde(default)]
    pub chaos_move_randomizer: bool,
    /// Formato de batalla preferido (Single o Double)
    /// Los encuentros salvajes siempre son Single, independientemente de este valor
    #[serde(default)]
    pub preferred_format: BattleFormat,
}

/// Equipo del jugador
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerTeam {
    /// Miembros activos del equipo (máximo 6)
    pub active_members: Vec<PokemonInstance>,
    /// Miembros en la caja (reserva ilimitada)
    pub box_members: Vec<PokemonInstance>,
}

impl PlayerTeam {
    /// Crea un nuevo equipo vacío
    pub fn new() -> Self {
        Self {
            active_members: Vec::new(),
            box_members: Vec::new(),
        }
    }

    /// Verifica si el equipo activo está completo (6 miembros)
    pub fn is_active_team_full(&self) -> bool {
        self.active_members.len() >= 6
    }

    /// Añade un miembro al equipo
    /// Si el equipo activo tiene espacio (< 6), lo añade ahí.
    /// Si el equipo está lleno, lo añade a la caja (box_members).
    pub fn add_member(&mut self, pokemon: PokemonInstance) {
        if self.active_members.len() < 6 {
            self.active_members.push(pokemon);
        } else {
            self.box_members.push(pokemon);
        }
    }

    /// Cura completamente a todos los miembros activos del equipo
    /// Restaura toda la vida y elimina condiciones de estado
    pub fn heal_all(&mut self) {
        for member in &mut self.active_members {
            member.full_restore();
        }
    }
}

impl Default for PlayerTeam {
    fn default() -> Self {
        Self::new()
    }
}

/// Sesión de juego en curso
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GameSession {
    /// Identificador único de la sesión (UUID)
    pub id: String,
    /// Estado actual del juego
    pub state: GameState,
    /// Equipo del jugador
    pub team: PlayerTeam,
    /// Opciones de starters generadas temporalmente (3 opciones)
    /// Se elimina después de que el usuario elija uno
    pub starter_choices: Option<Vec<PokemonInstance>>,
    /// Opciones de encuentro generadas temporalmente (5 opciones)
    /// Se elimina después de que el usuario elija uno
    pub encounter_choices: Option<Vec<PokemonInstance>>,
    /// Opciones de objetos de recompensa tras vencer a un Gym Leader (3 opciones)
    /// Se elimina después de que el usuario elija uno
    #[serde(default)]
    pub loot_options: Option<Vec<String>>,
    /// Estado de batalla activa (si existe)
    pub battle: Option<BattleState>,
    /// Configuración de la partida
    pub config: GameConfig,
    /// Número de encuentros ganados
    pub encounters_won: u32,
}

impl GameSession {
    /// Crea una nueva sesión de juego con configuración por defecto
    pub fn new(id: String) -> Self {
        Self::with_config(id, GameConfig::default())
    }

    /// Crea una nueva sesión de juego con configuración personalizada
    pub fn with_config(id: String, config: GameConfig) -> Self {
        Self {
            id,
            state: GameState::StarterSelection,
            team: PlayerTeam::new(),
            starter_choices: None,
            encounter_choices: None,
            loot_options: None,
            battle: None,
            config,
            encounters_won: 0,
        }
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            gym_interval: 5,
            total_encounters: 20,
            chaos_move_randomizer: false,
            preferred_format: BattleFormat::Single,
        }
    }
}

/// Respuesta para el endpoint de exploración
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ExploreResponse {
    /// Número de opciones disponibles (solo para encuentros salvajes)
    pub options_count: Option<u8>,
    /// Indica si se inició una batalla de líder de gimnasio
    pub is_gym_battle: bool,
    /// Nombre del líder de gimnasio (si aplica)
    pub gym_leader_name: Option<String>,
}

/// Estado de una batalla activa
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BattleState {
    /// Formato de batalla (Single o Double)
    #[serde(default)]
    pub format: BattleFormat,
    /// Índices en el equipo del jugador que están actualmente en el campo
    /// Size 1 para Single, size 2 para Double
    pub player_active_indices: Vec<usize>,
    /// Instancia del Pokémon enemigo (salvaje) - usado solo si is_trainer_battle = false
    pub opponent_instance: PokemonInstance,
    /// Indica si es una batalla contra un entrenador (líder de gimnasio)
    pub is_trainer_battle: bool,
    /// Equipo del oponente (usado solo si is_trainer_battle = true)
    pub opponent_team: Vec<PokemonInstance>,
    /// Índices en el equipo del oponente que están actualmente en el campo
    /// Size 1 para Single, size 2 para Double
    pub opponent_active_indices: Vec<usize>,
    /// Nombre del entrenador oponente (si es batalla de entrenador)
    pub opponent_name: Option<String>,
    /// Contador de turnos (inicia en 1)
    pub turn_counter: u16,
    /// Historial de acciones en la batalla
    /// Ejemplos: "Pikachu usó Rayo", "El enemigo usó Placaje"
    pub log: Vec<String>,
    /// Estado del clima en batalla (None al comenzar)
    #[serde(default)]
    pub weather: Option<WeatherState>,
    /// Estado del terreno en batalla (None al comenzar)
    #[serde(default)]
    pub terrain: Option<TerrainState>,
    /// Acciones pendientes del jugador para este turno (en dobles, se necesitan 2)
    #[serde(default)]
    pub pending_player_actions: Vec<PendingPlayerAction>,
}

impl BattleState {
    /// Crea un nuevo estado de batalla contra un Pokémon salvaje (formato Single)
    pub fn new(player_active_index: usize, opponent_instance: PokemonInstance) -> Self {
        Self {
            format: BattleFormat::Single,
            player_active_indices: vec![player_active_index],
            opponent_instance,
            is_trainer_battle: false,
            opponent_team: Vec::new(),
            opponent_active_indices: vec![0],
            opponent_name: None,
            turn_counter: 1,
            log: Vec::new(),
            weather: None,
            terrain: None,
            pending_player_actions: Vec::new(),
        }
    }

    /// Crea un nuevo estado de batalla contra un entrenador
    /// 
    /// # Parámetros
    /// - `player_active_index`: Índice del primer Pokémon del jugador
    /// - `opponent_team`: Equipo del oponente
    /// - `opponent_name`: Nombre del oponente
    /// - `format`: Formato de batalla (Single o Double)
    /// 
    /// # Nota
    /// Si el formato es Double, se espera que el jugador tenga al menos 2 Pokémon.
    /// Si solo tiene 1, se usará Single automáticamente.
    pub fn new_trainer_battle(
        player_active_index: usize,
        opponent_team: Vec<PokemonInstance>,
        opponent_name: String,
        format: BattleFormat,
    ) -> Self {
        // Determinar los índices activos según el formato
        let (player_indices, opponent_indices) = match format {
            BattleFormat::Single => {
                (vec![player_active_index], vec![0])
            }
            BattleFormat::Double => {
                // En dobles, usar el primer y segundo Pokémon (si existen)
                let player_indices = if player_active_index == 0 {
                    vec![0, 1] // Asumir que el segundo está en índice 1
                } else {
                    vec![player_active_index, player_active_index + 1] // O el siguiente
                };
                let opponent_indices = if opponent_team.len() >= 2 {
                    vec![0, 1]
                } else {
                    vec![0] // Si el oponente solo tiene 1, usar Single
                };
                (player_indices, opponent_indices)
            }
        };
        
        // Si el formato es Double pero no hay suficientes Pokémon, forzar Single
        let final_format = if format == BattleFormat::Double && (player_indices.len() < 2 || opponent_indices.len() < 2) {
            BattleFormat::Single
        } else {
            format
        };
        
        // Ajustar índices si se forzó Single
        let (final_player_indices, final_opponent_indices) = if final_format == BattleFormat::Single {
            (vec![player_active_index], vec![0])
        } else {
            (player_indices, opponent_indices)
        };
        
        Self {
            format: final_format,
            player_active_indices: final_player_indices,
            opponent_instance: opponent_team[0].clone(), // Para compatibilidad, usar el primero
            is_trainer_battle: true,
            opponent_team,
            opponent_active_indices: final_opponent_indices,
            opponent_name: Some(opponent_name),
            turn_counter: 1,
            log: Vec::new(),
            weather: None,
            terrain: None,
            pending_player_actions: Vec::new(),
        }
    }

    /// Obtiene el Pokémon activo del oponente (primero en la lista de activos)
    /// Para compatibilidad con código existente que espera un solo Pokémon
    pub fn get_opponent_active(&self) -> &PokemonInstance {
        if self.is_trainer_battle {
            let index = self.opponent_active_indices.first().copied().unwrap_or(0);
            &self.opponent_team[index]
        } else {
            &self.opponent_instance
        }
    }

    /// Obtiene una referencia mutable al Pokémon activo del oponente (primero en la lista de activos)
    /// Para compatibilidad con código existente que espera un solo Pokémon
    pub fn get_opponent_active_mut(&mut self) -> &mut PokemonInstance {
        if self.is_trainer_battle {
            let index = self.opponent_active_indices.first().copied().unwrap_or(0);
            &mut self.opponent_team[index]
        } else {
            &mut self.opponent_instance
        }
    }

    /// Actualiza el opponent_instance para que coincida con el Pokémon activo actual
    /// Útil para mantener sincronización con el frontend
    pub fn sync_opponent_instance(&mut self) {
        if self.is_trainer_battle && !self.opponent_team.is_empty() {
            let index = self.opponent_active_indices.first().copied().unwrap_or(0);
            self.opponent_instance = self.opponent_team[index].clone();
        }
    }

    /// Verifica si el oponente tiene más Pokémon disponibles
    pub fn has_more_opponents(&self) -> bool {
        if !self.is_trainer_battle {
            return false;
        }
        // Buscar si hay algún Pokémon en el equipo que no esté debilitado
        self.opponent_team.iter().any(|p| p.current_hp > 0)
    }

    /// Cambia al siguiente Pokémon disponible del oponente
    /// Retorna true si encontró un Pokémon disponible, false si no hay más
    /// IMPORTANTE: Solo cambia el índice si encuentra un Pokémon disponible
    pub fn switch_to_next_opponent(&mut self) -> bool {
        if !self.is_trainer_battle {
            return false;
        }

        // Verificar primero si hay algún Pokémon disponible antes de cambiar
        let has_available = self.opponent_team.iter().any(|p| p.current_hp > 0);
        if !has_available {
            return false;
        }

        // Buscar el siguiente Pokémon con HP > 0 (excluyendo los actuales)
        let current_index = self.opponent_active_indices.first().copied().unwrap_or(0);
        for i in (current_index + 1)..self.opponent_team.len() {
            if self.opponent_team[i].current_hp > 0 {
                // Actualizar el primer slot (en Single solo hay uno)
                if let Some(first) = self.opponent_active_indices.first_mut() {
                    *first = i;
                } else {
                    self.opponent_active_indices.push(i);
                }
                self.sync_opponent_instance();
                return true;
            }
        }

        // Si no hay más adelante, buscar desde el principio (excluyendo el actual)
        for i in 0..current_index {
            if self.opponent_team[i].current_hp > 0 {
                // Actualizar el primer slot (en Single solo hay uno)
                if let Some(first) = self.opponent_active_indices.first_mut() {
                    *first = i;
                } else {
                    self.opponent_active_indices.push(i);
                }
                self.sync_opponent_instance();
                return true;
            }
        }

        // Si llegamos aquí, significa que el Pokémon actual es el único con HP > 0
        // pero como se está llamando después de debilitar al actual, no debería pasar
        false
    }

    /// Añade un mensaje al log de la batalla
    pub fn add_log(&mut self, message: String) {
        self.log.push(message);
    }
}

