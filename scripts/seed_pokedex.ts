import axios from 'axios';
import fs from 'fs/promises';
import * as path from 'path';

// Tipos de la PokéAPI
interface PokeApiGeneration {
  pokemon_species: Array<{
    name: string;
    url: string;
  }>;
}

interface PokeApiPokemonSpecies {
  id: number;
  name: string;
  evolution_chain: {
    url: string;
  };
  varieties: Array<{
    is_default: boolean;
    pokemon: {
      name: string;
      url: string;
    };
  }>;
}

interface EvolutionDetail {
  min_level: number | null;
  trigger: {
    name: string;
    url: string;
  };
  item: {
    name: string;
    url: string;
  } | null;
  [key: string]: any; // Para otros campos que puedan existir
}

interface EvolutionChainLink {
  species: {
    name: string;
    url: string;
  };
  evolution_details: EvolutionDetail[];
  evolves_to: EvolutionChainLink[];
}

interface PokeApiEvolutionChain {
  id: number;
  chain: EvolutionChainLink;
}

interface PokeApiPokemon {
  id: number;
  name: string;
  types: Array<{
    slot: number;
    type: {
      name: string;
    };
  }>;
  stats: Array<{
    base_stat: number;
    stat: {
      name: string;
    };
  }>;
  moves: Array<{
    move: {
      name: string;
      url: string;
    };
    version_group_details: Array<{
      level_learned_at: number;
      move_learn_method: {
        name: string;
      };
      version_group: {
        name: string;
      };
    }>;
  }>;
}

interface PokeApiMoveDetail {
  name: string;
}

// Tipo de salida compatible con PokemonSpecies de Rust
interface EvolutionDataOutput {
  target_species_id: string;
  min_level: number | null;
  trigger: string;
}

interface PokemonSpeciesOutput {
  species_id: string;
  display_name: string;
  generation: number;
  primary_type: string;
  secondary_type: string | null;
  base_stats: {
    hp: number;
    attack: number;
    defense: number;
    special_attack: number;
    special_defense: number;
    speed: number;
  };
  move_pool: string[];
  is_starter_candidate: boolean;
  evolutions: EvolutionDataOutput[];
}

// Mapeo de tipos de la API al enum de Rust
const TYPE_MAP: Record<string, string> = {
  normal: 'Normal',
  fire: 'Fire',
  water: 'Water',
  grass: 'Grass',
  electric: 'Electric',
  ice: 'Ice',
  fighting: 'Fighting',
  poison: 'Poison',
  ground: 'Ground',
  flying: 'Flying',
  psychic: 'Psychic',
  bug: 'Bug',
  rock: 'Rock',
  ghost: 'Ghost',
  dragon: 'Dragon',
  dark: 'Dark',
  steel: 'Steel',
  fairy: 'Fairy',
};

// Mapeo de nombres de stats de la API a nuestro formato
const STAT_MAP: Record<string, keyof PokemonSpeciesOutput['base_stats']> = {
  hp: 'hp',
  attack: 'attack',
  defense: 'defense',
  'special-attack': 'special_attack',
  'special-defense': 'special_defense',
  speed: 'speed',
};

// Función para capitalizar la primera letra
function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

// Función para mapear el tipo de la API al enum de Rust
function mapType(apiType: string): string {
  const normalized = apiType.toLowerCase();
  return TYPE_MAP[normalized] || 'Unknown';
}

// Función para mapear los stats de la API a nuestro formato
function mapStats(apiStats: PokeApiPokemon['stats']): PokemonSpeciesOutput['base_stats'] {
  const stats: PokemonSpeciesOutput['base_stats'] = {
    hp: 0,
    attack: 0,
    defense: 0,
    special_attack: 0,
    special_defense: 0,
    speed: 0,
  };

  for (const stat of apiStats) {
    const statName = stat.stat.name;
    const mappedKey = STAT_MAP[statName];
    if (mappedKey) {
      switch (mappedKey) {
        case 'hp':
          stats.hp = stat.base_stat;
          break;
        case 'attack':
          stats.attack = stat.base_stat;
          break;
        case 'defense':
          stats.defense = stat.base_stat;
          break;
        case 'special_attack':
          stats.special_attack = stat.base_stat;
          break;
        case 'special_defense':
          stats.special_defense = stat.base_stat;
          break;
        case 'speed':
          stats.speed = stat.base_stat;
          break;
      }
    }
  }

  return stats;
}

// Función para extraer movimientos aprendidos por level-up
function extractLevelUpMoves(moves: PokeApiPokemon['moves']): string[] {
  const levelUpMoves = new Set<string>();

  for (const moveData of moves) {
    for (const detail of moveData.version_group_details) {
      if (detail.move_learn_method.name === 'level-up') {
        // Los nombres de movimientos ya vienen en formato kebab-case de la API
        levelUpMoves.add(moveData.move.name);
      }
    }
  }

  return Array.from(levelUpMoves).sort();
}

// Cache de profundidades de cadenas evolutivas (URL -> profundidad máxima)
const evolutionChainCache = new Map<string, number>();

// Cache de cadenas evolutivas completas (URL -> EvolutionChain)
const evolutionChainDataCache = new Map<string, PokeApiEvolutionChain>();

// Función para calcular la profundidad máxima de una cadena evolutiva
function calculateMaxDepth(chain: EvolutionChainLink, currentDepth: number = 1): number {
  if (chain.evolves_to.length === 0) {
    return currentDepth;
  }
  
  let maxDepth = currentDepth;
  for (const evolution of chain.evolves_to) {
    const depth = calculateMaxDepth(evolution, currentDepth + 1);
    maxDepth = Math.max(maxDepth, depth);
  }
  
  return maxDepth;
}

// Función para obtener la cadena evolutiva completa (con cache)
async function getEvolutionChain(
  evolutionChainUrl: string
): Promise<PokeApiEvolutionChain | null> {
  // Verificar cache
  if (evolutionChainDataCache.has(evolutionChainUrl)) {
    return evolutionChainDataCache.get(evolutionChainUrl)!;
  }

  try {
    const response = await axios.get<PokeApiEvolutionChain>(evolutionChainUrl, {
      timeout: 10000,
    });

    const evolutionChain = response.data;

    // Guardar en cache
    evolutionChainDataCache.set(evolutionChainUrl, evolutionChain);
    
    // También cachear la profundidad
    const maxDepth = calculateMaxDepth(evolutionChain.chain, 1);
    evolutionChainCache.set(evolutionChainUrl, maxDepth);

    return evolutionChain;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      console.error(`Error fetching evolution chain ${evolutionChainUrl}:`, error.message);
    } else {
      console.error(`Error fetching evolution chain ${evolutionChainUrl}:`, error);
    }
    return null;
  }
}

// Función para obtener la profundidad de una cadena evolutiva (con cache)
async function getEvolutionChainDepth(
  evolutionChainUrl: string
): Promise<number> {
  // Verificar cache
  if (evolutionChainCache.has(evolutionChainUrl)) {
    return evolutionChainCache.get(evolutionChainUrl)!;
  }

  // Si no está en cache, obtener la cadena completa (que también cachea la profundidad)
  const chain = await getEvolutionChain(evolutionChainUrl);
  if (chain) {
    return evolutionChainCache.get(evolutionChainUrl)!;
  }

  // En caso de error, retornar profundidad 1
  return 1;
}

// Función para buscar un nodo en el árbol evolutivo por nombre de especie
function findSpeciesNode(
  chain: EvolutionChainLink,
  speciesName: string
): EvolutionChainLink | null {
  // Normalizar nombres para comparación
  const normalizedTarget = speciesName.toLowerCase();
  const normalizedCurrent = chain.species.name.toLowerCase();

  if (normalizedCurrent === normalizedTarget) {
    return chain;
  }

  // Buscar recursivamente en las evoluciones
  for (const evolution of chain.evolves_to) {
    const found = findSpeciesNode(evolution, speciesName);
    if (found) {
      return found;
    }
  }

  return null;
}

// Función para extraer las evoluciones de un nodo
function extractEvolutions(node: EvolutionChainLink): EvolutionDataOutput[] {
  const evolutions: EvolutionDataOutput[] = [];

  for (const evolution of node.evolves_to) {
    // Obtener el ID de la especie objetivo (extraer del URL o usar el nombre)
    // El URL tiene formato: "https://pokeapi.co/api/v2/pokemon-species/{id}/"
    const speciesUrl = evolution.species.url;
    const speciesIdMatch = speciesUrl.match(/pokemon-species\/(\d+)/);
    const targetSpeciesId = speciesIdMatch
      ? speciesIdMatch[1].padStart(3, '0')
      : evolution.species.name;

    // Extraer detalles de evolución (puede haber múltiples, tomamos el primero)
    const evolutionDetail = evolution.evolution_details[0] || null;

    const evolutionData: EvolutionDataOutput = {
      target_species_id: targetSpeciesId,
      min_level: evolutionDetail?.min_level ?? null,
      trigger: evolutionDetail?.trigger?.name ?? 'unknown',
    };

    evolutions.push(evolutionData);
  }

  return evolutions;
}

// Función para hacer sleep (rate limiting)
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Función para resolver el nombre canónico del Pokémon y obtener info de especie
// Retorna tanto el nombre resuelto como la información completa de la especie
async function resolveDefaultPokemonNameAndInfo(
  speciesName: string
): Promise<{ resolvedName: string; speciesInfo: PokeApiPokemonSpecies | null }> {
  try {
    const response = await axios.get<PokeApiPokemonSpecies>(
      `https://pokeapi.co/api/v2/pokemon-species/${speciesName}`,
      {
        timeout: 10000,
      }
    );

    const species = response.data;
    
    // Buscar la variedad por defecto
    const defaultVariety = species.varieties.find(v => v.is_default);
    
    let resolvedName: string;
    if (defaultVariety) {
      // Extraer el nombre del URL o usar directamente
      // El nombre viene en formato "deoxys-normal", "giratina-origin", etc.
      resolvedName = defaultVariety.pokemon.name;
    } else if (species.varieties.length > 0) {
      // Si no hay variedad por defecto, usar el primer elemento
      resolvedName = species.varieties[0].pokemon.name;
    } else {
      // Fallback: devolver el nombre original
      resolvedName = speciesName;
    }
    
    return { resolvedName, speciesInfo: species };
  } catch (error) {
    if (axios.isAxiosError(error)) {
      console.error(`Error resolving species ${speciesName}:`, error.message);
    } else {
      console.error(`Error resolving species ${speciesName}:`, error);
    }
    // Fallback: devolver el nombre original y null para speciesInfo
    return { resolvedName: speciesName, speciesInfo: null };
  }
}

// Función para obtener datos de un Pokémon
async function fetchPokemonData(
  pokemonName: string,
  speciesName: string,
  generation: number,
  speciesInfo: PokeApiPokemonSpecies | null
): Promise<PokemonSpeciesOutput | null> {
  try {
    const response = await axios.get<PokeApiPokemon>(
      `https://pokeapi.co/api/v2/pokemon/${pokemonName}`,
      {
        timeout: 10000,
      }
    );

    const pokemon = response.data;

    // Mapear tipos
    const types = pokemon.types.sort((a, b) => a.slot - b.slot);
    const primaryType = mapType(types[0].type.name);
    const secondaryType = types.length > 1 ? mapType(types[1].type.name) : null;

    // Mapear stats
    const baseStats = mapStats(pokemon.stats);

    // Extraer movimientos
    const movePool = extractLevelUpMoves(pokemon.moves);

    // Extraer información de evolución
    let isStarterCandidate = false;
    let evolutions: EvolutionDataOutput[] = [];

    if (speciesInfo?.evolution_chain?.url) {
      // Obtener la cadena evolutiva completa (usa cache)
      const evolutionChain = await getEvolutionChain(speciesInfo.evolution_chain.url);

      if (evolutionChain) {
        // Calcular profundidad máxima
        const maxDepth = calculateMaxDepth(evolutionChain.chain, 1);

        // Verificar si es candidato a starter (base de cadena con profundidad >= 3)
        const baseSpeciesName = evolutionChain.chain.species.name.toLowerCase();
        isStarterCandidate = maxDepth >= 3 && baseSpeciesName === speciesName.toLowerCase();

        // Buscar el nodo de esta especie en el árbol evolutivo
        const speciesNode = findSpeciesNode(evolutionChain.chain, speciesName);

        if (speciesNode) {
          // Extraer las evoluciones de este nodo
          evolutions = extractEvolutions(speciesNode);
        }
      }
    }

    // Crear el objeto de salida
    // Usamos speciesName para el display_name (nombre de la especie, no la forma)
    const output: PokemonSpeciesOutput = {
      species_id: pokemon.id.toString().padStart(3, '0'),
      display_name: capitalize(speciesName),
      generation,
      primary_type: primaryType,
      secondary_type: secondaryType,
      base_stats: baseStats,
      move_pool: movePool,
      is_starter_candidate: isStarterCandidate,
      evolutions: evolutions,
    };

    return output;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      console.error(`Error fetching ${pokemonName} (species: ${speciesName}):`, error.message);
    } else {
      console.error(`Error fetching ${pokemonName} (species: ${speciesName}):`, error);
    }
    return null;
  }
}

// Función principal
async function main() {
  // Leer argumentos de línea de comandos
  const args = process.argv.slice(2);
  const generationArg = args.find(arg => arg.startsWith('--gen=') || arg.startsWith('-g='));
  
  let generations: number[];
  if (generationArg) {
    const genNum = parseInt(generationArg.split('=')[1]);
    if (isNaN(genNum) || genNum < 1 || genNum > 9) {
      console.error('❌ Error: La generación debe ser un número entre 1 y 9');
      console.log('Uso: npm run seed -- --gen=1');
      console.log('   o: npm run seed -- -g=1');
      process.exit(1);
    }
    generations = [genNum];
    console.log(`🎯 Procesando solo Generación ${genNum}...\n`);
  } else {
    generations = [1, 2, 3, 4, 5, 6, 7, 8, 9];
    console.log('🚀 Iniciando proceso ETL de PokéAPI (todas las generaciones)...\n');
  }

  const allSpecies: PokemonSpeciesOutput[] = [];
  const outputPath = path.join(process.cwd(), '..', 'server', 'data', 'pokedex.json');

  for (const gen of generations) {
    console.log(`📦 Procesando Generación ${gen}...`);

    try {
      // Obtener lista de especies de la generación
      const genResponse = await axios.get<PokeApiGeneration>(
        `https://pokeapi.co/api/v2/generation/${gen}`,
        { timeout: 10000 }
      );

      const speciesList = genResponse.data.pokemon_species;
      console.log(`   Encontradas ${speciesList.length} especies`);

      let successCount = 0;
      let errorCount = 0;

      // Procesar cada especie
      for (let i = 0; i < speciesList.length; i++) {
        const species = speciesList[i];
        
        // Resolver el nombre canónico y obtener información completa de la especie
        const { resolvedName, speciesInfo } = await resolveDefaultPokemonNameAndInfo(species.name);
        
        // Rate limiting: esperar 75ms después de obtener info de especie
        await sleep(75);
        
        // Obtener los datos del Pokémon usando el nombre resuelto y la info de especie
        const pokemonData = await fetchPokemonData(resolvedName, species.name, gen, speciesInfo);

        if (pokemonData) {
          allSpecies.push(pokemonData);
          successCount++;
        } else {
          errorCount++;
        }

        // Rate limiting: esperar 75ms entre peticiones
        if (i < speciesList.length - 1) {
          await sleep(75);
        }

        // Log de progreso cada 10 Pokémon
        if ((i + 1) % 10 === 0) {
          process.stdout.write(
            `   Progreso: ${i + 1}/${speciesList.length} procesados\r`
          );
        }
      }

      console.log(
        `   ✅ Gen ${gen} completada: ${successCount} exitosos, ${errorCount} errores\n`
      );
    } catch (error) {
      if (axios.isAxiosError(error)) {
        console.error(`❌ Error al procesar Generación ${gen}:`, error.message);
      } else {
        console.error(`❌ Error al procesar Generación ${gen}:`, error);
      }
    }
  }

  // Ordenar por species_id (número)
  allSpecies.sort((a, b) => parseInt(a.species_id) - parseInt(b.species_id));

  // Asegurar que el directorio existe
  await fs.mkdir(path.dirname(outputPath), { recursive: true });

  // Si solo se procesó una generación, leer el archivo existente y fusionar
  if (generations.length === 1) {
    try {
      let existingSpecies: PokemonSpeciesOutput[] = [];
      
      // Intentar leer el archivo existente
      try {
        const fileContent = await fs.readFile(outputPath, 'utf-8');
        existingSpecies = JSON.parse(fileContent);
      } catch {
        // Si el archivo no existe o hay error al leer, usar array vacío
        existingSpecies = [];
      }

      // Filtrar especies de la generación que estamos procesando
      const filtered = existingSpecies.filter(s => s.generation !== generations[0]);
      
      // Combinar con los nuevos datos
      const merged = [...filtered, ...allSpecies].sort(
        (a, b) => parseInt(a.species_id) - parseInt(b.species_id)
      );
      
      await fs.writeFile(outputPath, JSON.stringify(merged, null, 2), 'utf-8');
      console.log(`\n✨ Proceso completado!`);
      console.log(`📄 Archivo actualizado: ${outputPath}`);
      console.log(`📊 Pokémon procesados en esta ejecución: ${allSpecies.length}`);
      console.log(`📊 Total de Pokémon en el archivo: ${merged.length}`);
    } catch (error) {
      // Si hay error al leer, simplemente crear/sobrescribir con los nuevos datos
      await fs.writeFile(outputPath, JSON.stringify(allSpecies, null, 2), 'utf-8');
      console.log(`\n✨ Proceso completado!`);
      console.log(`📄 Archivo generado: ${outputPath}`);
      console.log(`📊 Total de Pokémon procesados: ${allSpecies.length}`);
    }
  } else {
    // Guardar el archivo JSON (modo completo)
    await fs.writeFile(outputPath, JSON.stringify(allSpecies, null, 2), 'utf-8');
    console.log(`\n✨ Proceso completado!`);
    console.log(`📄 Archivo generado: ${outputPath}`);
    console.log(`📊 Total de Pokémon procesados: ${allSpecies.length}`);
  }
}

// Ejecutar
main().catch((error) => {
  console.error('💥 Error fatal:', error);
  process.exit(1);
});

