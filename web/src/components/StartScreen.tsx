interface StartScreenProps {
  onNewGame: () => void;
  loading: boolean;
}

export function StartScreen({ onNewGame, loading }: StartScreenProps) {
  return (
    <div className="start-screen">
      <h1>PokeRandomWeb</h1>
      <button onClick={onNewGame} disabled={loading}>
        {loading ? 'Cargando...' : 'Nuevo Juego'}
      </button>
    </div>
  );
}

