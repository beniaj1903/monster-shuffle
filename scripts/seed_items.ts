import axios from 'axios';
import fs from 'fs/promises';
import * as path from 'path';

// Lista de IDs de objetos competitivos
const ITEM_IDS = [
  'leftovers',
  'life-orb',
  'choice-band',
  'choice-specs',
  'choice-scarf',
  'sitrus-berry',
  'rocky-helmet',
  'focus-sash',
];

// Tipo para la respuesta de la PokéAPI
interface PokeApiItem {
  id: number;
  name: string;
  sprites: {
    default: string; // URL del sprite
  };
  effect_entries: Array<{
    effect: string;
    short_effect: string;
    language: {
      name: string;
    };
  }>;
}

// Tipo de salida para nuestro JSON
interface ItemDataOutput {
  id: string;
  name: string;
  sprite_url: string;
  effect_text: string;
}

// Función para capitalizar la primera letra
function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

// Función para obtener datos de un objeto
async function fetchItemData(itemId: string): Promise<ItemDataOutput | null> {
  try {
    const response = await axios.get<PokeApiItem>(
      `https://pokeapi.co/api/v2/item/${itemId}`,
      {
        timeout: 10000,
      }
    );

    const item = response.data;

    // Buscar el efecto en inglés
    const englishEffect = item.effect_entries.find(
      (entry) => entry.language.name === 'en'
    );

    // Obtener el texto del efecto (preferir short_effect, fallback a effect)
    const effectText = englishEffect?.short_effect || englishEffect?.effect || 'No description available.';

    // Construir la URL del sprite
    // La API devuelve URLs como "https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/items/{name}.png"
    const spriteUrl = item.sprites.default || 
      `https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/items/${item.name}.png`;

    const output: ItemDataOutput = {
      id: item.name, // El ID es el nombre en kebab-case
      name: capitalize(item.name.replace(/-/g, ' ')), // Capitalizar y reemplazar guiones
      sprite_url: spriteUrl,
      effect_text: effectText,
    };

    return output;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      console.error(`Error fetching item ${itemId}:`, error.message);
    } else {
      console.error(`Error fetching item ${itemId}:`, error);
    }
    return null;
  }
}

// Función para hacer sleep (rate limiting)
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Función principal
async function main() {
  console.log('🚀 Iniciando proceso ETL de Items de PokéAPI...\n');

  const allItems: ItemDataOutput[] = [];
  const outputPath = path.join(process.cwd(), '..', 'server', 'data', 'items.json');

  let successCount = 0;
  let errorCount = 0;

  // Procesar cada item
  for (let i = 0; i < ITEM_IDS.length; i++) {
    const itemId = ITEM_IDS[i];
    console.log(`📦 Procesando ${itemId}...`);

    const itemData = await fetchItemData(itemId);

    if (itemData) {
      allItems.push(itemData);
      successCount++;
      console.log(`   ✅ ${itemData.name} procesado`);
    } else {
      errorCount++;
      console.log(`   ❌ Error al procesar ${itemId}`);
    }

    // Rate limiting: esperar 100ms entre peticiones
    if (i < ITEM_IDS.length - 1) {
      await sleep(100);
    }
  }

  // Ordenar por ID (nombre)
  allItems.sort((a, b) => a.id.localeCompare(b.id));

  // Asegurar que el directorio existe
  await fs.mkdir(path.dirname(outputPath), { recursive: true });

  // Guardar el archivo JSON
  await fs.writeFile(outputPath, JSON.stringify(allItems, null, 2), 'utf-8');

  console.log(`\n✨ Proceso completado!`);
  console.log(`📄 Archivo generado: ${outputPath}`);
  console.log(`📊 Total de items procesados: ${allItems.length}`);
  console.log(`✅ Exitosos: ${successCount}`);
  console.log(`❌ Errores: ${errorCount}`);
}

// Ejecutar
main().catch((error) => {
  console.error('💥 Error fatal:', error);
  process.exit(1);
});

