interface GameOverModalProps {
  onRetry: () => void;
  onQuit: () => void;
  loading?: boolean;
}

export function GameOverModal({ onRetry, onQuit, loading = false }: GameOverModalProps) {
  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.85)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 9999,
        fontFamily: 'monospace',
      }}
    >
      <div
        style={{
          backgroundColor: '#1a1a1a',
          border: '4px solid #ff0000',
          borderRadius: '16px',
          padding: '40px',
          maxWidth: '500px',
          width: '90%',
          textAlign: 'center',
          boxShadow: '0 0 30px rgba(255, 0, 0, 0.5)',
        }}
      >
        {/* Título GAME OVER */}
        <h1
          style={{
            fontSize: '48px',
            fontWeight: 'bold',
            color: '#ff0000',
            marginBottom: '20px',
            textShadow: '0 0 10px rgba(255, 0, 0, 0.8), 0 0 20px rgba(255, 0, 0, 0.6)',
            letterSpacing: '4px',
            fontFamily: 'monospace',
          }}
        >
          GAME OVER
        </h1>

        {/* Mensaje */}
        <p
          style={{
            fontSize: '20px',
            color: '#ffffff',
            marginBottom: '40px',
            lineHeight: '1.6',
          }}
        >
          Tu equipo ha sido debilitado.
        </p>

        {/* Botones de Acción */}
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            gap: '16px',
          }}
        >
          {/* Botón Reintentar */}
          <button
            onClick={onRetry}
            disabled={loading}
            style={{
              padding: '16px 32px',
              fontSize: '18px',
              fontWeight: 'bold',
              backgroundColor: loading ? '#666' : '#4CAF50',
              color: '#ffffff',
              border: 'none',
              borderRadius: '8px',
              cursor: loading ? 'not-allowed' : 'pointer',
              transition: 'all 0.2s',
              boxShadow: loading
                ? 'none'
                : '0 4px 12px rgba(76, 175, 80, 0.4)',
            }}
            onMouseEnter={(e) => {
              if (!loading) {
                e.currentTarget.style.backgroundColor = '#45a049';
                e.currentTarget.style.transform = 'translateY(-2px)';
                e.currentTarget.style.boxShadow = '0 6px 16px rgba(76, 175, 80, 0.5)';
              }
            }}
            onMouseLeave={(e) => {
              if (!loading) {
                e.currentTarget.style.backgroundColor = '#4CAF50';
                e.currentTarget.style.transform = 'translateY(0)';
                e.currentTarget.style.boxShadow = '0 4px 12px rgba(76, 175, 80, 0.4)';
              }
            }}
          >
            {loading ? 'Cargando...' : '🔄 Reintentar Misión'}
          </button>

          {/* Botón Volver al Título */}
          <button
            onClick={onQuit}
            disabled={loading}
            style={{
              padding: '16px 32px',
              fontSize: '18px',
              fontWeight: 'bold',
              backgroundColor: loading ? '#666' : '#666666',
              color: '#ffffff',
              border: 'none',
              borderRadius: '8px',
              cursor: loading ? 'not-allowed' : 'pointer',
              transition: 'all 0.2s',
              boxShadow: loading
                ? 'none'
                : '0 4px 12px rgba(102, 102, 102, 0.4)',
            }}
            onMouseEnter={(e) => {
              if (!loading) {
                e.currentTarget.style.backgroundColor = '#555555';
                e.currentTarget.style.transform = 'translateY(-2px)';
                e.currentTarget.style.boxShadow = '0 6px 16px rgba(102, 102, 102, 0.5)';
              }
            }}
            onMouseLeave={(e) => {
              if (!loading) {
                e.currentTarget.style.backgroundColor = '#666666';
                e.currentTarget.style.transform = 'translateY(0)';
                e.currentTarget.style.boxShadow = '0 4px 12px rgba(102, 102, 102, 0.4)';
              }
            }}
          >
            🏠 Volver al Título
          </button>
        </div>
      </div>
    </div>
  );
}

