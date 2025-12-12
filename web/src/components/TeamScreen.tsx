import { useState } from 'react';
import { GameSession } from '../types';
import { api } from '../api';
import { PokemonSprite } from './PokemonSprite';
import { HealthBar } from './HealthBar';

interface TeamScreenProps {
  session: GameSession;
  onBack: () => void;
  onUpdateSession: (session: GameSession) => void;
}

export function TeamScreen({ session, onBack, onUpdateSession }: TeamScreenProps) {
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const [localTeamOrder, setLocalTeamOrder] = useState<number[]>(
    session.team.active_members.map((_, i) => i)
  );
  const [localMoveOrders, setLocalMoveOrders] = useState<Map<number, number[]>>(new Map());
  const [loading, setLoading] = useState(false);
  const [evolutionMessage, setEvolutionMessage] = useState<string | null>(null);

  const team = session.team.active_members;
  const selectedPokemon = selectedIndex !== null ? team[selectedIndex] : null;

  // Inicializar orden de movimientos si no existe
  if (selectedIndex !== null && !localMoveOrders.has(selectedIndex)) {
    const moveOrder = team[selectedIndex].randomized_profile.moves.map((_, i) => i);
    setLocalMoveOrders(new Map(localMoveOrders.set(selectedIndex, moveOrder)));
  }

  const handleMoveUp = (index: number) => {
    if (index === 0) return;
    const newOrder = [...localTeamOrder];
    [newOrder[index - 1], newOrder[index]] = [newOrder[index], newOrder[index - 1]];
    setLocalTeamOrder(newOrder);
  };

  const handleMoveDown = (index: number) => {
    if (index === localTeamOrder.length - 1) return;
    const newOrder = [...localTeamOrder];
    [newOrder[index], newOrder[index + 1]] = [newOrder[index + 1], newOrder[index]];
    setLocalTeamOrder(newOrder);
  };

  const handleSaveTeamOrder = async () => {
    if (!session.id) return;
    setLoading(true);
    try {
      // localTeamOrder contiene los índices originales en el nuevo orden
      // Necesitamos crear un array que mapee cada posición nueva al índice original
      const newOrder = localTeamOrder;
      const updatedSession = await api.reorderTeam(session.id, newOrder);
      onUpdateSession(updatedSession);
      // Resetear el orden local después de guardar
      setLocalTeamOrder(updatedSession.team.active_members.map((_, i) => i));
      // Ajustar selectedIndex si es necesario
      if (selectedIndex !== null) {
        const newSelectedIndex = newOrder.indexOf(selectedIndex);
        setSelectedIndex(newSelectedIndex >= 0 ? newSelectedIndex : null);
      }
    } catch (error) {
      console.error('Failed to reorder team:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleMoveMoveUp = (moveIndex: number) => {
    if (selectedIndex === null) return;
    const currentOrder = localMoveOrders.get(selectedIndex) || [];
    if (moveIndex === 0) return;
    const newOrder = [...currentOrder];
    [newOrder[moveIndex - 1], newOrder[moveIndex]] = [newOrder[moveIndex], newOrder[moveIndex - 1]];
    setLocalMoveOrders(new Map(localMoveOrders.set(selectedIndex, newOrder)));
  };

  const handleMoveMoveDown = (moveIndex: number) => {
    if (selectedIndex === null) return;
    const currentOrder = localMoveOrders.get(selectedIndex) || [];
    if (moveIndex === currentOrder.length - 1) return;
    const newOrder = [...currentOrder];
    [newOrder[moveIndex], newOrder[moveIndex + 1]] = [newOrder[moveIndex + 1], newOrder[moveIndex]];
    setLocalMoveOrders(new Map(localMoveOrders.set(selectedIndex, newOrder)));
  };

  const handleSaveMoveOrder = async () => {
    if (selectedIndex === null || !session.id) return;
    setLoading(true);
    try {
      const moveOrder = localMoveOrders.get(selectedIndex) || [];
      const updatedSession = await api.reorderMoves(session.id, selectedIndex, moveOrder);
      onUpdateSession(updatedSession);
      // Actualizar el orden local
      const newMoveOrder = updatedSession.team.active_members[selectedIndex].randomized_profile.moves.map((_, i) => i);
      setLocalMoveOrders(new Map(localMoveOrders.set(selectedIndex, newMoveOrder)));
    } catch (error) {
      console.error('Failed to reorder moves:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleEvolve = async (targetSpeciesId: string) => {
    if (selectedIndex === null || !session.id) return;
    setLoading(true);
    setEvolutionMessage(null);
    try {
      const updatedSession = await api.evolvePokemon(session.id, selectedIndex, targetSpeciesId);
      onUpdateSession(updatedSession);
      const evolvedPokemon = updatedSession.team.active_members[selectedIndex];
      setEvolutionMessage(`¡${evolvedPokemon.species.display_name} ha evolucionado!`);
      setTimeout(() => setEvolutionMessage(null), 3000);
    } catch (error) {
      console.error('Failed to evolve pokemon:', error);
      setEvolutionMessage('Error al evolucionar');
      setTimeout(() => setEvolutionMessage(null), 3000);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div
      style={{
        width: '100%',
        height: '100vh',
        backgroundColor: '#f0f0f0',
        display: 'flex',
        flexDirection: 'column',
        padding: '20px',
        fontFamily: 'monospace',
      }}
    >
      {/* Header */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: '20px',
        }}
      >
        <h1 style={{ margin: 0 }}>Gestión de Equipo</h1>
        <button
          onClick={onBack}
          style={{
            padding: '10px 20px',
            fontSize: '16px',
            backgroundColor: '#4a5568',
            color: '#ffffff',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          Volver al Mapa
        </button>
      </div>

      {/* Mensaje de evolución */}
      {evolutionMessage && (
        <div
          style={{
            padding: '12px',
            backgroundColor: '#10b981',
            color: '#ffffff',
            borderRadius: '4px',
            marginBottom: '20px',
            textAlign: 'center',
            fontWeight: 'bold',
          }}
        >
          {evolutionMessage}
        </div>
      )}

      <div style={{ display: 'flex', gap: '20px', flex: 1, overflow: 'hidden' }}>
        {/* Lista de Equipo (Izquierda) */}
        <div
          style={{
            width: '300px',
            backgroundColor: '#ffffff',
            border: '2px solid #000000',
            borderRadius: '8px',
            padding: '15px',
            overflowY: 'auto',
          }}
        >
          <h2 style={{ marginTop: 0 }}>Equipo</h2>
          {team.length === 0 ? (
            <p style={{ color: '#000000' }}>No hay Pokémon en el equipo</p>
          ) : (
            <>
              {localTeamOrder.map((originalIndex, displayIndex) => {
                const pokemon = team[originalIndex];
                const isSelected = selectedIndex === originalIndex;
                return (
                  <div
                    key={pokemon.id}
                    onClick={() => setSelectedIndex(originalIndex)}
                    style={{
                      padding: '10px',
                      marginBottom: '8px',
                      backgroundColor: isSelected ? '#e0e0e0' : '#f9f9f9',
                      border: isSelected ? '3px solid #4a5568' : '1px solid #ccc',
                      borderRadius: '4px',
                      cursor: 'pointer',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '10px',
                    }}
                  >
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '4px', flex: 1 }}>
                      <div style={{ fontWeight: 'bold', color: '#000000' }}>{pokemon.species.display_name}</div>
                      <div style={{ fontSize: '12px', color: '#000000' }}>Nv.{pokemon.level}</div>
                      <HealthBar
                        current={pokemon.current_hp}
                        max={pokemon.base_computed_stats.hp}
                      />
                    </div>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleMoveUp(displayIndex);
                        }}
                        disabled={displayIndex === 0}
                        style={{
                          padding: '4px 8px',
                          fontSize: '12px',
                          backgroundColor: displayIndex === 0 ? '#ccc' : '#4a5568',
                          color: '#ffffff',
                          border: 'none',
                          borderRadius: '2px',
                          cursor: displayIndex === 0 ? 'not-allowed' : 'pointer',
                        }}
                      >
                        ↑
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleMoveDown(displayIndex);
                        }}
                        disabled={displayIndex === localTeamOrder.length - 1}
                        style={{
                          padding: '4px 8px',
                          fontSize: '12px',
                          backgroundColor: displayIndex === localTeamOrder.length - 1 ? '#ccc' : '#4a5568',
                          color: '#ffffff',
                          border: 'none',
                          borderRadius: '2px',
                          cursor: displayIndex === localTeamOrder.length - 1 ? 'not-allowed' : 'pointer',
                        }}
                      >
                        ↓
                      </button>
                    </div>
                  </div>
                );
              })}
              <button
                onClick={handleSaveTeamOrder}
                disabled={loading}
                style={{
                  width: '100%',
                  padding: '10px',
                  marginTop: '10px',
                  backgroundColor: '#10b981',
                  color: '#ffffff',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: loading ? 'not-allowed' : 'pointer',
                  fontWeight: 'bold',
                }}
              >
                {loading ? 'Guardando...' : 'Guardar Orden'}
              </button>
            </>
          )}
        </div>

        {/* Detalle del Pokémon (Derecha) */}
        {selectedPokemon ? (
          <div
            style={{
              flex: 1,
              backgroundColor: '#ffffff',
              border: '2px solid #000000',
              borderRadius: '8px',
              padding: '20px',
              overflowY: 'auto',
              color: '#000000',
            }}
          >
            <div style={{ display: 'flex', gap: '20px', marginBottom: '20px' }}>
              {/* Sprite */}
              <div>
                <PokemonSprite speciesId={selectedPokemon.species.species_id} back={false} />
              </div>

              {/* Info Básica */}
              <div style={{ flex: 1, color: '#000000' }}>
                <h2 style={{ marginTop: 0, color: '#000000' }}>{selectedPokemon.species.display_name}</h2>
                <div style={{ color: '#000000' }}>Nivel: {selectedPokemon.level}</div>
                <div style={{ marginTop: '10px' }}>
                  <HealthBar
                    current={selectedPokemon.current_hp}
                    max={selectedPokemon.base_computed_stats.hp}
                    showText={true}
                  />
                </div>
              </div>
            </div>

            {/* Stats */}
            <div style={{ marginBottom: '20px', color: '#000000' }}>
              <h3 style={{ color: '#000000' }}>Estadísticas</h3>
              <div
                style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(2, 1fr)',
                  gap: '10px',
                  color: '#000000',
                }}
              >
                <div style={{ color: '#000000' }}>HP: {selectedPokemon.base_computed_stats.hp}</div>
                <div style={{ color: '#000000' }}>Ataque: {selectedPokemon.base_computed_stats.attack}</div>
                <div style={{ color: '#000000' }}>Defensa: {selectedPokemon.base_computed_stats.defense}</div>
                <div style={{ color: '#000000' }}>Ataque Especial: {selectedPokemon.base_computed_stats.special_attack}</div>
                <div style={{ color: '#000000' }}>Defensa Especial: {selectedPokemon.base_computed_stats.special_defense}</div>
                <div style={{ color: '#000000' }}>Velocidad: {selectedPokemon.base_computed_stats.speed}</div>
              </div>
            </div>

            {/* Movimientos */}
            <div style={{ marginBottom: '20px', color: '#000000' }}>
              <h3 style={{ color: '#000000' }}>Movimientos</h3>
              <p style={{ 
                fontSize: '12px', 
                color: '#666666', 
                marginTop: '4px', 
                marginBottom: '12px',
                fontStyle: 'italic'
              }}>
                Solo los primeros 4 movimientos se usarán en batalla. Reordena para equipar.
              </p>
              {(() => {
                const moveOrder = localMoveOrders.get(selectedIndex!) || selectedPokemon.randomized_profile.moves.map((_, i) => i);
                const activeMoves = moveOrder.slice(0, 4);
                const reserveMoves = moveOrder.slice(4);
                
                return (
                  <>
                    {/* Movimientos Activos (0-3) */}
                    {activeMoves.length > 0 && (
                      <div style={{ marginBottom: '16px' }}>
                        <div style={{ 
                          fontSize: '12px', 
                          fontWeight: 'bold', 
                          color: '#059669',
                          marginBottom: '8px',
                          textTransform: 'uppercase',
                          letterSpacing: '0.5px'
                        }}>
                          En Combate
                        </div>
                        {activeMoves.map((moveIndex, displayIndex) => {
                          const move = selectedPokemon.randomized_profile.moves[moveIndex];
                          return (
                            <div
                              key={displayIndex}
                              style={{
                                display: 'flex',
                                alignItems: 'center',
                                gap: '10px',
                                padding: '8px',
                                marginBottom: '4px',
                                backgroundColor: '#d1fae5',
                                border: '2px solid #10b981',
                                borderRadius: '4px',
                              }}
                            >
                              <button
                                onClick={() => handleMoveMoveUp(displayIndex)}
                                disabled={displayIndex === 0}
                                style={{
                                  padding: '4px 8px',
                                  backgroundColor: displayIndex === 0 ? '#ccc' : '#4a5568',
                                  color: '#ffffff',
                                  border: 'none',
                                  borderRadius: '2px',
                                  cursor: displayIndex === 0 ? 'not-allowed' : 'pointer',
                                }}
                              >
                                ↑
                              </button>
                              <button
                                onClick={() => handleMoveMoveDown(displayIndex)}
                                disabled={displayIndex === activeMoves.length - 1 && reserveMoves.length === 0}
                                style={{
                                  padding: '4px 8px',
                                  backgroundColor: (displayIndex === activeMoves.length - 1 && reserveMoves.length === 0) ? '#ccc' : '#4a5568',
                                  color: '#ffffff',
                                  border: 'none',
                                  borderRadius: '2px',
                                  cursor: (displayIndex === activeMoves.length - 1 && reserveMoves.length === 0) ? 'not-allowed' : 'pointer',
                                }}
                              >
                                ↓
                              </button>
                              <span style={{ flex: 1, color: '#000000', fontWeight: '500' }}>{move.template_id}</span>
                            </div>
                          );
                        })}
                      </div>
                    )}
                    
                    {/* Movimientos en Reserva (4+) */}
                    {reserveMoves.length > 0 && (
                      <div style={{ marginBottom: '16px' }}>
                        <div style={{ 
                          fontSize: '12px', 
                          fontWeight: 'bold', 
                          color: '#6b7280',
                          marginBottom: '8px',
                          textTransform: 'uppercase',
                          letterSpacing: '0.5px'
                        }}>
                          Reserva
                        </div>
                        {reserveMoves.map((moveIndex, displayIndex) => {
                          const move = selectedPokemon.randomized_profile.moves[moveIndex];
                          const actualDisplayIndex = displayIndex + 4; // Ajustar índice para los botones
                          return (
                            <div
                              key={actualDisplayIndex}
                              style={{
                                display: 'flex',
                                alignItems: 'center',
                                gap: '10px',
                                padding: '8px',
                                marginBottom: '4px',
                                backgroundColor: '#f3f4f6',
                                border: '1px solid #d1d5db',
                                borderRadius: '4px',
                                opacity: 0.7,
                              }}
                            >
                              <button
                                onClick={() => handleMoveMoveUp(actualDisplayIndex)}
                                disabled={actualDisplayIndex === 0}
                                style={{
                                  padding: '4px 8px',
                                  backgroundColor: actualDisplayIndex === 0 ? '#ccc' : '#6b7280',
                                  color: '#ffffff',
                                  border: 'none',
                                  borderRadius: '2px',
                                  cursor: actualDisplayIndex === 0 ? 'not-allowed' : 'pointer',
                                }}
                              >
                                ↑
                              </button>
                              <button
                                onClick={() => handleMoveMoveDown(actualDisplayIndex)}
                                disabled={actualDisplayIndex === moveOrder.length - 1}
                                style={{
                                  padding: '4px 8px',
                                  backgroundColor: actualDisplayIndex === moveOrder.length - 1 ? '#ccc' : '#6b7280',
                                  color: '#ffffff',
                                  border: 'none',
                                  borderRadius: '2px',
                                  cursor: actualDisplayIndex === moveOrder.length - 1 ? 'not-allowed' : 'pointer',
                                }}
                              >
                                ↓
                              </button>
                              <span style={{ flex: 1, color: '#6b7280' }}>{move.template_id}</span>
                            </div>
                          );
                        })}
                      </div>
                    )}
                  </>
                );
              })()}
              <button
                onClick={handleSaveMoveOrder}
                disabled={loading}
                style={{
                  marginTop: '10px',
                  padding: '8px 16px',
                  backgroundColor: '#10b981',
                  color: '#ffffff',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: loading ? 'not-allowed' : 'pointer',
                }}
              >
                Guardar Orden de Movimientos
              </button>
            </div>

            {/* Evoluciones */}
            <div style={{ color: '#000000' }}>
              <h3 style={{ color: '#000000' }}>Evoluciones</h3>
              {selectedPokemon.species.evolutions.length === 0 ? (
                <p style={{ color: '#000000' }}>Este Pokémon no tiene evoluciones disponibles</p>
              ) : (
                selectedPokemon.species.evolutions.map((evolution, index) => {
                  const canEvolve = evolution.min_level === null || selectedPokemon.level >= evolution.min_level;
                  return (
                    <div
                      key={index}
                      style={{
                        padding: '12px',
                        marginBottom: '8px',
                        backgroundColor: canEvolve ? '#f0fdf4' : '#f9fafb',
                        border: canEvolve ? '2px solid #10b981' : '1px solid #ccc',
                        borderRadius: '4px',
                      }}
                    >
                      {canEvolve ? (
                        <div>
                          <div style={{ fontWeight: 'bold', marginBottom: '8px', color: '#000000' }}>
                            ¡EVOLUCIONAR A {evolution.target_species_id.toUpperCase()}!
                          </div>
                          <button
                            onClick={() => handleEvolve(evolution.target_species_id)}
                            disabled={loading}
                            style={{
                              padding: '10px 20px',
                              backgroundColor: '#10b981',
                              color: '#ffffff',
                              border: 'none',
                              borderRadius: '4px',
                              cursor: loading ? 'not-allowed' : 'pointer',
                              fontWeight: 'bold',
                              fontSize: '14px',
                            }}
                          >
                            ¡EVOLUCIONAR!
                          </button>
                        </div>
                      ) : (
                        <div style={{ color: '#6b7280' }}>
                          Evoluciona a <strong style={{ color: '#6b7280' }}>{evolution.target_species_id}</strong> al nivel{' '}
                          {evolution.min_level}
                        </div>
                      )}
                    </div>
                  );
                })
              )}
            </div>
          </div>
        ) : (
          <div
            style={{
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: '#6b7280',
            }}
          >
            Selecciona un Pokémon para ver sus detalles
          </div>
        )}
      </div>
    </div>
  );
}

