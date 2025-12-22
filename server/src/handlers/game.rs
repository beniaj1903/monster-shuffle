use axum::{extract::{Path, State}, http::StatusCode, response::Json};
use core::factory::create_pokemon_instance;
use core::game::{BattleState, ExploreResponse, GameConfig, GameSession, GameState};
use core::models::PokemonInstance;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

use core::models::BattleFormat;

/// Payload para crear una nueva sesión de juego
#[derive(Deserialize, Debug)]
pub struct NewGamePayload {
    /// Generaciones a incluir (vacío = todas)
    pub generations: Vec<u8>,
    /// Intervalo de encuentros para líderes de gimnasio (default: 5)
    #[serde(default = "default_gym_interval")]
    pub gym_interval: Option<u32>,
    /// Total de encuentros para completar la partida (default: 20)
    #[serde(default = "default_total_encounters")]
    pub total_encounters: Option<u32>,
    /// Modo Chaos Move Randomizer (default: false)
    #[serde(default)]
    pub chaos_move_randomizer: Option<bool>,
    /// Formato de batalla preferido para batallas de gimnasio (default: Single)
    /// Nota: Los encuentros salvajes siempre son Single, independientemente de este valor
    #[serde(default)]
    pub preferred_format: Option<BattleFormat>,
}

fn default_gym_interval() -> Option<u32> {
    Some(5)
}

fn default_total_encounters() -> Option<u32> {
    Some(20)
}

/// Respuesta del endpoint de nueva sesión
#[derive(Serialize, Debug)]
pub struct NewGameResponse {
    pub session_id: String,
    pub starters: Vec<PokemonInstance>,
}

/// Handler para crear una nueva sesión de juego
/// 
/// POST /api/game/new
/// 
/// Crea una nueva sesión de juego con 3 opciones de starters basadas en las generaciones especificadas.
pub async fn start_new_game(
    State(state): State<AppState>,
    Json(payload): Json<NewGamePayload>,
) -> Result<Json<NewGameResponse>, StatusCode> {
    // Generar UUID para la nueva sesión
    let session_id = Uuid::new_v4().to_string();

    // Filtrar especies que sean candidatos a starter y estén en las generaciones especificadas
    let starter_candidates: Vec<_> = if payload.generations.is_empty() {
        // Si no se especifican generaciones, usar todas las generaciones
        state
            .pokedex
            .values()
            .filter(|species| species.is_starter_candidate)
            .collect()
    } else {
        // Filtrar por generaciones especificadas Y que sean candidatos a starter
        state
            .pokedex
            .values()
            .filter(|species| {
                species.is_starter_candidate
                    && payload.generations.contains(&species.generation)
            })
            .collect()
    };

    if starter_candidates.len() < 3 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Seleccionar 3 especies distintas al azar
    let mut rng = rand::thread_rng();
    let selected_species: Vec<_> = starter_candidates
        .choose_multiple(&mut rng, 3)
        .cloned()
        .collect();

    // Obtener el pool global de movimientos para modo Chaos
    let global_move_pool: Vec<String> = state.moves.keys().cloned().collect();
    let chaos_mode = payload.chaos_move_randomizer.unwrap_or(false);
    
    // Crear 3 instancias de Pokémon (Nivel 5)
    let mut starters = Vec::new();
    for species in selected_species {
        // Generar una seed única para cada Pokémon
        let seed = rng.gen::<u64>();
        
        // Crear la instancia con nivel 5
        let instance = create_pokemon_instance(&species, 5, seed, chaos_mode, &global_move_pool, Some(&state.moves));
        starters.push(instance);
    }

    // Crear la configuración de la partida
    let config = GameConfig {
        gym_interval: payload.gym_interval.unwrap_or(5),
        total_encounters: payload.total_encounters.unwrap_or(20),
        chaos_move_randomizer: payload.chaos_move_randomizer.unwrap_or(false),
        preferred_format: payload.preferred_format.unwrap_or(BattleFormat::Single),
    };

    // Crear la sesión de juego
    let mut session = GameSession::with_config(session_id.clone(), config);
    session.state = GameState::StarterSelection;
    session.starter_choices = Some(starters.clone());

    // Insertar la sesión en el estado
    state.sessions.insert(session_id.clone(), session);

    Ok(Json(NewGameResponse {
        session_id,
        starters,
    }))
}

/// Payload para elegir un starter
#[derive(Deserialize, Debug)]
pub struct ChooseStarterRequest {
    pub session_id: String,
    pub starter_index: usize, // 0, 1, o 2
}

/// Handler para confirmar la elección de starter
/// 
/// POST /api/game/choose-starter
/// 
/// Permite al usuario elegir uno de los 3 starters disponibles y avanzar al estado Map.
pub async fn choose_starter(
    State(state): State<AppState>,
    Json(payload): Json<ChooseStarterRequest>,
) -> Result<Json<GameSession>, StatusCode> {
    // Buscar la sesión en el estado
    let mut session = state
        .sessions
        .get_mut(&payload.session_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Verificar que el estado de la sesión sea StarterSelection
    if session.state != GameState::StarterSelection {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Verificar que starter_choices exista y que el índice sea válido
    let starter_choices = session
        .starter_choices
        .as_ref()
        .ok_or(StatusCode::BAD_REQUEST)?;

    if payload.starter_index >= starter_choices.len() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Extraer el Pokémon seleccionado
    let selected_pokemon = starter_choices[payload.starter_index].clone();

    // Añadir el Pokémon al equipo
    session.team.add_member(selected_pokemon);

    // Limpiar las opciones de starter
    session.starter_choices = None;

    // Cambiar el estado a Map
    session.state = GameState::Map;

    // Clonar la sesión actualizada para retornarla
    let updated_session = session.clone();

    // El DashMap se actualiza automáticamente cuando se suelta el guard
    // No necesitamos hacer insert explícito porque ya tenemos una referencia mutable

    Ok(Json(updated_session))
}

/// Payload para explorar una ruta
#[derive(Deserialize, Debug)]
pub struct ExploreRequest {
    pub session_id: String,
}

/// Handler para explorar una ruta y generar encuentros
/// 
/// POST /api/game/explore
/// 
/// Genera 5 opciones de Pokémon aleatorios o inicia una batalla contra un líder de gimnasio
/// según el progreso del jugador.
pub async fn explore(
    State(state): State<AppState>,
    Json(payload): Json<ExploreRequest>,
) -> Result<Json<ExploreResponse>, StatusCode> {
    // Buscar la sesión en el estado
    let mut session = state
        .sessions
        .get_mut(&payload.session_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Verificar que el estado de la sesión sea Map
    if session.state != GameState::Map {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Verificar si es momento de un líder de gimnasio
    // Si (encounters_won + 1) es múltiplo de gym_interval, genera un líder
    // Usar saturating_add para evitar overflow
    let next_encounter = session.encounters_won.saturating_add(1);
    if next_encounter % session.config.gym_interval == 0 {
        // Generar un líder de gimnasio
        use core::game::BattleState;
        
        let gym_leader_names = vec![
            "Líder Brock",
            "Líder Misty",
            "Líder Lt. Surge",
            "Líder Erika",
            "Líder Koga",
            "Líder Sabrina",
            "Líder Blaine",
            "Líder Giovanni",
        ];
        
        let mut rng = rand::thread_rng();
        let leader_name = gym_leader_names
            .choose(&mut rng)
            .unwrap_or(&"Líder de Gimnasio")
            .to_string();

        // Calcular el número de gimnasio actual
        let current_gym = (session.encounters_won / session.config.gym_interval) + 1;

        // Calcular el tamaño del equipo según el número de gym
        // Fórmula: min(6, 1 + current_gym)
        // Gym 1 = 2 pokes, Gym 2 = 3 pokes... Gym 5+ = 6 pokes
        let team_size = (1 + current_gym).min(6);

        // Calcular el nivel promedio del equipo del jugador
        let avg_level = if session.team.active_members.is_empty() {
            5
        } else {
            // Usar u16 para evitar overflow al sumar niveles
            let sum: u16 = session.team.active_members.iter().map(|p| p.level as u16).sum();
            (sum as f32 / session.team.active_members.len() as f32).ceil() as u8
        };
        
        // Obtener el pool global de movimientos para modo Chaos
        let global_move_pool: Vec<String> = state.moves.keys().cloned().collect();
        let chaos_mode = session.config.chaos_move_randomizer;
        
        let available_species: Vec<_> = state.pokedex.values().collect();
        
        if available_species.is_empty() {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        let mut opponent_team = Vec::new();
        for i in 0..team_size {
            let species = available_species
                .choose(&mut rng)
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
            
            // Nivel ligeramente superior al promedio del jugador (1-3 niveles más)
            let level_bonus = rng.gen_range(1..=3);
            let opponent_level = (avg_level + level_bonus).min(100);
            
            let seed = rng.gen::<u64>() + i as u64;
            let instance = create_pokemon_instance(species, opponent_level, seed, chaos_mode, &global_move_pool, Some(&state.moves));
            opponent_team.push(instance);
        }

        // Asignar items aleatorios a los Gym Leaders (50% de probabilidad por Pokémon)
        // Lista de items competitivos disponibles
        let available_items = vec![
            "leftovers",
            "life-orb",
            "choice-band",
            "choice-specs",
            "choice-scarf",
            "sitrus-berry",
            "rocky-helmet",
            "focus-sash",
        ];
        
        for opponent in &mut opponent_team {
            // 50% de probabilidad de tener un item
            if rng.gen_bool(0.5) {
                if let Some(item_id) = available_items.choose(&mut rng) {
                    opponent.held_item = Some(item_id.to_string());
                }
            }
            
            // Inicializar battle_stages para los oponentes al entrar en batalla
            if opponent.battle_stages.is_none() {
                opponent.init_battle_stages();
            }
        }

        // Crear el estado de batalla contra el líder
        // Usar el formato preferido de la configuración de la sesión
        let preferred_format = session.config.preferred_format;
        eprintln!("[DEBUG] explore: Creando batalla de gimnasio con formato: {:?}", preferred_format);
        let battle_state = BattleState::new(
            0, // El jugador usa su primer Pokémon
            opponent_team,
            leader_name.clone(),
            preferred_format,   
            true,
        );
        eprintln!("[DEBUG] explore: BattleState creado - format: {:?}, player_active_indices: {:?}, opponent_active_indices: {:?}", 
            battle_state.format, battle_state.player_active_indices, battle_state.opponent_active_indices);

        // Inicializar battle_stages para los Pokémon del jugador al entrar en batalla
        if let Some(player_pokemon) = session.team.active_members.get_mut(0) {
            if player_pokemon.battle_stages.is_none() {
                player_pokemon.init_battle_stages();
            }
        }

        // Inicializar la batalla
        session.battle = Some(battle_state);
        session.state = GameState::GymBattle;

        // Retornar respuesta indicando que es una batalla de gimnasio
        Ok(Json(ExploreResponse {
            options_count: None,
            is_gym_battle: true,
            gym_leader_name: Some(leader_name),
        }))
    } else {
        // Encuentro normal: generar 5 opciones salvajes con dificultad progresiva
        
        // Calcular progreso (0.0 a 1.0)
        let progress = (session.encounters_won as f32 / session.config.total_encounters as f32).min(1.0).max(0.0);
        
        // Definir rango BST basado en el progreso
        let min_bst = (180.0 + (320.0 * progress)) as u32;
        let max_bst = (300.0 + (350.0 * progress)) as u32;
        
        // Calcular nivel salvaje (crece con el jugador)
        // El nivel crece al mismo ritmo que el jugador (10 / gym_interval por encuentro)
        // Usar saturating_mul para evitar overflow en la multiplicación
        let level_increase = (session.encounters_won.saturating_mul(10) / session.config.gym_interval.max(1)) as u8;
        let wild_level = 5u8.saturating_add(level_increase).min(100);
        
        // Filtrar especies por BST
        let mut rng = rand::thread_rng();
        let filtered_species: Vec<_> = state
            .pokedex
            .values()
            .filter(|species| {
                let bst = species.bst();
                bst >= min_bst && bst <= max_bst
            })
            .collect();
        
        // Fallback: si el filtro es muy restrictivo, expandir el rango
        let candidate_species: Vec<_> = if filtered_species.is_empty() {
            // Expandir el rango BST
            let expanded_min = min_bst.saturating_sub(50);
            let expanded_max = max_bst.saturating_add(50);
            state
                .pokedex
                .values()
                .filter(|species| {
                    let bst = species.bst();
                    bst >= expanded_min && bst <= expanded_max
                })
                .collect()
        } else {
            filtered_species
        };
        
        // Fallback final: si aún no hay candidatos, usar todas las especies
        let final_candidates: Vec<_> = if candidate_species.is_empty() {
            state.pokedex.values().collect()
        } else {
            candidate_species
        };
        
        if final_candidates.is_empty() {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        
        // Seleccionar 5 especies aleatorias del pool filtrado
        let selected_species: Vec<_> = final_candidates
            .choose_multiple(&mut rng, 5.min(final_candidates.len()))
            .cloned()
            .collect();
        
        // Obtener el pool global de movimientos para modo Chaos
        let global_move_pool: Vec<String> = state.moves.keys().cloned().collect();
        let chaos_mode = session.config.chaos_move_randomizer;
        
        // Crear 5 instancias de Pokémon con el nivel calculado
        let mut encounters = Vec::new();
        for species in selected_species {
            // Generar una seed única para cada Pokémon
            let seed = rng.gen::<u64>();
            
            // Crear la instancia con el nivel calculado
            let instance = create_pokemon_instance(&species, wild_level, seed, chaos_mode, &global_move_pool, Some(&state.moves));
            encounters.push(instance);
        }
        
        // Guardar las opciones de encuentro en la sesión
        session.encounter_choices = Some(encounters);
        
        // Cambiar el estado a EncounterSelection
        session.state = GameState::EncounterSelection;
        
        // Retornar solo el número de opciones (sin revelar los Pokémon)
        Ok(Json(ExploreResponse {
            options_count: Some(5),
            is_gym_battle: false,
            gym_leader_name: None,
        }))
    }
}

/// Payload para seleccionar un encuentro
#[derive(Deserialize, Debug)]
pub struct SelectEncounterRequest {
    pub session_id: String,
    pub selection_index: usize, // 0, 1, 2, 3, o 4
}

/// Respuesta del endpoint de selección de encuentro
/// Ahora retorna la sesión completa para sincronización inmediata
pub type SelectEncounterResponse = GameSession;

/// Handler para seleccionar un encuentro
/// 
/// POST /api/game/select-encounter
/// 
/// Permite al usuario elegir uno de los 5 encuentros disponibles.
/// Inicia una batalla contra el Pokémon seleccionado (no lo añade al equipo).
/// Retorna el Pokémon seleccionado (aquí se revela la sorpresa).
pub async fn select_encounter(
    State(state): State<AppState>,
    Json(payload): Json<SelectEncounterRequest>,
) -> Result<Json<SelectEncounterResponse>, StatusCode> {
    // Buscar la sesión en el estado
    let mut session = state
        .sessions
        .get_mut(&payload.session_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Verificar que el estado de la sesión sea EncounterSelection
    if session.state != GameState::EncounterSelection {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Verificar que el jugador tenga al menos un Pokémon en su equipo
    if session.team.active_members.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Verificar que encounter_choices exista y que el índice sea válido
    let encounter_choices = session
        .encounter_choices
        .as_ref()
        .ok_or(StatusCode::BAD_REQUEST)?;

    if payload.selection_index >= encounter_choices.len() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Extraer el Pokémon seleccionado
    let mut selected_pokemon = encounter_choices[payload.selection_index].clone();

    // Inicializar battle_stages para el oponente al entrar en batalla
    if selected_pokemon.battle_stages.is_none() {
        selected_pokemon.init_battle_stages();
    }

    let opponent_team = vec![selected_pokemon.clone()];

    let mut battle_state = BattleState::new(
        0, // El jugador empieza con su primer slot
        opponent_team,
        String::new(), // Batallas salvajes no tienen nombre de entrenador
        BattleFormat::Single, // Salvajes SIEMPRE son Singles
        false // is_trainer_battle = false
    );

    // Inicializar battle_stages para los Pokémon del jugador al entrar en batalla
    if let Some(player_pokemon) = session.team.active_members.get_mut(0) {
        if player_pokemon.battle_stages.is_none() {
            player_pokemon.init_battle_stages();
        }
    }

    // Añadir mensaje inicial al log
    battle_state.add_log(format!(
        "¡Un {} salvaje apareció!",
        selected_pokemon.species.display_name
    ));

    // Inicializar el combate
    session.battle = Some(battle_state);
    session.state = GameState::Battle;
    
    // Limpiar las opciones de encuentro
    session.encounter_choices = None;

    // Retornar la sesión completa para sincronización inmediata del frontend
    Ok(Json(session.clone()))
}

/// Respuesta de error para cuando no se encuentra una sesión
#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    pub error: String,
}

/// Handler para obtener el estado actual de una partida
/// 
/// GET /api/game/:session_id
/// 
/// Retorna la sesión de juego completa, incluyendo el equipo y el estado actual.
pub async fn get_game_state(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<GameSession>, (StatusCode, Json<ErrorResponse>)> {
    // Buscar la sesión en el estado
    let session = state
        .sessions
        .get(&session_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Session '{}' not found", session_id),
                }),
            )
        })?;

    // Clonar la sesión para retornarla (GameSession implementa Clone)
    let session_clone = session.clone();

    Ok(Json(session_clone))
}

/// Payload para seleccionar un objeto de recompensa
#[derive(Deserialize, Debug)]
pub struct SelectLootRequest {
    pub session_id: String,
    /// Índice del objeto seleccionado (0-2)
    pub item_index: usize,
    /// Índice del Pokémon del equipo que recibirá el objeto
    pub target_pokemon_index: usize,
}

/// Handler para seleccionar un objeto de recompensa tras vencer a un Gym Leader
/// 
/// POST /api/game/select-loot
/// 
/// Asigna el objeto seleccionado al Pokémon indicado y vuelve al mapa.
pub async fn select_loot(
    State(state): State<AppState>,
    Json(payload): Json<SelectLootRequest>,
) -> Result<Json<GameSession>, StatusCode> {
    let mut session = state
        .sessions
        .get(&payload.session_id)
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();

    // Validar que el estado sea LootSelection
    if session.state != GameState::LootSelection {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validar que existan opciones de loot
    let loot_options = session.loot_options.as_ref()
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Validar el índice del objeto
    if payload.item_index >= loot_options.len() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validar el índice del Pokémon
    if payload.target_pokemon_index >= session.team.active_members.len() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Obtener el objeto seleccionado
    let selected_item = loot_options[payload.item_index].clone();

    // Asignar el objeto al Pokémon
    session.team.active_members[payload.target_pokemon_index].held_item = Some(selected_item);

    // Limpiar las opciones de loot
    session.loot_options = None;

    // Volver al mapa
    session.state = GameState::Map;

    // Guardar la sesión actualizada
    state.sessions.insert(payload.session_id.clone(), session.clone());

    Ok(Json(session))
}

