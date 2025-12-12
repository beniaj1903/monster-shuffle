import { GameSession } from '../types';
import { PokemonSprite } from './PokemonSprite';

interface VictoryScreenProps {
  session: GameSession;
  onRestart: () => void;
}

export function VictoryScreen({ session, onRestart }: VictoryScreenProps) {
  const teamMembers = session.team.active_members;

  return (
    <>
      <style>
        {`
          @keyframes victory-bounce {
            0%, 100% { transform: translateY(0); }
            50% { transform: translateY(-10px); }
          }
          @keyframes victory-shimmer {
            0%, 100% { opacity: 0.3; }
            50% { opacity: 0.6; }
          }
          @keyframes victory-glow {
            0%, 100% { box-shadow: 0 0 20px rgba(255, 215, 0, 0.5); }
            50% { box-shadow: 0 0 40px rgba(255, 215, 0, 0.8); }
          }
          .victory-pokemon-card {
            animation: victory-bounce 2s ease-in-out infinite;
          }
          .victory-pokemon-card:nth-child(1) { animation-delay: 0s; }
          .victory-pokemon-card:nth-child(2) { animation-delay: 0.2s; }
          .victory-pokemon-card:nth-child(3) { animation-delay: 0.4s; }
          .victory-pokemon-card:nth-child(4) { animation-delay: 0.6s; }
          .victory-pokemon-card:nth-child(5) { animation-delay: 0.8s; }
          .victory-pokemon-card:nth-child(6) { animation-delay: 1s; }
        `}
      </style>
      <div
        style={{
          minHeight: '100vh',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          background: 'linear-gradient(135deg, #f6d365 0%, #fda085 50%, #ffd700 100%)',
          padding: '40px 20px',
          fontFamily: 'monospace',
          position: 'relative',
          overflow: 'hidden',
        }}
      >
        {/* Efecto de partículas/brillo de fondo */}
        <div
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: 'radial-gradient(circle at 20% 50%, rgba(255, 255, 255, 0.3) 0%, transparent 50%), radial-gradient(circle at 80% 80%, rgba(255, 255, 255, 0.2) 0%, transparent 50%)',
            animation: 'victory-shimmer 3s ease-in-out infinite',
          }}
        />

      {/* Contenedor principal */}
      <div
        style={{
          backgroundColor: 'rgba(255, 255, 255, 0.95)',
          borderRadius: '24px',
          padding: '50px',
          maxWidth: '900px',
          width: '100%',
          boxShadow: '0 20px 60px rgba(0, 0, 0, 0.3)',
          position: 'relative',
          zIndex: 1,
          animation: 'victory-glow 2s ease-in-out infinite',
        }}
      >
        {/* Título */}
        <h1
          style={{
            fontSize: '56px',
            fontWeight: 'bold',
            textAlign: 'center',
            marginBottom: '20px',
            background: 'linear-gradient(135deg, #ffd700 0%, #ffed4e 50%, #ffd700 100%)',
            WebkitBackgroundClip: 'text',
            WebkitTextFillColor: 'transparent',
            backgroundClip: 'text',
            textShadow: '2px 2px 4px rgba(0, 0, 0, 0.2)',
            letterSpacing: '4px',
          }}
        >
          ⭐ ¡CAMPEÓN DE LA LIGA! ⭐
        </h1>

        {/* Subtítulo */}
        <p
          style={{
            fontSize: '24px',
            textAlign: 'center',
            color: '#333',
            marginBottom: '40px',
            fontWeight: 'bold',
          }}
        >
          Has completado tu aventura Pokémon
        </p>

        {/* Hall of Fame */}
        <div
          style={{
            marginBottom: '40px',
          }}
        >
          <h2
            style={{
              fontSize: '32px',
              textAlign: 'center',
              color: '#1a1a1a',
              marginBottom: '30px',
              fontWeight: 'bold',
              textDecoration: 'underline',
            }}
          >
            🏆 Hall of Fame 🏆
          </h2>

          {/* Grid de Pokémon */}
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
              gap: '20px',
              marginBottom: '30px',
            }}
          >
            {teamMembers.map((pokemon, index) => (
              <div
                key={pokemon.id || index}
                className="victory-pokemon-card"
                style={{
                  backgroundColor: '#fff',
                  border: '3px solid #ffd700',
                  borderRadius: '16px',
                  padding: '20px',
                  textAlign: 'center',
                  boxShadow: '0 8px 16px rgba(0, 0, 0, 0.2)',
                  transition: 'transform 0.2s',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = 'scale(1.1)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = 'scale(1)';
                }}
              >
                {/* Sprite del Pokémon */}
                <div
                  style={{
                    marginBottom: '12px',
                    display: 'flex',
                    justifyContent: 'center',
                    alignItems: 'center',
                    minHeight: '96px',
                  }}
                >
                  <PokemonSprite
                    speciesId={pokemon.species.species_id}
                    back={false}
                  />
                </div>

                {/* Nombre */}
                <div
                  style={{
                    fontSize: '16px',
                    fontWeight: 'bold',
                    color: '#1a1a1a',
                    marginBottom: '8px',
                  }}
                >
                  {pokemon.species.display_name}
                </div>

                {/* Nivel */}
                <div
                  style={{
                    fontSize: '14px',
                    color: '#666',
                    fontWeight: 'bold',
                  }}
                >
                  Nv. {pokemon.level}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Estadísticas */}
        <div
          style={{
            backgroundColor: '#f0f0f0',
            borderRadius: '12px',
            padding: '24px',
            marginBottom: '30px',
            textAlign: 'center',
          }}
        >
          <div
            style={{
              fontSize: '20px',
              color: '#1a1a1a',
              fontWeight: 'bold',
            }}
          >
            📊 Estadísticas Finales
          </div>
          <div
            style={{
              fontSize: '18px',
              color: '#333',
              marginTop: '12px',
            }}
          >
            Total Encuentros Ganados: <strong>{session.encounters_won}</strong>
          </div>
          <div
            style={{
              fontSize: '18px',
              color: '#333',
              marginTop: '8px',
            }}
          >
            Equipo Final: <strong>{teamMembers.length}</strong> Pokémon
          </div>
        </div>

        {/* Botón de Nueva Partida */}
        <div style={{ textAlign: 'center' }}>
          <button
            onClick={onRestart}
            style={{
              padding: '18px 48px',
              fontSize: '22px',
              fontWeight: 'bold',
              backgroundColor: '#4CAF50',
              color: '#ffffff',
              border: 'none',
              borderRadius: '12px',
              cursor: 'pointer',
              transition: 'all 0.3s',
              boxShadow: '0 6px 20px rgba(76, 175, 80, 0.4)',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = '#45a049';
              e.currentTarget.style.transform = 'translateY(-3px)';
              e.currentTarget.style.boxShadow = '0 8px 24px rgba(76, 175, 80, 0.6)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = '#4CAF50';
              e.currentTarget.style.transform = 'translateY(0)';
              e.currentTarget.style.boxShadow = '0 6px 20px rgba(76, 175, 80, 0.4)';
            }}
          >
            🎮 Jugar Nueva Partida
          </button>
        </div>
      </div>
    </div>
    </>
  );
}

