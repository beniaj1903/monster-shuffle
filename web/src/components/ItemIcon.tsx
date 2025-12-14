interface ItemIconProps {
  itemId: string;
}

// Helper para formatear el nombre del objeto
// Convierte "leftovers" -> "Leftovers", "life-orb" -> "Life Orb"
function formatItemName(itemId: string): string {
  return itemId
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

export function ItemIcon({ itemId }: ItemIconProps) {
  const spriteUrl = `https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/items/${itemId}.png`;
  const itemName = formatItemName(itemId);

  return (
    <div
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        width: '32px',
        height: '32px',
        backgroundColor: '#f3f4f6',
        border: '1px solid #d1d5db',
        borderRadius: '4px',
        padding: '2px',
      }}
      title={itemName}
    >
      <img
        src={spriteUrl}
        alt={itemName}
        style={{
          width: '28px',
          height: '28px',
          objectFit: 'contain',
        }}
        onError={(e) => {
          // Si la imagen falla, ocultar el componente
          e.currentTarget.style.display = 'none';
        }}
      />
    </div>
  );
}

