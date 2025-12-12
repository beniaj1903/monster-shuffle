import { useState, useEffect } from 'react';

interface PokemonSpriteProps {
  speciesId: string | number;
  back?: boolean;
  className?: string;
}

export function PokemonSprite({ speciesId, back = false, className = '' }: PokemonSpriteProps) {
  // Convertir speciesId a número entero (elimina ceros a la izquierda)
  // Ej: "025" -> 25, "001" -> 1, "pikachu" -> NaN (se manejará en el fallback)
  const parseId = (id: string | number): number | null => {
    if (typeof id === 'number') {
      return id;
    }
    // Intentar convertir string a número
    const numId = parseInt(id, 10);
    return isNaN(numId) ? null : numId;
  };

  const id = parseId(speciesId);

  // URLs de sprites
  const animatedUrl = id !== null 
    ? `https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon/versions/generation-v/black-white/animated/${back ? 'back/' : ''}${id}.gif`
    : '';
  const staticUrl = id !== null
    ? `https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon/${back ? 'back/' : ''}${id}.png`
    : '';

  // Estado para manejar el fallback
  const [currentSrc, setCurrentSrc] = useState<string>(animatedUrl);
  const [hasError, setHasError] = useState<boolean>(false);

  // Resetear el estado cuando cambia el speciesId o back
  useEffect(() => {
    if (id !== null) {
      setCurrentSrc(animatedUrl);
      setHasError(false);
    }
  }, [id, back, animatedUrl]);

  // Manejar error de carga de imagen
  const handleError = () => {
    if (currentSrc === animatedUrl) {
      // Si falla la animada, intentar la estática
      setCurrentSrc(staticUrl);
    } else {
      // Si también falla la estática, marcar como error
      setHasError(true);
    }
  };

  // Si hay error en ambas URLs, no renderizar nada o un placeholder
  if (hasError) {
    return (
      <div 
        className={className}
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          minWidth: '96px',
          minHeight: '96px',
          backgroundColor: '#f0f0f0',
          border: '1px solid #ccc',
          borderRadius: '4px',
        }}
      >
        <span style={{ fontSize: '12px', color: '#666' }}>?</span>
      </div>
    );
  }

  return (
    <img
      src={currentSrc}
      alt={`Pokémon ${id}`}
      className={className}
      onError={handleError}
      style={{
        imageRendering: 'pixelated',
        maxWidth: '100%',
        maxHeight: '100%',
      }}
    />
  );
}

