import { useState, useEffect } from 'react';
import { GameSession } from '../types';
import { HealthBar } from './HealthBar';
import { PokemonSprite } from './PokemonSprite';
import { PokemonSwitchModal } from './PokemonSwitchModal';
import { StatusBadge } from './StatusBadge';
import { StatModifiers } from './StatModifiers';

interface BattleScreenProps {
  session: GameSession;
  onMoveSelect: (moveIndex: number) => void;
  onSwitchPokemon: (index: number) => void;
  isBoss?: boolean;
}

export function BattleScreen({ session, onMoveSelect, onSwitchPokemon, isBoss = false }: BattleScreenProps) {
  const [showSwitchModal, setShowSwitchModal] = useState(false);

  if (!session.battle) {
    return <div>Error: No hay batalla activa</div>;
  }

  const activePokemon = session.team.active_members[session.battle.player_active_index];
  // Obtener el oponente activo (puede cambiar si es batalla de entrenador)
  const opponent = session.battle.is_trainer_battle 
    ? session.battle.opponent_team[session.battle.opponent_active_index]
    : session.battle.opponent_instance;

  if (!activePokemon) {
    return <div>Error: No hay Pokémon activo</div>;
  }

  // Detectar si el Pokémon activo está debilitado
  const isPokemonFainted = activePokemon.current_hp === 0;
  
  // Si el Pokémon está debilitado, abrir automáticamente el modal de cambio
  // y no permitir cerrarlo hasta que se elija un Pokémon
  useEffect(() => {
    if (isPokemonFainted) {
      setShowSwitchModal(true);
    }
  }, [isPokemonFainted]);

  // El species_id ya viene como string numérico con padding (ej: "001", "025")
  // El componente PokemonSprite lo convertirá a número automáticamente

  return (
    <div
      style={{
        width: '100%',
        height: '100vh',
        backgroundColor: '#87CEEB',
        display: 'flex',
        flexDirection: 'column',
        fontFamily: 'monospace',
      }}
    >
      {/* Área Superior - Oponente */}
      <div
        style={{
          flex: 1,
          display: 'flex',
          justifyContent: 'flex-end',
          alignItems: 'flex-start',
          padding: '20px',
        }}
      >
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'flex-end',
            gap: '10px',
          }}
        >
          {/* Caja de Info del Oponente */}
          <div
            className={`bg-white rounded-lg p-3 min-w-[200px] shadow-lg ${
              isBoss ? 'border-4 border-yellow-500 shadow-yellow-500/50' : 'border-2 border-black'
            }`}
          >
            {isBoss && (
              <div className="text-xs font-bold text-yellow-500 mb-1 drop-shadow-md">
                ⭐ LÍDER DE GIMNASIO ⭐
              </div>
            )}
            <div className="flex justify-between items-center mb-2">
              <span className={`font-bold text-base ${isBoss ? 'text-red-900' : 'text-black'}`}>
                {session.battle?.opponent_name || opponent.species.display_name}
              </span>
              <span className="text-sm text-black">Lv. {opponent.level}</span>
            </div>
            
            <div className="flex items-center gap-2 mb-1">
              <HealthBar current={opponent.current_hp} max={opponent.base_computed_stats.hp} />
              <StatusBadge status={opponent.status_condition} />
            </div>

            <StatModifiers stages={opponent.battle_stages} />
          </div>

          {/* Sprite del Oponente */}
          <div
            style={{
              width: '150px',
              height: '150px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <PokemonSprite 
              key={`opponent-${session.battle.opponent_active_index}-${opponent.species.species_id}`}
              speciesId={opponent.species.species_id} 
              back={false} 
            />
          </div>
        </div>
      </div>

      {/* Área Inferior - Jugador */}
      <div
        style={{
          flex: 1,
          display: 'flex',
          justifyContent: 'flex-start',
          alignItems: 'flex-end',
          padding: '20px',
        }}
      >
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'flex-start',
            gap: '10px',
          }}
        >
          {/* Sprite del Jugador */}
          <div
            style={{
              width: '150px',
              height: '150px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <PokemonSprite speciesId={activePokemon.species.species_id} back={true} />
          </div>

          {/* Caja de Info del Jugador */}
          <div className="bg-white border-2 border-black rounded-lg p-3 min-w-[200px] shadow-lg">
            <div className="flex justify-between items-center mb-2">
              <span className="font-bold text-base text-black">
                {activePokemon.species.display_name}
              </span>
              <span className="text-sm text-black">Lv. {activePokemon.level}</span>
            </div>
            
            <div className="flex items-center gap-2 mb-1">
              <HealthBar
                current={activePokemon.current_hp}
                max={activePokemon.base_computed_stats.hp}
                showText={true}
              />
              <StatusBadge status={activePokemon.status_condition} />
            </div>

            <StatModifiers stages={activePokemon.battle_stages} />
          </div>
        </div>
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
        }}
      >
        {/* Logs (Izquierda) */}
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

        {/* Movimientos y Botón de Cambio (Derecha) */}
        <div
          style={{
            flex: 1,
            display: 'flex',
            flexDirection: 'column',
            gap: '8px',
          }}
        >
          {/* Mensaje de alerta si el Pokémon está debilitado */}
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

          {/* Grid de Movimientos - Solo los primeros 4 activos */}
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: '1fr 1fr',
              gap: '8px',
            }}
          >
            {activePokemon.randomized_profile.moves.slice(0, 4).map((move, index) => (
              <button
                key={index}
                onClick={() => onMoveSelect(index)}
                disabled={isPokemonFainted}
                style={{
                  backgroundColor: isPokemonFainted ? '#9ca3af' : '#4a5568',
                  color: '#ffffff',
                  border: '2px solid #000000',
                  borderRadius: '4px',
                  padding: '12px',
                  fontSize: '14px',
                  fontWeight: 'bold',
                  cursor: isPokemonFainted ? 'not-allowed' : 'pointer',
                  transition: 'background-color 0.2s',
                  opacity: isPokemonFainted ? 0.6 : 1,
                }}
                onMouseEnter={(e) => {
                  if (!isPokemonFainted) {
                    e.currentTarget.style.backgroundColor = '#5a6578';
                  }
                }}
                onMouseLeave={(e) => {
                  if (!isPokemonFainted) {
                    e.currentTarget.style.backgroundColor = '#4a5568';
                  }
                }}
              >
                {move.template_id}
              </button>
            ))}
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
    </div>
  );
}

