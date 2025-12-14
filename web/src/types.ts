// Tipos base para movimientos
export interface Move {
  id: string; // El ID del template (ej: "scratch")
  name?: string; // Opcional, si el back lo envía
  type?: string;
  power?: number | null;
  accuracy?: number | null;
  pp?: number;
  damage_class?: string;
  target?: string; // Tipo de objetivo del movimiento (ej: "selected-pokemon", "user", "all-opponents")
}

// Especie de Pokémon
export interface PokemonSpecies {
  species_id: string;
  display_name: string;
  generation: number;
  primary_type: string;
  secondary_type: string | null;
  base_stats: {
    hp: number;
    attack: number;
    defense: number;
    special_attack: number;
    special_defense: number;
    speed: number;
  };
  move_pool: string[];
  is_starter_candidate: boolean;
  evolutions: Array<{
    target_species_id: string;
    min_level: number | null;
    trigger: string;
  }>;
}

// Stages de stats en batalla (cambios temporales de -6 a +6)
export interface StatStages {
  attack: number;
  defense: number;
  special_attack: number;
  special_defense: number;
  speed: number;
  accuracy: number;
  evasion: number;
}

// Instancia de Pokémon
export interface PokemonInstance {
  id: string;
  species: PokemonSpecies;
  level: number;
  current_hp: number;
  status_condition: 'Burn' | 'Freeze' | 'Paralysis' | 'Poison' | 'BadPoison' | 'Sleep' | null;
  ability: string; // ID de la habilidad activa (ej: "blaze", "intimidate", "levitate")
  held_item: string | null; // ID del objeto equipado (ej: "leftovers", "life-orb")
  battle_stages: StatStages | null; // Puede ser null si el pokemon no ha entrado en combate
  individual_values: {
    hp: number;
    attack: number;
    defense: number;
    special_attack: number;
    special_defense: number;
    speed: number;
  };
  effort_values: {
    hp: number;
    attack: number;
    defense: number;
    special_attack: number;
    special_defense: number;
    speed: number;
  };
  base_computed_stats: {
    hp: number;
    attack: number;
    defense: number;
    special_attack: number;
    special_defense: number;
    speed: number;
  };
  randomized_profile: {
    rolled_primary_type: string;
    rolled_secondary_type: string | null;
    rolled_ability_id: string;
    stat_modifiers: {
      additive: Record<string, number>;
      multipliers: Record<string, number>;
    };
    moves: Array<{
      template_id: string;
      randomized_type: string | null;
      power_variation: number | null;
      accuracy_multiplier: number | null;
    }>;
    learned_moves: Array<{
      move_id: string;
      current_pp: number;
      max_pp: number;
    }>;
  };
}

// Tipo de clima en batalla
export type WeatherType = 'Sun' | 'Rain' | 'Sandstorm' | 'Hail' | 'None';

// Estado del clima en batalla
export interface WeatherState {
  weather_type: WeatherType;
  turns_remaining: number;
}

// Tipo de terreno en batalla
export type TerrainType = 'Electric' | 'Grassy' | 'Misty' | 'Psychic';

// Estado del terreno en batalla
export interface TerrainState {
  terrain_type: TerrainType;
  turns_remaining: number;
}

// Formato de batalla
export type BattleFormat = 'Single' | 'Double';

// Posición en el campo de batalla
export type FieldPosition = 'PlayerLeft' | 'PlayerRight' | 'OpponentLeft' | 'OpponentRight';

// Estado de batalla
export interface BattleState {
  format: BattleFormat; // Formato de batalla (Single o Double)
  player_active_indices: number[]; // Índices de los Pokémon activos del jugador
  opponent_instance: PokemonInstance;
  is_trainer_battle: boolean;
  opponent_team: PokemonInstance[];
  opponent_active_indices: number[]; // Índices de los Pokémon activos del oponente
  opponent_name: string | null;
  turn_counter: number;
  log: string[]; // Historial de lo que pasó en el turno
  weather: WeatherState | null; // Clima activo en la batalla
  terrain: TerrainState | null; // Terreno activo en la batalla
  // Campos de compatibilidad (deprecated, usar player_active_indices y opponent_active_indices)
  player_active_index?: number;
  opponent_active_index?: number;
}

// Resultado de un turno
export interface TurnResult {
  logs: string[];
  player_damage_dealt: number;
  enemy_damage_dealt: number;
  is_battle_over: boolean;
}

// Tipo de estado del juego
export type GameState = 
  | 'StarterSelection' 
  | 'Map' 
  | 'EncounterSelection' 
  | 'TeamManagement' 
  | 'Battle' 
  | 'GymBattle' 
  | 'LootSelection'
  | 'Completed';

// Configuración de la partida
export interface GameConfig {
  gym_interval: number;
  total_encounters: number;
}

// Sesión de juego
export interface GameSession {
  id: string;
  state: GameState;
  team: {
    active_members: PokemonInstance[];
    box_members: PokemonInstance[];
  };
  starter_choices: PokemonInstance[] | null;
  encounter_choices: PokemonInstance[] | null;
  loot_options: string[] | null; // IDs de objetos de recompensa (ej: ["leftovers", "life-orb", "choice-band"])
  battle: BattleState | null;
  config: GameConfig;
  encounters_won: number;
}

