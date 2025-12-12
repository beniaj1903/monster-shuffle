import { StatStages } from '../types';

interface StatModifiersProps {
  stages: StatStages | null;
}

export function StatModifiers({ stages }: StatModifiersProps) {
  if (!stages) {
    return null;
  }

  // Mapeo de nombres completos a abreviaciones
  const statLabels: Record<keyof StatStages, string> = {
    attack: 'Atk',
    defense: 'Def',
    special_attack: 'SpA',
    special_defense: 'SpD',
    speed: 'Spd',
    accuracy: 'Acc',
    evasion: 'Eva',
  };

  // Filtrar solo los stats que son diferentes de 0
  const modifiedStats = Object.entries(stages).filter(
    ([_, value]) => value !== 0
  ) as Array<[keyof StatStages, number]>;

  if (modifiedStats.length === 0) {
    return null;
  }

  return (
    <div className="flex flex-row flex-wrap gap-2">
      {modifiedStats.map(([stat, value]) => {
        const label = statLabels[stat];
        const isPositive = value > 0;
        const displayValue = value > 0 ? `+${value}` : `${value}`;
        
        return (
          <span
            key={stat}
            className={`px-2 py-0.5 rounded text-xs font-bold border ${
              isPositive
                ? 'text-green-600 bg-green-50 border-green-300'
                : 'text-red-600 bg-red-50 border-red-300'
            }`}
          >
            {label} {displayValue}
          </span>
        );
      })}
    </div>
  );
}

