use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use core::models::{PokemonSpecies, MoveData};
use core::game::GameSession;

/// Estado global de la aplicación compartido entre todos los handlers
#[derive(Clone)]
pub struct AppState {
    /// Pokedex indexado por species_id para acceso O(1)
    pub pokedex: Arc<HashMap<String, PokemonSpecies>>,
    /// Base de datos de movimientos indexada por move_id para acceso O(1)
    pub moves: Arc<HashMap<String, MoveData>>,
    /// Sesiones de juego activas indexadas por session_id
    pub sessions: Arc<DashMap<String, GameSession>>,
}

impl AppState {
    /// Crea un nuevo AppState con un pokedex y movimientos vacíos
    pub fn new(pokedex: HashMap<String, PokemonSpecies>, moves: HashMap<String, MoveData>) -> Self {
        Self {
            pokedex: Arc::new(pokedex),
            moves: Arc::new(moves),
            sessions: Arc::new(DashMap::new()),
        }
    }

    /// Obtiene el número de Pokémon cargados
    pub fn count(&self) -> usize {
        self.pokedex.len()
    }
}

/// Carga el pokedex desde el archivo JSON
/// 
/// # Errors
/// 
/// Retorna un error si:
/// - No se puede leer el archivo `./data/pokedex.json`
/// - El contenido no es un JSON válido
/// - El JSON no puede ser deserializado a `Vec<PokemonSpecies>`
pub fn load_pokedex() -> Result<HashMap<String, PokemonSpecies>, Box<dyn std::error::Error>> {
    // Leer el archivo
    let file_path = "./data/pokedex.json";
    let contents = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read pokedex file at {}: {}", file_path, e))?;

    // Deserializar el JSON
    let species_list: Vec<PokemonSpecies> = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse pokedex JSON: {}", e))?;

    // Convertir a HashMap indexado por species_id
    let mut pokedex = HashMap::new();
    for species in species_list {
        let key = species.species_id.clone();
        pokedex.insert(key, species);
    }

    Ok(pokedex)
}

/// Carga los movimientos desde el archivo JSON
/// 
/// # Errors
/// 
/// Retorna un error si:
/// - No se puede leer el archivo `./data/moves.json`
/// - El contenido no es un JSON válido
/// - El JSON no puede ser deserializado a `Vec<MoveData>`
pub fn load_moves() -> Result<HashMap<String, MoveData>, Box<dyn std::error::Error>> {
    // Leer el archivo
    let file_path = "./data/moves.json";
    let contents = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read moves file at {}: {}", file_path, e))?;

    // Deserializar el JSON
    let moves_list: Vec<MoveData> = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse moves JSON: {}", e))?;

    // Convertir a HashMap indexado por id
    let mut moves = HashMap::new();
    for move_data in moves_list {
        let key = move_data.id.clone();
        moves.insert(key, move_data);
    }

    Ok(moves)
}

