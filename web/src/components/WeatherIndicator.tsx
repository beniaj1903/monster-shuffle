import { WeatherState } from '../types';

interface WeatherIndicatorProps {
  weather: WeatherState | null;
}

export function WeatherIndicator({ weather }: WeatherIndicatorProps) {
  if (!weather) {
    return null;
  }

  const getWeatherConfig = () => {
    switch (weather.weather_type) {
      case 'Sun':
        return {
          icon: '☀️',
          name: 'Sol',
          bgColor: 'bg-orange-100',
          borderColor: 'border-orange-400',
          textColor: 'text-orange-800',
        };
      case 'Rain':
        return {
          icon: '💧',
          name: 'Lluvia',
          bgColor: 'bg-blue-100',
          borderColor: 'border-blue-400',
          textColor: 'text-blue-800',
        };
      case 'Sandstorm':
        return {
          icon: '🌪️',
          name: 'Tormenta de Arena',
          bgColor: 'bg-stone-100',
          borderColor: 'border-stone-400',
          textColor: 'text-stone-800',
        };
      case 'Hail':
        return {
          icon: '❄️',
          name: 'Granizo',
          bgColor: 'bg-cyan-50',
          borderColor: 'border-cyan-300',
          textColor: 'text-cyan-800',
        };
      default:
        return null;
    }
  };

  const config = getWeatherConfig();
  if (!config) {
    return null;
  }

  return (
    <div
      className={`absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 
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
          {weather.turns_remaining} turno{weather.turns_remaining !== 1 ? 's' : ''} restante{weather.turns_remaining !== 1 ? 's' : ''}
        </span>
      </div>
    </div>
  );
}

