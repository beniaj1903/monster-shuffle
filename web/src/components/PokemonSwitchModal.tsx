import { GameSession } from '../types';
import { HealthBar } from './HealthBar';
import { PokemonSprite } from './PokemonSprite';

interface PokemonSwitchModalProps {
  session: GameSession;
  isOpen: boolean;
  onClose?: () => void;
  onSelect: (index: number) => void;
  forceSwitch?: boolean; // Si es true, no se puede cerrar el modal hasta elegir un Pokémon
}

export function PokemonSwitchModal({
  session,
  isOpen,
  onClose,
  onSelect,
  forceSwitch = false,
}: PokemonSwitchModalProps) {
  if (!isOpen || !session.battle) {
    return null;
  }

  const activeIndex = session.battle.player_active_index;
  const team = session.team.active_members;

  const handleSelect = (index: number) => {
    // Validar que no sea el activo y que tenga HP > 0
    if (index === activeIndex) {
      return; // Ya está activo
    }

    const pokemon = team[index];
    if (pokemon.current_hp === 0) {
      return; // Está debilitado
    }

    onSelect(index);
    if (onClose) {
      onClose();
    }
  };

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.7)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}
      onClick={onClose}
    >
      <div
        style={{
          backgroundColor: '#ffffff',
          border: '4px solid #000000',
          borderRadius: '12px',
          padding: '20px',
          maxWidth: '500px',
          width: '90%',
          maxHeight: '80vh',
          overflowY: 'auto',
          boxShadow: '0 8px 16px rgba(0,0,0,0.3)',
          color: '#000000',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            marginBottom: '20px',
          }}
        >
          <h2
            style={{
              margin: 0,
              fontSize: '24px',
              fontWeight: 'bold',
              color: '#000000',
            }}
          >
            Cambiar Pokémon
          </h2>
          {!forceSwitch && onClose && (
            <button
              onClick={onClose}
              style={{
                backgroundColor: '#ef4444',
                color: '#ffffff',
                border: '2px solid #000000',
                borderRadius: '4px',
                padding: '8px 16px',
                fontSize: '16px',
                fontWeight: 'bold',
                cursor: 'pointer',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = '#dc2626';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = '#ef4444';
              }}
            >
              ✕
            </button>
          )}
          {forceSwitch && (
            <div
              style={{
                fontSize: '12px',
                color: '#ef4444',
                fontWeight: 'bold',
                padding: '8px 16px',
              }}
            >
              ¡Debes elegir un Pokémon!
            </div>
          )}
        </div>

        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(2, 1fr)',
            gap: '12px',
          }}
        >
          {team.map((pokemon, index) => {
            const isActive = index === activeIndex;
            const isFainted = pokemon.current_hp === 0;
            const canSelect = !isActive && !isFainted;

            return (
              <button
                key={pokemon.id}
                onClick={() => handleSelect(index)}
                disabled={!canSelect}
                style={{
                  backgroundColor: isActive
                    ? '#9ca3af'
                    : isFainted
                    ? '#6b7280'
                    : '#ffffff',
                  border: isActive
                    ? '3px solid #4b5563'
                    : isFainted
                    ? '3px solid #374151'
                    : '3px solid #000000',
                  borderRadius: '8px',
                  padding: '12px',
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  gap: '8px',
                  cursor: canSelect ? 'pointer' : 'not-allowed',
                  opacity: canSelect ? 1 : 0.6,
                  transition: 'transform 0.2s, box-shadow 0.2s',
                }}
                onMouseEnter={(e) => {
                  if (canSelect) {
                    e.currentTarget.style.transform = 'scale(1.05)';
                    e.currentTarget.style.boxShadow = '0 4px 8px rgba(0,0,0,0.3)';
                  }
                }}
                onMouseLeave={(e) => {
                  if (canSelect) {
                    e.currentTarget.style.transform = 'scale(1)';
                    e.currentTarget.style.boxShadow = 'none';
                  }
                }}
              >
                {/* Sprite */}
                <div
                  style={{
                    width: '64px',
                    height: '64px',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  <PokemonSprite
                    speciesId={pokemon.species.species_id}
                    back={false}
                  />
                </div>

                {/* Nombre y Nivel */}
                <div
                  style={{
                    textAlign: 'center',
                    width: '100%',
                  }}
                >
                  <div
                    style={{
                      fontWeight: 'bold',
                      fontSize: '14px',
                      marginBottom: '4px',
                      color: isActive || isFainted ? '#6b7280' : '#000000',
                    }}
                  >
                    {pokemon.species.display_name}
                  </div>
                  <div
                    style={{
                      fontSize: '12px',
                      color: isActive || isFainted ? '#9ca3af' : '#666666',
                      marginBottom: '8px',
                    }}
                  >
                    Nv.{pokemon.level}
                  </div>

                  {/* Barra de Vida */}
                  <div style={{ width: '100%' }}>
                    <HealthBar
                      current={pokemon.current_hp}
                      max={pokemon.base_computed_stats.hp}
                    />
                  </div>
                </div>

                {/* Estado del Botón */}
                {isActive && (
                  <div
                    style={{
                      fontSize: '11px',
                      color: '#4b5563',
                      fontWeight: 'bold',
                      marginTop: '4px',
                    }}
                  >
                    En combate
                  </div>
                )}
                {isFainted && (
                  <div
                    style={{
                      fontSize: '11px',
                      color: '#dc2626',
                      fontWeight: 'bold',
                      marginTop: '4px',
                    }}
                  >
                    Debilitado
                  </div>
                )}
              </button>
            );
          })}
        </div>

        {team.length === 0 && (
          <div
            style={{
              textAlign: 'center',
              padding: '20px',
              color: '#666666',
            }}
          >
            No hay Pokémon en el equipo
          </div>
        )}
      </div>
    </div>
  );
}

