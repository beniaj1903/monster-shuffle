interface HealthBarProps {
  current: number;
  max: number;
  showText?: boolean;
}

export function HealthBar({ current, max, showText = false }: HealthBarProps) {
  // Calcular el porcentaje de HP
  const percentage = Math.max(0, Math.min(100, (current / max) * 100));

  // Determinar el color según el porcentaje
  let color: string;
  if (percentage > 50) {
    color = '#4ade80'; // Verde
  } else if (percentage > 20) {
    color = '#fbbf24'; // Amarillo
  } else {
    color = '#ef4444'; // Rojo
  }

  return (
    <div style={{ width: '100%' }}>
      <div
        style={{
          width: '100%',
          height: '20px',
          backgroundColor: '#e5e7eb',
          border: '2px solid #374151',
          borderRadius: '4px',
          overflow: 'hidden',
          position: 'relative',
        }}
      >
        <div
          style={{
            width: `${percentage}%`,
            height: '100%',
            backgroundColor: color,
            transition: 'width 0.5s ease-in-out',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          {showText && percentage > 15 && (
            <span
              style={{
                fontSize: '12px',
                fontWeight: 'bold',
                color: '#ffffff',
                textShadow: '1px 1px 2px rgba(0,0,0,0.5)',
              }}
            >
              {current}/{max}
            </span>
          )}
        </div>
      </div>
      {showText && (
        <div style={{ marginTop: '4px', fontSize: '12px', textAlign: 'center' }}>
          {current}/{max}
        </div>
      )}
    </div>
  );
}

