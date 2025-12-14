import { useState } from 'react';

export type BattleFormat = 'Single' | 'Double';

export interface GameConfig {
  generations: number[];
  gym_interval: number;
  total_encounters: number;
  chaos_move_randomizer: boolean;
  preferred_format: BattleFormat;
}

interface NewGameFormProps {
  onSubmit: (config: GameConfig) => void;
  loading: boolean;
}

export function NewGameForm({ onSubmit, loading }: NewGameFormProps) {
  const [generations, setGenerations] = useState<number[]>([1, 2, 3, 4, 5, 6, 7, 8, 9]);
  const [gymInterval, setGymInterval] = useState(5);
  const [totalEncounters, setTotalEncounters] = useState(20);
  const [chaosMode, setChaosMode] = useState(false);
  const [preferredFormat, setPreferredFormat] = useState<BattleFormat>('Single');

  const allGenerations = [1, 2, 3, 4, 5, 6, 7, 8, 9];

  const toggleGeneration = (gen: number) => {
    if (generations.includes(gen)) {
      setGenerations(generations.filter((g) => g !== gen));
    } else {
      setGenerations([...generations, gen].sort((a, b) => a - b));
    }
  };

  const selectAllGenerations = () => {
    setGenerations([...allGenerations]);
  };

  const deselectAllGenerations = () => {
    setGenerations([]);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (generations.length === 0) {
      alert('Debes seleccionar al menos una generación');
      return;
    }
    onSubmit({
      generations,
      gym_interval: gymInterval,
      total_encounters: totalEncounters,
      chaos_move_randomizer: chaosMode,
      preferred_format: preferredFormat,
    });
  };

  return (
    <div
      style={{
        minHeight: '100vh',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
        padding: '20px',
        fontFamily: 'monospace',
      }}
    >
      <div
        style={{
          backgroundColor: '#ffffff',
          borderRadius: '16px',
          padding: '40px',
          maxWidth: '600px',
          width: '100%',
          boxShadow: '0 20px 60px rgba(0, 0, 0, 0.3)',
        }}
      >
        <h1
          style={{
            textAlign: 'center',
            marginBottom: '30px',
            fontSize: '32px',
            fontWeight: 'bold',
            color: '#1a1a1a',
            textShadow: '2px 2px 4px rgba(0, 0, 0, 0.1)',
          }}
        >
          ⚡ Configuración de Partida ⚡
        </h1>

        <form onSubmit={handleSubmit}>
          {/* Selector de Generaciones */}
          <div style={{ marginBottom: '30px' }}>
            <label
              style={{
                display: 'block',
                marginBottom: '12px',
                fontSize: '18px',
                fontWeight: 'bold',
                color: '#1a1a1a',
              }}
            >
              Generaciones Pokémon
            </label>
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(3, 1fr)',
                gap: '10px',
                marginBottom: '12px',
              }}
            >
              {allGenerations.map((gen) => (
                <button
                  key={gen}
                  type="button"
                  onClick={() => toggleGeneration(gen)}
                  disabled={loading}
                  style={{
                    padding: '12px',
                    fontSize: '16px',
                    fontWeight: 'bold',
                    border: '2px solid',
                    borderRadius: '8px',
                    cursor: loading ? 'not-allowed' : 'pointer',
                    transition: 'all 0.2s',
                    backgroundColor: generations.includes(gen) ? '#4CAF50' : '#f5f5f5',
                    borderColor: generations.includes(gen) ? '#45a049' : '#ddd',
                    color: generations.includes(gen) ? '#ffffff' : '#333',
                    opacity: loading ? 0.6 : 1,
                  }}
                  onMouseEnter={(e) => {
                    if (!loading && !generations.includes(gen)) {
                      e.currentTarget.style.backgroundColor = '#e0e0e0';
                    }
                  }}
                  onMouseLeave={(e) => {
                    if (!loading && !generations.includes(gen)) {
                      e.currentTarget.style.backgroundColor = '#f5f5f5';
                    }
                  }}
                >
                  Gen {gen}
                </button>
              ))}
            </div>
            <div style={{ display: 'flex', gap: '10px', justifyContent: 'center' }}>
              <button
                type="button"
                onClick={selectAllGenerations}
                disabled={loading}
                style={{
                  padding: '8px 16px',
                  fontSize: '14px',
                  border: '1px solid #4CAF50',
                  borderRadius: '6px',
                  backgroundColor: '#e8f5e9',
                  color: '#2e7d32',
                  cursor: loading ? 'not-allowed' : 'pointer',
                  opacity: loading ? 0.6 : 1,
                }}
              >
                Seleccionar Todas
              </button>
              <button
                type="button"
                onClick={deselectAllGenerations}
                disabled={loading}
                style={{
                  padding: '8px 16px',
                  fontSize: '14px',
                  border: '1px solid #f44336',
                  borderRadius: '6px',
                  backgroundColor: '#ffebee',
                  color: '#c62828',
                  cursor: loading ? 'not-allowed' : 'pointer',
                  opacity: loading ? 0.6 : 1,
                }}
              >
                Deseleccionar Todas
              </button>
            </div>
            {generations.length === 0 && (
              <p style={{ color: '#f44336', fontSize: '14px', marginTop: '8px', textAlign: 'center' }}>
                ⚠️ Debes seleccionar al menos una generación
              </p>
            )}
          </div>

          {/* Slider de Intervalo de Gym */}
          <div style={{ marginBottom: '30px' }}>
            <label
              style={{
                display: 'block',
                marginBottom: '12px',
                fontSize: '18px',
                fontWeight: 'bold',
                color: '#1a1a1a',
              }}
            >
              Intervalo de Gimnasio: {gymInterval}
            </label>
            <input
              type="range"
              min="3"
              max="10"
              value={gymInterval}
              onChange={(e) => setGymInterval(Number(e.target.value))}
              disabled={loading}
              style={{
                width: '100%',
                height: '8px',
                borderRadius: '4px',
                outline: 'none',
                opacity: loading ? 0.6 : 1,
              }}
            />
            <p style={{ fontSize: '14px', color: '#666', marginTop: '8px', fontStyle: 'italic' }}>
              Cada {gymInterval} combates habrá un Líder de Gimnasio
            </p>
          </div>

          {/* Slider de Duración de Partida */}
          <div style={{ marginBottom: '30px' }}>
            <label
              style={{
                display: 'block',
                marginBottom: '12px',
                fontSize: '18px',
                fontWeight: 'bold',
                color: '#1a1a1a',
              }}
            >
              Duración de Partida: {totalEncounters} encuentros
            </label>
            <input
              type="range"
              min="10"
              max="100"
              value={totalEncounters}
              onChange={(e) => setTotalEncounters(Number(e.target.value))}
              disabled={loading}
              style={{
                width: '100%',
                height: '8px',
                borderRadius: '4px',
                outline: 'none',
                opacity: loading ? 0.6 : 1,
              }}
            />
            <p style={{ fontSize: '14px', color: '#666', marginTop: '8px', fontStyle: 'italic' }}>
              Total de encuentros necesarios para completar la partida
            </p>
          </div>

          {/* Selector de Formato de Liga */}
          <div style={{ marginBottom: '30px' }}>
            <label
              style={{
                display: 'block',
                marginBottom: '12px',
                fontSize: '18px',
                fontWeight: 'bold',
                color: '#1a1a1a',
              }}
            >
              Formato de Liga (Gyms)
            </label>
            <div
              style={{
                display: 'flex',
                gap: '10px',
              }}
            >
              <button
                type="button"
                onClick={() => setPreferredFormat('Single')}
                disabled={loading}
                style={{
                  flex: 1,
                  padding: '16px',
                  fontSize: '16px',
                  fontWeight: 'bold',
                  border: '2px solid',
                  borderRadius: '8px',
                  cursor: loading ? 'not-allowed' : 'pointer',
                  transition: 'all 0.2s',
                  backgroundColor: preferredFormat === 'Single' ? '#4CAF50' : '#f5f5f5',
                  borderColor: preferredFormat === 'Single' ? '#45a049' : '#ddd',
                  color: preferredFormat === 'Single' ? '#ffffff' : '#333',
                  opacity: loading ? 0.6 : 1,
                }}
                onMouseEnter={(e) => {
                  if (!loading && preferredFormat !== 'Single') {
                    e.currentTarget.style.backgroundColor = '#e0e0e0';
                  }
                }}
                onMouseLeave={(e) => {
                  if (!loading && preferredFormat !== 'Single') {
                    e.currentTarget.style.backgroundColor = '#f5f5f5';
                  }
                }}
              >
                Individuales (1v1)
              </button>
              <button
                type="button"
                onClick={() => setPreferredFormat('Double')}
                disabled={loading}
                style={{
                  flex: 1,
                  padding: '16px',
                  fontSize: '16px',
                  fontWeight: 'bold',
                  border: '2px solid',
                  borderRadius: '8px',
                  cursor: loading ? 'not-allowed' : 'pointer',
                  transition: 'all 0.2s',
                  backgroundColor: preferredFormat === 'Double' ? '#2196F3' : '#f5f5f5',
                  borderColor: preferredFormat === 'Double' ? '#1976D2' : '#ddd',
                  color: preferredFormat === 'Double' ? '#ffffff' : '#333',
                  opacity: loading ? 0.6 : 1,
                }}
                onMouseEnter={(e) => {
                  if (!loading && preferredFormat !== 'Double') {
                    e.currentTarget.style.backgroundColor = '#e0e0e0';
                  }
                }}
                onMouseLeave={(e) => {
                  if (!loading && preferredFormat !== 'Double') {
                    e.currentTarget.style.backgroundColor = '#f5f5f5';
                  }
                }}
              >
                Dobles (VGC)
              </button>
            </div>
            <p style={{ fontSize: '14px', color: '#666', marginTop: '8px', fontStyle: 'italic' }}>
              Nota: Los encuentros salvajes siempre son Individuales (1v1)
            </p>
          </div>

          {/* Modo Caos */}
          <div
            style={{
              marginBottom: '30px',
              padding: '20px',
              backgroundColor: chaosMode ? '#fff3cd' : '#f5f5f5',
              border: `2px solid ${chaosMode ? '#ffc107' : '#ddd'}`,
              borderRadius: '12px',
              transition: 'all 0.3s',
            }}
          >
            <label
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '12px',
                cursor: loading ? 'not-allowed' : 'pointer',
                opacity: loading ? 0.6 : 1,
              }}
            >
              <input
                type="checkbox"
                checked={chaosMode}
                onChange={(e) => setChaosMode(e.target.checked)}
                disabled={loading}
                style={{
                  width: '24px',
                  height: '24px',
                  cursor: loading ? 'not-allowed' : 'pointer',
                }}
              />
              <div>
                <div
                  style={{
                    fontSize: '20px',
                    fontWeight: 'bold',
                    color: chaosMode ? '#ff9800' : '#1a1a1a',
                    marginBottom: '4px',
                  }}
                >
                  ⚠️ Modo Caos
                </div>
                <div style={{ fontSize: '14px', color: '#666' }}>
                  ¡Los Pokémon aprenden cualquier ataque del juego al azar!
                </div>
              </div>
            </label>
          </div>

          {/* Botón de Acción */}
          <button
            type="submit"
            disabled={loading || generations.length === 0}
            style={{
              width: '100%',
              padding: '16px',
              fontSize: '20px',
              fontWeight: 'bold',
              backgroundColor: generations.length === 0 || loading ? '#ccc' : '#4CAF50',
              color: '#ffffff',
              border: 'none',
              borderRadius: '8px',
              cursor: generations.length === 0 || loading ? 'not-allowed' : 'pointer',
              transition: 'all 0.2s',
              boxShadow: generations.length === 0 || loading
                ? 'none'
                : '0 4px 12px rgba(76, 175, 80, 0.4)',
            }}
            onMouseEnter={(e) => {
              if (generations.length > 0 && !loading) {
                e.currentTarget.style.backgroundColor = '#45a049';
                e.currentTarget.style.transform = 'translateY(-2px)';
                e.currentTarget.style.boxShadow = '0 6px 16px rgba(76, 175, 80, 0.5)';
              }
            }}
            onMouseLeave={(e) => {
              if (generations.length > 0 && !loading) {
                e.currentTarget.style.backgroundColor = '#4CAF50';
                e.currentTarget.style.transform = 'translateY(0)';
                e.currentTarget.style.boxShadow = '0 4px 12px rgba(76, 175, 80, 0.4)';
              }
            }}
          >
            {loading ? 'Cargando...' : '🚀 Comenzar Aventura'}
          </button>
        </form>
      </div>
    </div>
  );
}

