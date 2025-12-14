import {
  GameSession,
  PokemonInstance,
  TurnResult,
  FieldPosition,
  Move,
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
    preferred_format?: 'Single' | 'Double';
  }): Promise<NewGameResponse> {
    const payload: {
      generations?: number[];
      gym_interval?: number;
      total_encounters?: number;
      chaos_move_randomizer?: boolean;
      preferred_format?: 'Single' | 'Double';
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
      if (config.preferred_format !== undefined) {
        payload.preferred_format = config.preferred_format;
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
  async submitMove(
    sessionId: string,
    moveInput: {
      move_id: string;
      user_index: number;
      target_position?: FieldPosition | null;
    }
  ): Promise<{
    result: TurnResult;
    player_hp: number;
    enemy_hp: number;
    battle_over: boolean;
    player_won: boolean | null;
    session?: GameSession;
    turn_executed: boolean;
    pending_actions: number;
  }> {
    // El backend espera los campos directamente en el nivel superior, no anidados en move_input
    const payload = {
      session_id: sessionId,
      move_id: moveInput.move_id,
      user_index: moveInput.user_index,
      target_position: moveInput.target_position ?? null,
    };
    
    console.log('[DEBUG] submitMove: Enviando payload:', payload);
    
    const response = await fetch(`${API_BASE_URL}/game/battle/move`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      // Intentar obtener más información del error
      let errorMessage = `Failed to submit move: ${response.statusText}`;
      try {
        const errorBody = await response.text();
        console.error('[DEBUG] submitMove: Error response body:', errorBody);
        if (errorBody) {
          try {
            const errorJson = JSON.parse(errorBody);
            errorMessage = `Failed to submit move: ${errorJson.message || errorJson.error || response.statusText}`;
          } catch {
            errorMessage = `Failed to submit move: ${response.statusText} - ${errorBody}`;
          }
        }
      } catch (e) {
        console.error('[DEBUG] submitMove: Error al leer el cuerpo de la respuesta:', e);
      }
      throw new Error(errorMessage);
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

  /**
   * Obtiene todos los movimientos disponibles
   */
  async getAllMoves(): Promise<Record<string, Move>> {
    const response = await fetch(`${API_BASE_URL}/moves`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      throw new Error(`Failed to get moves: ${response.statusText}`);
    }

    const data = await response.json();
    return data.moves;
  },
};

