import { PokemonInstance } from '../types';

interface StarterSelectionProps {
  starters: PokemonInstance[];
  onChooseStarter: (index: number) => void;
  loading: boolean;
}

export function StarterSelection({
  starters,
  onChooseStarter,
  loading,
}: StarterSelectionProps) {
  return (
    <div className="starter-selection">
      <h2>Elige tu Starter</h2>
      <div className="starters-grid">
        {starters.map((starter, index) => (
          <div
            key={starter.id}
            className="starter-card"
            onClick={() => !loading && onChooseStarter(index)}
            style={{ cursor: loading ? 'not-allowed' : 'pointer', opacity: loading ? 0.6 : 1 }}
          >
            <h3>{starter.species.display_name}</h3>
            <p>Nivel {starter.level}</p>
            <p>HP: {starter.current_hp}/{starter.base_computed_stats.hp}</p>
            <p>Tipo: {starter.species.primary_type}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

