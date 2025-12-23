import { useState, useEffect } from 'react';
import { api } from './api';
import { GameSession, PokemonInstance, FieldPosition } from './types';
import { NewGameForm, GameConfig } from './components/NewGameForm';
import { StarterSelection } from './components/StarterSelection';
import { MapScreen } from './components/MapScreen';
import { EncounterSelection } from './components/EncounterSelection';
import { TeamManagement } from './components/TeamManagement';
import { BattleScreen } from './components/BattleScreen';
import { TeamScreen } from './components/TeamScreen';
import { GameOverModal } from './components/GameOverModal';
import { VictoryScreen } from './components/VictoryScreen';
import './App.css';

function App() {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [gameState, setGameState] = useState<GameSession | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedPokemon, setSelectedPokemon] = useState<PokemonInstance | null>(null);
  const [showTeamManagement, setShowTeamManagement] = useState(false);
  const [lastConfig, setLastConfig] = useState<GameConfig | null>(null);
  const [showGameOver, setShowGameOver] = useState(false);

  // Función para refrescar el estado del juego
  const refreshGameState = async () => {
    if (!sessionId) return;

    try {
      const state = await api.getGameState(sessionId);
      setGameState(state);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to refresh game state');
    }
  };

  // Refrescar estado después de cada acción
  useEffect(() => {
    if (sessionId) {
      refreshGameState();
    }
  }, [sessionId]);

  // Detectar derrota basándose en el estado del juego
  // El backend revive los Pokémon con 1 HP después de la derrota
  // Detectamos la derrota cuando:
  // 1. Estamos en estado Map (después de perder)
  // 2. No hay batalla activa
  // 3. Todos los Pokémon tienen exactamente 1 HP (fueron revividos)
  // 4. NO estamos en proceso de crear un nuevo juego (loading === false)
  useEffect(() => {
    // No detectar derrota si estamos cargando o si ya se está mostrando el modal
    if (loading || showGameOver || !sessionId) return;
    
    if (gameState && gameState.team.active_members.length > 0 && gameState.state === 'Map' && !gameState.battle) {
      // Verificar si todos los Pokémon tienen exactamente 1 HP (fueron revividos después de perder)
      const allRevivedWith1HP = gameState.team.active_members.every(
        (pokemon) => pokemon.current_hp === 1
      );
      
      // Solo mostrar Game Over si todos tienen 1 HP (fueron revividos) y no hay batalla
      // Esto indica que el jugador perdió y fue revivido
      if (allRevivedWith1HP) {
        setShowGameOver(true);
      }
    }
  }, [gameState, showGameOver, loading, sessionId]);

  // Handler para reintentar con la misma configuración
  const handleRetry = async () => {
    if (!lastConfig) return;
    
    // IMPORTANTE: Ocultar el modal y limpiar el estado ANTES de crear el nuevo juego
    setShowGameOver(false);
    setGameState(null);
    setSessionId(null);
    setError(null);
    setSelectedPokemon(null);
    setLoading(true);

    try {
      const response = await api.newGame({
        generations: lastConfig.generations,
        gym_interval: lastConfig.gym_interval,
        total_encounters: lastConfig.total_encounters,
        chaos_move_randomizer: lastConfig.chaos_move_randomizer,
      });
      setSessionId(response.session_id);
      // El useEffect se encargará de refrescar el estado
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to retry game');
    } finally {
      setLoading(false);
    }
  };

  // Handler para volver al título
  const handleQuit = () => {
    setSessionId(null);
    setGameState(null);
    setShowGameOver(false);
    setSelectedPokemon(null);
  };

  // Iniciar nuevo juego con configuración
  const handleNewGame = async (config: GameConfig) => {
    setLoading(true);
    setError(null);
    setSelectedPokemon(null);
    setShowGameOver(false);
    
    // Guardar la configuración para poder reintentar
    setLastConfig(config);

    try {
      const response = await api.newGame({
        generations: config.generations,
        gym_interval: config.gym_interval,
        total_encounters: config.total_encounters,
        chaos_move_randomizer: config.chaos_move_randomizer,
        preferred_format: config.preferred_format,
      });
      setSessionId(response.session_id);
      // El useEffect se encargará de refrescar el estado
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create new game');
    } finally {
      setLoading(false);
    }
  };

  // Elegir starter
  const handleChooseStarter = async (index: number) => {
    if (!sessionId) return;

    setLoading(true);
    setError(null);

    try {
      await api.chooseStarter(sessionId, index);
      await refreshGameState();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to choose starter');
    } finally {
      setLoading(false);
    }
  };

  // Explorar ruta
  const handleExplore = async () => {
    if (!sessionId) return;

    setLoading(true);
    setError(null);
    setSelectedPokemon(null);

    try {
      await api.explore(sessionId);
      await refreshGameState();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to explore');
    } finally {
      setLoading(false);
    }
  };

  // Seleccionar encuentro
  const handleSelectEncounter = async (index: number) => {
    if (!sessionId) return;

    setLoading(true);
    setError(null);

    try {
      // El backend ahora retorna la sesión completa con el estado de batalla inicializado
      const updatedSession = await api.selectEncounter(sessionId, index);
      setGameState(updatedSession); // Actualizar inmediatamente con la sesión del servidor
      // No necesitamos refreshGameState porque ya tenemos el estado actualizado
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to select encounter');
    } finally {
      setLoading(false);
    }
  };

  // Enviar movimiento en batalla
  const handleMoveSelect = async (moveId: string, userIndex: number, targetPosition?: FieldPosition | null) => {
    if (!sessionId || !gameState) return;

    setLoading(true);
    setError(null);

    try {
      const response = await api.submitMove(sessionId, {
        move_id: moveId,
        user_index: userIndex,
        target_position: targetPosition ?? null,
      });

      // Si el turno no se ejecutó (faltan más acciones en dobles), mostrar mensaje
      if (!response.turn_executed && response.pending_actions > 0) {
        console.log(`Acción registrada. Faltan ${response.pending_actions} acción(es) más.`);
      }

      // Si la batalla terminó y tenemos la sesión actualizada, usarla directamente
      if (response.battle_over && response.session) {
        setGameState(response.session);

        // Si el jugador perdió, mostrar Game Over
        if (response.player_won === false) {
          setShowGameOver(true);
        }
      } else if (response.turn_executed) {
        // Actualizar HP inmediatamente para feedback visual instantáneo
        if (gameState.battle && (response.player_hp !== undefined || response.enemy_hp !== undefined)) {
          setGameState(prev => {
            if (!prev || !prev.battle) return prev;

            const updatedState = { ...prev };
            const battle = updatedState.battle;
            if (!battle) return prev;

            // Actualizar HP del jugador (primer Pokémon activo)
            if (response.player_hp !== undefined && updatedState.team.active_members.length > 0) {
              const playerActiveIndex = battle.player_active_indices?.[0] ?? 0;
              const updatedMembers = [...updatedState.team.active_members];
              if (updatedMembers[playerActiveIndex]) {
                updatedMembers[playerActiveIndex] = {
                  ...updatedMembers[playerActiveIndex],
                  current_hp: response.player_hp,
                };
                updatedState.team = { ...updatedState.team, active_members: updatedMembers };
              }
            }

            // Actualizar HP del enemigo
            if (response.enemy_hp !== undefined) {
              if (battle.is_trainer_battle && battle.opponent_team) {
                const opponentActiveIndex = battle.opponent_active_indices?.[0] ?? 0;
                const updatedOpponents = [...battle.opponent_team];
                if (updatedOpponents[opponentActiveIndex]) {
                  updatedOpponents[opponentActiveIndex] = {
                    ...updatedOpponents[opponentActiveIndex],
                    current_hp: response.enemy_hp,
                  };
                  updatedState.battle = { ...battle, opponent_team: updatedOpponents };
                }
              } else {
                // Wild battle - actualizar opponent_instance
                updatedState.battle = {
                  ...battle,
                  opponent_instance: {
                    ...battle.opponent_instance,
                    current_hp: response.enemy_hp,
                  },
                };
              }
            }

            return updatedState;
          });
        }

        // Refrescar el estado completo de forma asíncrona para mantener consistencia
        await refreshGameState();
      } else {
        // Si el turno no se ejecutó, refrescar para ver las acciones pendientes
        await refreshGameState();
      }

    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to submit move');
    } finally {
      setLoading(false);
    }
  };

  // Cambiar de Pokémon durante la batalla
  const handleSwitchPokemon = async (index: number) => {
    if (!sessionId || !gameState) return;

    setLoading(true);
    setError(null);

    try {
      const response = await api.switchPokemon(sessionId, index);

      // Actualizar HP inmediatamente si están disponibles
      if (response.player_hp !== undefined || response.enemy_hp !== undefined) {
        setGameState(prev => {
          if (!prev || !prev.battle) return response.session;

          const updatedState = { ...response.session };
          const battle = updatedState.battle;
          if (!battle) return response.session;

          // Actualizar HP del jugador si está disponible
          if (response.player_hp !== undefined && updatedState.team.active_members.length > 0) {
            const playerActiveIndex = battle.player_active_indices?.[0] ?? 0;
            const updatedMembers = [...updatedState.team.active_members];
            if (updatedMembers[playerActiveIndex]) {
              updatedMembers[playerActiveIndex] = {
                ...updatedMembers[playerActiveIndex],
                current_hp: response.player_hp,
              };
              updatedState.team = { ...updatedState.team, active_members: updatedMembers };
            }
          }

          // Actualizar HP del enemigo si está disponible
          if (response.enemy_hp !== undefined) {
            if (battle.is_trainer_battle && battle.opponent_team) {
              const opponentActiveIndex = battle.opponent_active_indices?.[0] ?? 0;
              const updatedOpponents = [...battle.opponent_team];
              if (updatedOpponents[opponentActiveIndex]) {
                updatedOpponents[opponentActiveIndex] = {
                  ...updatedOpponents[opponentActiveIndex],
                  current_hp: response.enemy_hp,
                };
                updatedState.battle = { ...battle, opponent_team: updatedOpponents };
              }
            } else {
              updatedState.battle = {
                ...battle,
                opponent_instance: {
                  ...battle.opponent_instance,
                  current_hp: response.enemy_hp,
                },
              };
            }
          }

          return updatedState;
        });
      } else {
        // Si no hay HP values, usar la sesión directamente
        setGameState(response.session);
      }

      // Si la batalla terminó, refrescar el estado completo
      if (response.battle_over) {
        await refreshGameState();

        // Si el jugador perdió, mostrar Game Over
        if (response.player_won === false) {
          setShowGameOver(true);
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to switch pokemon');
    } finally {
      setLoading(false);
    }
  };

  // Renderizar según el estado del juego
  const renderContent = () => {
    if (!sessionId || !gameState) {
      return <NewGameForm onSubmit={handleNewGame} loading={loading} />;
    }

    switch (gameState.state) {
      case 'StarterSelection':
        return (
          <StarterSelection
            starters={gameState.starter_choices || []}
            onChooseStarter={handleChooseStarter}
            loading={loading}
          />
        );

      case 'Map':
        if (showTeamManagement) {
          return (
            <TeamScreen
              session={gameState}
              onBack={() => setShowTeamManagement(false)}
              onUpdateSession={setGameState}
            />
          );
        }
        return (
          <MapScreen
            team={gameState.team.active_members}
            onExplore={handleExplore}
            onManageTeam={() => setShowTeamManagement(true)}
            loading={loading}
          />
        );

      case 'EncounterSelection':
        return (
          <EncounterSelection
            optionsCount={gameState.encounter_choices?.length || 0}
            onSelectEncounter={handleSelectEncounter}
            selectedPokemon={selectedPokemon}
            loading={loading}
          />
        );

      case 'TeamManagement':
        return <TeamManagement />;

      case 'Battle':
      case 'GymBattle':
        return (
          <BattleScreen
            session={gameState}
            onMoveSelect={handleMoveSelect}
            onSwitchPokemon={handleSwitchPokemon}
            isBoss={gameState.state === 'GymBattle'}
          />
        );

      case 'Completed':
        return (
          <VictoryScreen
            session={gameState}
            onRestart={handleQuit}
          />
        );

      default:
        return <div>Estado desconocido: {gameState.state}</div>;
    }
  };

  return (
    <div className="app">
      {error && <div className="error">{error}</div>}
      {renderContent()}
      {showGameOver && (
        <GameOverModal
          onRetry={handleRetry}
          onQuit={handleQuit}
          loading={loading}
        />
      )}
    </div>
  );
}

export default App;

