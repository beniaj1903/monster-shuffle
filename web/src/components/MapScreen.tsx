import { PokemonInstance } from '../types';

interface MapScreenProps {
  team: PokemonInstance[];
  onExplore: () => void;
  onManageTeam: () => void;
  loading: boolean;
}

export function MapScreen({ team, onExplore, onManageTeam, loading }: MapScreenProps) {
  return (
    <div className="map-screen">
      <h2>Mapa</h2>
      <div className="team-section">
        <h3>Tu Equipo</h3>
        {team.length === 0 ? (
          <p>No tienes Pokémon en tu equipo</p>
        ) : (
          <ul className="team-list">
            {team.map((pokemon) => (
              <li key={pokemon.id} className="team-member">
                <strong>{pokemon.species.display_name}</strong> - Nivel {pokemon.level} - HP:{' '}
                {pokemon.current_hp}/{pokemon.base_computed_stats.hp}
              </li>
            ))}
          </ul>
        )}
      </div>
      <div style={{ display: 'flex', gap: '10px' }}>
        <button onClick={onExplore} disabled={loading}>
          {loading ? 'Explorando...' : 'Explorar Hierba Alta'}
        </button>
        <button onClick={onManageTeam} disabled={loading}>
          Gestionar Equipo
        </button>
      </div>
    </div>
  );
}

