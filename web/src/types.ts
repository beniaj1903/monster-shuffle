// Tipos base para movimientos
export interface Move {
  id: string; // El ID del template (ej: "scratch")
  name?: string; // Opcional, si el back lo envía
  type?: string;
  power?: number | null;
  accuracy?: number | null;
  pp?: number;
  damage_class?: string;
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

// Instancia de Pokémon
export interface PokemonInstance {
  id: string;
  species: PokemonSpecies;
  level: number;
  current_hp: number;
  status_condition: string | null;
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
  };
}

// Estado de batalla
export interface BattleState {
  player_active_index: number;
  opponent_instance: PokemonInstance;
  is_trainer_battle: boolean;
  opponent_team: PokemonInstance[];
  opponent_active_index: number;
  opponent_name: string | null;
  turn_counter: number;
  log: string[]; // Historial de lo que pasó en el turno
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
  battle: BattleState | null;
  config: GameConfig;
  encounters_won: number;
}

