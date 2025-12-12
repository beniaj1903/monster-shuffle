import { PokemonInstance } from '../types';

interface EncounterSelectionProps {
  optionsCount: number;
  onSelectEncounter: (index: number) => void;
  selectedPokemon: PokemonInstance | null;
  loading: boolean;
}

export function EncounterSelection({
  optionsCount,
  onSelectEncounter,
  selectedPokemon,
  loading,
}: EncounterSelectionProps) {
  return (
    <div className="encounter-selection">
      <h2>¡Un Pokémon salvaje apareció!</h2>
      <p>Elige una opción (a ciegas):</p>
      <div className="encounters-grid">
        {Array.from({ length: optionsCount }).map((_, index) => (
          <button
            key={index}
            className="encounter-button"
            onClick={() => onSelectEncounter(index)}
            disabled={loading}
          >
            ?
          </button>
        ))}
      </div>
      {selectedPokemon && (
        <div className="selected-pokemon">
          <h3>¡Has capturado a {selectedPokemon.species.display_name}!</h3>
          <p>Nivel {selectedPokemon.level}</p>
          <p>Tipo: {selectedPokemon.species.primary_type}</p>
          <p>HP: {selectedPokemon.current_hp}/{selectedPokemon.base_computed_stats.hp}</p>
        </div>
      )}
    </div>
  );
}

