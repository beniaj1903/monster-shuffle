interface StatusBadgeProps {
  status: string | null;
}

export function StatusBadge({ status }: StatusBadgeProps) {
  if (!status) {
    return null;
  }

  // Mapear el estado a código y clases de Tailwind
  const getStatusClasses = (status: string) => {
    // Normalizar a minúsculas para comparación
    const normalizedStatus = status.toLowerCase();
    
    switch (normalizedStatus) {
      case 'burn':
        return {
          code: 'BRN',
          classes: 'bg-red-500 text-white border-red-600',
        };
      case 'freeze':
        return {
          code: 'FRZ',
          classes: 'bg-cyan-400 text-white border-cyan-500',
        };
      case 'paralysis':
        return {
          code: 'PAR',
          classes: 'bg-yellow-400 text-black border-yellow-500',
        };
      case 'poison':
        return {
          code: 'PSN',
          classes: 'bg-purple-500 text-white border-purple-600',
        };
      case 'badpoison':
        return {
          code: 'PSN',
          classes: 'bg-purple-700 text-white border-purple-800',
        };
      case 'sleep':
        return {
          code: 'SLP',
          classes: 'bg-gray-500 text-white border-gray-600',
        };
      default:
        // Fallback para estados desconocidos
        return {
          code: status.length >= 3 ? status.substring(0, 3).toUpperCase() : status.toUpperCase(),
          classes: 'bg-gray-500 text-white border-gray-600',
        };
    }
  };

  const { code, classes } = getStatusClasses(status);

  return (
    <span
      className={`px-2 py-0.5 rounded text-xs font-bold border-2 font-mono tracking-wide shadow-sm ${classes}`}
    >
      {code}
    </span>
  );
}
