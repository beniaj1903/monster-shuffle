import { FieldPosition } from '../types';

interface TargetSelectionModalProps {
  isOpen: boolean;
  moveName: string;
  availableTargets: FieldPosition[];
  onSelect: (target: FieldPosition) => void;
  onCancel: () => void;
}

export function TargetSelectionModal({
  isOpen,
  moveName,
  availableTargets,
  onSelect,
  onCancel,
}: TargetSelectionModalProps) {
  if (!isOpen) return null;

  const getTargetLabel = (pos: FieldPosition): string => {
    switch (pos) {
      case 'PlayerLeft':
        return 'Tu Pokémon Izquierdo';
      case 'PlayerRight':
        return 'Tu Pokémon Derecho';
      case 'OpponentLeft':
        return 'Oponente Izquierdo';
      case 'OpponentRight':
        return 'Oponente Derecho';
    }
  };

  const getTargetColor = (pos: FieldPosition): string => {
    switch (pos) {
      case 'PlayerLeft':
      case 'PlayerRight':
        return '#10b981'; // Verde para aliados
      case 'OpponentLeft':
      case 'OpponentRight':
        return '#ef4444'; // Rojo para oponentes
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
      onClick={onCancel}
    >
      <div
        style={{
          backgroundColor: '#ffffff',
          border: '4px solid #000000',
          borderRadius: '8px',
          padding: '24px',
          minWidth: '300px',
          maxWidth: '500px',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h3 style={{ marginTop: 0, marginBottom: '16px', textAlign: 'center' }}>
          Selecciona el objetivo para {moveName}
        </h3>
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            gap: '12px',
          }}
        >
          {availableTargets.map((target) => (
            <button
              key={target}
              onClick={() => onSelect(target)}
              style={{
                backgroundColor: getTargetColor(target),
                color: '#ffffff',
                border: '2px solid #000000',
                borderRadius: '4px',
                padding: '12px',
                fontSize: '16px',
                fontWeight: 'bold',
                cursor: 'pointer',
                transition: 'transform 0.1s',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.transform = 'scale(1.05)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.transform = 'scale(1)';
              }}
            >
              {getTargetLabel(target)}
            </button>
          ))}
          <button
            onClick={onCancel}
            style={{
              backgroundColor: '#6b7280',
              color: '#ffffff',
              border: '2px solid #000000',
              borderRadius: '4px',
              padding: '12px',
              fontSize: '16px',
              fontWeight: 'bold',
              cursor: 'pointer',
              marginTop: '8px',
            }}
          >
            Cancelar
          </button>
        </div>
      </div>
    </div>
  );
}

