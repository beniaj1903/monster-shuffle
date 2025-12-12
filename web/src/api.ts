import {
  GameSession,
  PokemonInstance,
  TurnResult,
} from './types';

// El proxy de Vite redirige /api a http://localhost:3000/api
const API_BASE_URL = '/api';

// Respuestas de la API
export interface NewGameResponse {
  session_id: string;
  starters: PokemonInstance[];
}

export interface ExploreResponse {
  options_count: number;
}

// SelectEncounterResponse ahora es GameSession (definido en types.ts)

// Cliente API
export const api = {
  /**
   * Inicia un nuevo juego
   */
  async newGame(config?: {
    generations?: number[];
    gym_interval?: number;
    total_encounters?: number;
    chaos_move_randomizer?: boolean;
  }): Promise<NewGameResponse> {
    const payload: {
      generations?: number[];
      gym_interval?: number;
      total_encounters?: number;
      chaos_move_randomizer?: boolean;
    } = {};

    if (config) {
      if (config.generations !== undefined) {
        payload.generations = config.generations;
      }
      if (config.gym_interval !== undefined) {
        payload.gym_interval = config.gym_interval;
      }
      if (config.total_encounters !== undefined) {
        payload.total_encounters = config.total_encounters;
      }
      if (config.chaos_move_randomizer !== undefined) {
        payload.chaos_move_randomizer = config.chaos_move_randomizer;
      }
    }

    const response = await fetch(`${API_BASE_URL}/game/new`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      throw new Error(`Failed to create new game: ${response.statusText}`);
    }

    return response.json();
  },

  /**
   * Elige un starter
   */
  async chooseStarter(sessionId: string, starterIndex: number): Promise<GameSession> {
    const response = await fetch(`${API_BASE_URL}/game/choose-starter`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        session_id: sessionId,
        starter_index: starterIndex,
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to choose starter: ${response.statusText}`);
    }

    return response.json();
  },

  /**
   * Explora una ruta (genera encuentros)
   */
  async explore(sessionId: string): Promise<ExploreResponse> {
    const response = await fetch(`${API_BASE_URL}/game/explore`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        session_id: sessionId,
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to explore: ${response.statusText}`);
    }

    return response.json();
  },

  /**
   * Selecciona un encuentro
   * Retorna la sesión completa con el estado de batalla inicializado
   */
  async selectEncounter(
    sessionId: string,
    selectionIndex: number
  ): Promise<GameSession> {
    const response = await fetch(`${API_BASE_URL}/game/select-encounter`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        session_id: sessionId,
        selection_index: selectionIndex,
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to select encounter: ${response.statusText}`);
    }

    return response.json();
  },

  /**
   * Obtiene el estado actual del juego
   */
  async getGameState(sessionId: string): Promise<GameSession> {
    const response = await fetch(`${API_BASE_URL}/game/${sessionId}`);

    if (!response.ok) {
      throw new Error(`Failed to get game state: ${response.statusText}`);
    }

    return response.json();
  },

  /**
   * Envía un movimiento en batalla
   */
  async submitMove(sessionId: string, moveIndex: number): Promise<{
    result: TurnResult;
    player_hp: number;
    enemy_hp: number;
    battle_over: boolean;
    player_won: boolean | null;
    session?: GameSession;
  }> {
    const response = await fetch(`${API_BASE_URL}/game/battle/move`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        session_id: sessionId,
        move_index: moveIndex,
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to submit move: ${response.statusText}`);
    }

    return response.json();
  },

  /**
   * Cambia de Pokémon durante la batalla
   */
  async switchPokemon(sessionId: string, index: number): Promise<{
    result: TurnResult;
    player_hp: number;
    enemy_hp: number;
    battle_over: boolean;
    player_won: boolean | null;
    session: GameSession;
  }> {
    const response = await fetch(`${API_BASE_URL}/game/battle/switch`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        session_id: sessionId,
        switch_to_index: index,
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to switch pokemon: ${response.statusText}`);
    }

    return response.json();
  },

  /**
   * Reordena el equipo
   */
  async reorderTeam(sessionId: string, newOrder: number[]): Promise<GameSession> {
    const response = await fetch(`${API_BASE_URL}/game/team/reorder`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        session_id: sessionId,
        new_order: newOrder,
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to reorder team: ${response.statusText}`);
    }

    return response.json();
  },

  /**
   * Reordena los movimientos de un Pokémon
   */
  async reorderMoves(
    sessionId: string,
    pokemonIndex: number,
    moveIndices: number[]
  ): Promise<GameSession> {
    const response = await fetch(`${API_BASE_URL}/game/pokemon/move-reorder`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        session_id: sessionId,
        pokemon_index: pokemonIndex,
        move_indices: moveIndices,
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to reorder moves: ${response.statusText}`);
    }

    return response.json();
  },

  /**
   * Evoluciona un Pokémon
   */
  async evolvePokemon(
    sessionId: string,
    pokemonIndex: number,
    targetSpeciesId: string
  ): Promise<GameSession> {
    const response = await fetch(`${API_BASE_URL}/game/pokemon/evolve`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        session_id: sessionId,
        pokemon_index: pokemonIndex,
        target_species_id: targetSpeciesId,
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to evolve pokemon: ${response.statusText}`);
    }

    return response.json();
  },
};

