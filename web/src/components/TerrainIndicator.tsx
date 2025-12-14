import { TerrainState } from '../types';

interface TerrainIndicatorProps {
  terrain: TerrainState | null;
}

export function TerrainIndicator({ terrain }: TerrainIndicatorProps) {
  if (!terrain) {
    return null;
  }

  const getTerrainConfig = () => {
    switch (terrain.terrain_type) {
      case 'Electric':
        return {
          icon: '⚡',
          name: 'Campo Eléctrico',
          bgColor: 'bg-yellow-200',
          borderColor: 'border-yellow-500',
          textColor: 'text-yellow-900',
        };
      case 'Grassy':
        return {
          icon: '🌿',
          name: 'Campo de Hierba',
          bgColor: 'bg-green-200',
          borderColor: 'border-green-500',
          textColor: 'text-green-900',
        };
      case 'Misty':
        return {
          icon: '🌫️',
          name: 'Campo de Niebla',
          bgColor: 'bg-pink-200',
          borderColor: 'border-pink-500',
          textColor: 'text-pink-900',
        };
      case 'Psychic':
        return {
          icon: '🔮',
          name: 'Campo Psíquico',
          bgColor: 'bg-purple-200',
          borderColor: 'border-purple-500',
          textColor: 'text-purple-900',
        };
      default:
        return null;
    }
  };

  const config = getTerrainConfig();
  if (!config) {
    return null;
  }

  return (
    <div
      className={`absolute top-1/2 left-1/2 transform -translate-x-1/2 translate-y-8 
        ${config.bgColor} ${config.borderColor} border-2 rounded-lg px-4 py-2 
        shadow-lg z-10 flex items-center gap-2`}
      style={{
        minWidth: '180px',
      }}
    >
      <span className="text-2xl">{config.icon}</span>
      <div className="flex flex-col">
        <span className={`font-bold text-sm ${config.textColor}`}>
          {config.name}
        </span>
        <span className={`text-xs ${config.textColor} opacity-75`}>
          {terrain.turns_remaining} turno{terrain.turns_remaining !== 1 ? 's' : ''} restante{terrain.turns_remaining !== 1 ? 's' : ''}
        </span>
      </div>
    </div>
  );
}

