import { useState, useEffect } from 'react';
import { GameSession, FieldPosition, Move } from '../types';
import { HealthBar } from './HealthBar';
import { PokemonSprite } from './PokemonSprite';
import { PokemonSwitchModal } from './PokemonSwitchModal';
import { StatusBadge } from './StatusBadge';
import { StatModifiers } from './StatModifiers';
import { ItemIcon } from './ItemIcon';
import { WeatherIndicator } from './WeatherIndicator';
import { TerrainIndicator } from './TerrainIndicator';
import { TargetSelectionModal } from './TargetSelectionModal';
import { api } from '../api';

interface BattleScreenProps {
  session: GameSession;
  onMoveSelect: (moveId: string, userIndex: number, targetPosition?: FieldPosition | null) => void;
  onSwitchPokemon: (index: number) => void;
  isBoss?: boolean;
}

// Helper para formatear el nombre de la habilidad
function formatAbilityName(abilityId: string): string {
  return abilityId
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

export function BattleScreen({ session, onMoveSelect, onSwitchPokemon, isBoss = false }: BattleScreenProps) {
  const [showSwitchModal, setShowSwitchModal] = useState(false);
  const [pendingMove, setPendingMove] = useState<{
    moveId: string;
    moveName: string;
    userIndex: number;
  } | null>(null);
  const [movesMap, setMovesMap] = useState<Record<string, Move>>({});

  // Cargar los movimientos al montar el componente
  useEffect(() => {
    const loadMoves = async () => {
      try {
        const moves = await api.getAllMoves();
        setMovesMap(moves);
        console.log('[DEBUG] BattleScreen: Movimientos cargados:', Object.keys(moves).length);
      } catch (error) {
        console.error('[DEBUG] BattleScreen: Error al cargar movimientos:', error);
        // Si falla, usar un mapa vacío (se usará 'selected-pokemon' por defecto)
      }
    };
    loadMoves();
  }, []);

  if (!session.battle) {
    return <div>Error: No hay batalla activa</div>;
  }

  const battleFormat = session.battle.format || 'Single';
  const isDouble = battleFormat === 'Double';

  // Obtener los índices activos del jugador
  const player_active_indices = session.battle.player_active_indices || [session.battle.player_active_index ?? 0];
  const player_left_index = player_active_indices[0] ?? 0;
  const player_right_index = player_active_indices[1];
  
  // Obtener los oponentes activos
  const opponent_active_indices = session.battle.opponent_active_indices || [session.battle.opponent_active_index ?? 0];
  const opponent_left_index = opponent_active_indices[0] ?? 0;
  const opponent_right_index = opponent_active_indices[1];

  // Obtener los Pokémon activos
  const playerLeft = session.team.active_members[player_left_index];
  const playerRight = player_right_index !== undefined ? session.team.active_members[player_right_index] : null;
  
  const opponentLeft = session.battle.is_trainer_battle
    ? session.battle.opponent_team[opponent_left_index]
    : session.battle.opponent_instance;
  const opponentRight = session.battle.is_trainer_battle && opponent_right_index !== undefined
    ? session.battle.opponent_team[opponent_right_index]
    : null;

  // Para compatibilidad con Single, usar el primer Pokémon activo
  const activePokemon = playerLeft;

  if (!activePokemon) {
    return <div>Error: No hay Pokémon activo</div>;
  }

  // Detectar si algún Pokémon del jugador está debilitado
  const isPokemonFainted = activePokemon.current_hp === 0;
  
  useEffect(() => {
    if (isPokemonFainted) {
      setShowSwitchModal(true);
    }
  }, [isPokemonFainted]);

  // Determinar el color de fondo según el clima
  const getBackgroundColor = () => {
    if (!session.battle?.weather) {
      return '#87CEEB';
    }
    switch (session.battle.weather.weather_type) {
      case 'Sun': return '#fed7aa';
      case 'Rain': return '#dbeafe';
      case 'Sandstorm': return '#f5f5f4';
      case 'Hail': return '#ecfeff';
      default: return '#87CEEB';
    }
  };

  // Determinar el color del gradiente del suelo según el terreno
  const getTerrainGradient = () => {
    if (!session.battle?.terrain) return null;
    switch (session.battle.terrain.terrain_type) {
      case 'Electric':
        return 'linear-gradient(to top, rgba(254, 240, 138, 0.6), rgba(254, 240, 138, 0.2), transparent)';
      case 'Grassy':
        return 'linear-gradient(to top, rgba(187, 247, 208, 0.6), rgba(187, 247, 208, 0.2), transparent)';
      case 'Misty':
        return 'linear-gradient(to top, rgba(251, 207, 232, 0.6), rgba(251, 207, 232, 0.2), transparent)';
      case 'Psychic':
        return 'linear-gradient(to top, rgba(221, 214, 254, 0.6), rgba(221, 214, 254, 0.2), transparent)';
      default: return null;
    }
  };

  const terrainGradient = getTerrainGradient();

  // Manejar selección de movimiento
  const handleMoveClick = (moveId: string, userIndex: number) => {
    console.log('[DEBUG] BattleScreen: handleMoveClick - moveId:', moveId, 'userIndex:', userIndex, 'battleFormat:', battleFormat);
    
    // Obtener el target del movimiento desde la base de datos
    const moveData = movesMap[moveId];
    const moveTarget = moveData?.target || 'selected-pokemon'; // Por defecto 'selected-pokemon' si no se encuentra
    console.log('[DEBUG] BattleScreen: moveTarget para', moveId, ':', moveTarget, '(desde DB:', !!moveData, ')');
    
    // Lógica según el tipo de target
    switch (moveTarget) {
      case 'user':
        // Movimientos que afectan al usuario mismo (Swords Dance, Recover, etc.)
        // NUNCA pedir selección, enviar inmediatamente con target_position: null
        console.log('[DEBUG] BattleScreen: target="user" - Enviando inmediatamente sin selección');
        onMoveSelect(moveId, userIndex, null);
        break;
        
      case 'random-opponent':
      case 'all-opponents':
      case 'all-other-pokemon':
      case 'users-field':
      case 'opponents-field':
        // Movimientos de área o automáticos - enviar inmediatamente
        console.log('[DEBUG] BattleScreen: target="' + moveTarget + '" - Enviando inmediatamente sin selección');
        onMoveSelect(moveId, userIndex, null);
        break;
        
      case 'ally':
        // Movimientos que afectan al aliado (Healing Wish, Helping Hand, etc.)
        if (isDouble) {
          // En dobles, mostrar modal para seleccionar al aliado
          console.log('[DEBUG] BattleScreen: target="ally" en Double - Mostrando modal de selección');
          setPendingMove({ moveId, moveName: moveId, userIndex });
        } else {
          // En Single, no hay aliado, enviar con null (el backend manejará el error)
          console.log('[DEBUG] BattleScreen: target="ally" en Single - Enviando con null (no hay aliado)');
          onMoveSelect(moveId, userIndex, null);
        }
        break;
        
      case 'selected-pokemon':
      default:
        // Movimientos que requieren selección de objetivo específico
        if (isDouble) {
          // En dobles, mostrar modal de selección de objetivo
          console.log('[DEBUG] BattleScreen: target="selected-pokemon" en Double - Mostrando modal de selección');
          setPendingMove({ moveId, moveName: moveId, userIndex });
        } else {
          // En Single: automáticamente asignar OpponentLeft
          console.log('[DEBUG] BattleScreen: target="selected-pokemon" en Single - Asignando automáticamente OpponentLeft');
          onMoveSelect(moveId, userIndex, 'OpponentLeft');
        }
        break;
    }
  };

  // Obtener objetivos disponibles para un movimiento según su tipo de target
  const getAvailableTargets = (moveId?: string, userIndex?: number): FieldPosition[] => {
    const targets: FieldPosition[] = [];
    
    // Si se proporciona moveId, obtener el target del movimiento desde la base de datos
    const moveData = moveId ? movesMap[moveId] : undefined;
    const moveTarget = moveData?.target || null;
    
    // Determinar qué objetivos son válidos según el tipo de target
    const needsOpponents = !moveTarget || moveTarget === 'selected-pokemon' || moveTarget === 'random-opponent' || moveTarget === 'all-opponents' || moveTarget === 'all-other-pokemon';
    const needsAllies = moveTarget === 'ally' || moveTarget === 'all-other-pokemon';
    
    // Agregar oponentes vivos (si el movimiento puede afectarlos)
    if (needsOpponents) {
      if (opponentLeft && opponentLeft.current_hp > 0) {
        targets.push('OpponentLeft');
      }
      if (opponentRight && opponentRight.current_hp > 0) {
        targets.push('OpponentRight');
      }
    }
    
    // Agregar aliados vivos (si el movimiento puede afectarlos y es dobles)
    if (needsAllies && isDouble && userIndex !== undefined) {
      // Determinar la posición del usuario actual basándose en userIndex
      // userIndex 0 = PlayerLeft, userIndex 1 = PlayerRight
      const isUserLeft = userIndex === 0;
      
      // Agregar el aliado (el otro Pokémon del jugador)
      if (isUserLeft && playerRight && playerRight.current_hp > 0) {
        targets.push('PlayerRight');
      } else if (!isUserLeft && playerLeft && playerLeft.current_hp > 0) {
        targets.push('PlayerLeft');
      }
    }
    
    return targets;
  };

  // Renderizar un slot de Pokémon (jugador u oponente)
  const renderPokemonSlot = (
    pokemon: typeof playerLeft | null,
    position: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right',
    isPlayer: boolean,
    isLeft: boolean
  ) => {
    if (!pokemon) return null;

    const isFainted = pokemon.current_hp === 0;
    const slotStyle: React.CSSProperties = {
      display: 'flex',
      flexDirection: isPlayer ? 'column' : 'column-reverse',
      alignItems: position.includes('left') ? 'flex-start' : 'flex-end',
      justifyContent: 'center',
      gap: '10px',
      padding: '10px',
      opacity: isFainted ? 0.5 : 1,
    };

    return (
      <div key={`${isPlayer ? 'player' : 'opponent'}-${isLeft ? 'left' : 'right'}`} style={slotStyle}>
        {/* Sprite */}
        <div style={{ width: '120px', height: '120px', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <PokemonSprite
            speciesId={pokemon.species.species_id}
            back={isPlayer}
          />
        </div>

        {/* Info Box */}
        <div
          className={`bg-white rounded-lg p-2 min-w-[180px] shadow-lg ${
            isBoss && !isPlayer ? 'border-4 border-yellow-500 shadow-yellow-500/50' : 'border-2 border-black'
          }`}
        >
          {isBoss && !isPlayer && (
            <div className="text-xs font-bold text-yellow-500 mb-1 drop-shadow-md">
              ⭐ LÍDER ⭐
            </div>
          )}
          <div className="flex justify-between items-center mb-1">
            <span className={`font-bold text-sm ${isBoss && !isPlayer ? 'text-red-900' : 'text-black'}`}>
              {pokemon.species.display_name}
            </span>
            <span className="text-xs text-black">Lv. {pokemon.level}</span>
          </div>
          
          <div className="text-xs text-gray-600 mb-1" title={`ID: ${pokemon.ability}`}>
            {formatAbilityName(pokemon.ability)}
          </div>
          
          <div className="flex items-center gap-2 mb-1">
            <HealthBar
              current={pokemon.current_hp}
              max={pokemon.base_computed_stats.hp}
              showText={isPlayer}
            />
            <StatusBadge status={pokemon.status_condition} />
            {pokemon.held_item && <ItemIcon itemId={pokemon.held_item} />}
          </div>

          <StatModifiers stages={pokemon.battle_stages} />
        </div>
      </div>
    );
  };

  return (
    <div
      style={{
        width: '100%',
        height: '100vh',
        backgroundColor: getBackgroundColor(),
        display: 'flex',
        flexDirection: 'column',
        fontFamily: 'monospace',
        position: 'relative',
      }}
    >
      {/* Indicadores de Clima y Terreno */}
      {session.battle?.weather && <WeatherIndicator weather={session.battle.weather} />}
      {session.battle?.terrain && <TerrainIndicator terrain={session.battle.terrain} />}
      
      {/* Efecto de fondo del terreno */}
      {terrainGradient && (
        <div
          style={{
            position: 'absolute',
            bottom: 0,
            left: 0,
            right: 0,
            height: '40%',
            background: terrainGradient,
            pointerEvents: 'none',
            zIndex: 0,
          }}
        />
      )}
      
      {/* Grid de Batalla 2x2 (o centrado para Single) */}
      <div
        style={{
          flex: 1,
          display: 'grid',
          gridTemplateColumns: isDouble ? '1fr 1fr' : '1fr',
          gridTemplateRows: isDouble ? '1fr 1fr' : '1fr',
          gap: '20px',
          padding: '20px',
          position: 'relative',
          zIndex: 1,
          alignItems: isDouble ? 'stretch' : 'center',
          justifyContent: isDouble ? 'stretch' : 'center',
        }}
      >
        {/* Fila Superior - Oponentes */}
        {isDouble ? (
          <>
            {renderPokemonSlot(opponentLeft, 'top-left', false, true)}
            {renderPokemonSlot(opponentRight, 'top-right', false, false)}
          </>
        ) : (
          <div style={{ display: 'flex', justifyContent: 'center', width: '100%' }}>
            {renderPokemonSlot(opponentLeft, 'top-left', false, true)}
          </div>
        )}

        {/* Fila Inferior - Jugadores */}
        {isDouble ? (
          <>
            {renderPokemonSlot(playerLeft, 'bottom-left', true, true)}
            {renderPokemonSlot(playerRight, 'bottom-right', true, false)}
          </>
        ) : (
          <div style={{ display: 'flex', justifyContent: 'center', width: '100%' }}>
            {renderPokemonSlot(playerLeft, 'bottom-left', true, true)}
          </div>
        )}
      </div>

      {/* Panel de Control */}
      <div
        style={{
          height: '200px',
          backgroundColor: '#2d3748',
          borderTop: '4px solid #000000',
          display: 'flex',
          gap: '10px',
          padding: '10px',
          position: 'relative',
          zIndex: 1,
        }}
      >
        {/* Logs */}
        <div
          style={{
            flex: 1,
            backgroundColor: '#ffffff',
            color: 'black',
            border: '2px solid #000000',
            borderRadius: '4px',
            padding: '10px',
            overflowY: 'auto',
            fontSize: '14px',
            fontFamily: 'monospace',
          }}
        >
          {session.battle.log.length === 0 ? (
            <div style={{ color: '#666' }}>Esperando acción...</div>
          ) : (
            session.battle.log.map((log, index) => (
              <div key={index} style={{ marginBottom: '4px' }}>
                {log}
              </div>
            ))
          )}
        </div>

        {/* Movimientos y Botón de Cambio */}
        <div
          style={{
            flex: 1,
            display: 'flex',
            flexDirection: 'column',
            gap: '8px',
          }}
        >
          {isPokemonFainted && (
            <div
              style={{
                backgroundColor: '#ef4444',
                color: '#ffffff',
                padding: '12px',
                borderRadius: '4px',
                marginBottom: '8px',
                textAlign: 'center',
                fontWeight: 'bold',
                fontSize: '14px',
              }}
            >
              ¡Tu Pokémon se debilitó! Elige otro.
            </div>
          )}

          {/* Grid de Movimientos */}
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: '1fr 1fr',
              gap: '8px',
            }}
          >
            {(() => {
              const movesToShow = activePokemon.randomized_profile.learned_moves?.length > 0
                ? activePokemon.randomized_profile.learned_moves.slice(0, 4).map(m => ({
                    move_id: m.move_id,
                    current_pp: m.current_pp,
                    max_pp: m.max_pp,
                  }))
                : activePokemon.randomized_profile.moves.slice(0, 4).map(m => ({
                    move_id: m.template_id,
                    current_pp: 0,
                    max_pp: 0,
                  }));
              
              return movesToShow.map((move, index) => (
                <button
                  key={index}
                  onClick={() => handleMoveClick(move.move_id, 0)}
                  disabled={isPokemonFainted || move.current_pp === 0}
                  style={{
                    backgroundColor: isPokemonFainted || move.current_pp === 0 ? '#9ca3af' : '#4a5568',
                    color: '#ffffff',
                    border: '2px solid #000000',
                    borderRadius: '4px',
                    padding: '12px',
                    fontSize: '14px',
                    fontWeight: 'bold',
                    cursor: isPokemonFainted || move.current_pp === 0 ? 'not-allowed' : 'pointer',
                    transition: 'background-color 0.2s',
                    opacity: isPokemonFainted || move.current_pp === 0 ? 0.6 : 1,
                  }}
                  onMouseEnter={(e) => {
                    if (!isPokemonFainted && move.current_pp > 0) {
                      e.currentTarget.style.backgroundColor = '#5a6578';
                    }
                  }}
                  onMouseLeave={(e) => {
                    if (!isPokemonFainted && move.current_pp > 0) {
                      e.currentTarget.style.backgroundColor = '#4a5568';
                    }
                  }}
                  title={move.max_pp > 0 ? `PP: ${move.current_pp}/${move.max_pp}` : undefined}
                >
                  {move.move_id}
                  {move.max_pp > 0 && (
                    <span style={{ fontSize: '10px', display: 'block', marginTop: '2px' }}>
                      PP: {move.current_pp}/{move.max_pp}
                    </span>
                  )}
                </button>
              ));
            })()}
          </div>

          {/* Botón de Cambiar Pokémon */}
          <button
            onClick={() => setShowSwitchModal(true)}
            style={{
              backgroundColor: '#10b981',
              color: '#ffffff',
              border: '2px solid #000000',
              borderRadius: '4px',
              padding: '12px',
              fontSize: '14px',
              fontWeight: 'bold',
              cursor: 'pointer',
              transition: 'background-color 0.2s',
              marginTop: '4px',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = '#059669';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = '#10b981';
            }}
          >
            Pokémon
          </button>
        </div>
      </div>

      {/* Modal de Cambio de Pokémon */}
      <PokemonSwitchModal
        session={session}
        isOpen={showSwitchModal}
        onClose={isPokemonFainted ? undefined : () => setShowSwitchModal(false)}
        onSelect={(index) => {
          onSwitchPokemon(index);
          setShowSwitchModal(false);
        }}
        forceSwitch={isPokemonFainted}
      />

      {/* Modal de Selección de Objetivo */}
      <TargetSelectionModal
        isOpen={pendingMove !== null}
        moveName={pendingMove?.moveName || ''}
        availableTargets={pendingMove ? getAvailableTargets(pendingMove.moveId, pendingMove.userIndex) : []}
        onSelect={(target) => {
          if (pendingMove) {
            onMoveSelect(pendingMove.moveId, pendingMove.userIndex, target);
            setPendingMove(null);
          }
        }}
        onCancel={() => setPendingMove(null)}
      />
    </div>
  );
}
