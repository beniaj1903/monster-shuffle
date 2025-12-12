import axios from 'axios';
import { promises as fs } from 'fs';
import * as path from 'path';

// Tipo para la respuesta de la PokéAPI
interface PokeApiMove {
  id: number;
  name: string;
  type: {
    name: string;
  };
  power: number | null;
  accuracy: number | null;
  pp: number;
  priority: number;
  damage_class: {
    name: string; // "physical", "special", "status"
  };
  meta: {
    ailment: {
      name: string; // "burn", "paralysis", "none", etc.
    };
    ailment_chance: number; // 0-100
    crit_rate: number; // 0-4
    drain: number; // % de daño que cura al usuario
    flinch_chance: number; // % de retroceso
    stat_chance: number; // % de que ocurran los cambios de stats
  };
  stat_changes: Array<{
    stat: {
      name: string; // "attack", "special-attack", etc.
    };
    change: number; // -6 a +6
  }>;
  target: {
    name: string; // "selected-pokemon", "users-field", "user", etc.
  };
}

// Tipo de salida para nuestro JSON
interface MoveDataOutput {
  id: string;
  name: string;
  type: string;
  power: number | null;
  accuracy: number | null;
  priority: number;
  pp: number;
  damage_class: string;
  meta: {
    ailment: string;
    ailment_chance: number;
    crit_rate: number;
    drain: number;
    flinch_chance: number;
    stat_chance: number;
  };
  stat_changes: Array<{
    stat: string;
    change: number;
  }>;
  target: string;
}

// Función para capitalizar la primera letra (para tipos)
function capitalizeFirst(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

// Función para mapear el tipo de la API a nuestro formato
function mapType(apiType: string): string {
  return capitalizeFirst(apiType);
}

// Función para mapear la categoría de daño
function mapDamageClass(apiClass: string): string {
  return apiClass; // Ya viene en el formato correcto: "physical", "special", "status"
}

// Función para mapear nombres de stats de la API a nuestro formato
// "special-attack" -> "special_attack", "attack" -> "attack", etc.
function mapStatName(apiStatName: string): string {
  return apiStatName.replace(/-/g, '_');
}

// Función para mapear el nombre del ailment
function mapAilment(apiAilment: string): string {
  // La API usa "none" para movimientos sin efecto de estado
  return apiAilment === 'none' ? 'none' : apiAilment;
}

// Función para obtener datos de un movimiento por ID
async function fetchMoveData(id: number): Promise<MoveDataOutput | null> {
  try {
    const response = await axios.get<PokeApiMove>(
      `https://pokeapi.co/api/v2/move/${id}`,
      {
        timeout: 10000,
      }
    );

    const move = response.data;

    // Mapear stat_changes
    const stat_changes = move.stat_changes.map((statChange) => ({
      stat: mapStatName(statChange.stat.name),
      change: statChange.change,
    }));

    return {
      id: move.name, // Usamos el nombre como ID (ej: "scratch")
      name: capitalizeFirst(move.name.replace(/-/g, ' ')), // "scratch" -> "Scratch"
      type: mapType(move.type.name),
      power: move.power,
      accuracy: move.accuracy,
      priority: move.priority,
      pp: move.pp,
      damage_class: mapDamageClass(move.damage_class.name),
      meta: {
        ailment: mapAilment(move.meta.ailment.name),
        ailment_chance: move.meta.ailment_chance,
        crit_rate: move.meta.crit_rate,
        drain: move.meta.drain,
        flinch_chance: move.meta.flinch_chance,
        stat_chance: move.meta.stat_chance,
      },
      stat_changes: stat_changes,
      target: move.target.name,
    };
  } catch (error) {
    if (axios.isAxiosError(error)) {
      if (error.response?.status === 404) {
        console.warn(`Move ID ${id} not found (404)`);
        return null;
      }
      console.error(`Error fetching move ID ${id}:`, error.message);
    } else {
      console.error(`Unexpected error fetching move ID ${id}:`, error);
    }
    return null;
  }
}

// Función principal
async function seedMoves() {
  const outputDir = path.join(process.cwd(), '..', 'server', 'data');
  const outputFile = path.join(outputDir, 'moves.json');

  // Asegurar que el directorio existe
  try {
    await fs.mkdir(outputDir, { recursive: true });
  } catch (error) {
    console.error('Error creating output directory:', error);
    process.exit(1);
  }

  console.log('Iniciando descarga de movimientos de PokéAPI...');
  console.log('Rango: Movimientos 1-850');

  const moves: MoveDataOutput[] = [];
  const startId = 1;
  const endId = 850;
  let successCount = 0;
  let errorCount = 0;

  // Iterar por todos los IDs de movimientos
  for (let id = startId; id <= endId; id++) {
    const moveData = await fetchMoveData(id);

    if (moveData) {
      moves.push(moveData);
      successCount++;
      if (successCount % 50 === 0) {
        console.log(`Procesados ${successCount} movimientos... (ID actual: ${id})`);
      }
    } else {
      errorCount++;
    }

    // Rate limiting: esperar 75ms entre peticiones
    if (id < endId) {
      await new Promise((resolve) => setTimeout(resolve, 75));
    }
  }

  console.log(`\nDescarga completada:`);
  console.log(`  - Movimientos exitosos: ${successCount}`);
  console.log(`  - Errores/No encontrados: ${errorCount}`);
  console.log(`  - Total procesados: ${moves.length}`);

  // Ordenar por ID (nombre) para facilitar búsquedas
  moves.sort((a, b) => a.id.localeCompare(b.id));

  // Guardar el archivo JSON
  try {
    await fs.writeFile(outputFile, JSON.stringify(moves, null, 2), 'utf-8');
    console.log(`\n✅ Movimientos guardados en: ${outputFile}`);
  } catch (error) {
    console.error('Error escribiendo el archivo:', error);
    process.exit(1);
  }
}

// Ejecutar el script
seedMoves().catch((error) => {
  console.error('Error fatal:', error);
  process.exit(1);
});

