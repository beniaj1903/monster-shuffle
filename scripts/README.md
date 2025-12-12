# Scripts ETL para PokeRandomWeb

Este directorio contiene scripts para poblar la base de datos con datos de la PokéAPI.

## Instalación

```bash
cd scripts
npm install
```

## Uso

### Generar Pokedex desde PokéAPI

**Procesar todas las generaciones (1-9):**
```bash
npm run seed
```

O directamente con tsx:
```bash
npx tsx seed_pokedex.ts
```

**Procesar una generación específica:**
```bash
npm run seed -- --gen=1
# o usando la forma corta
npm run seed -- -g=1
```

O directamente con tsx:
```bash
npx tsx seed_pokedex.ts --gen=1
npx tsx seed_pokedex.ts -g=1
```

### Características

Este script:
- Descarga datos de las generaciones 1-9 desde la PokéAPI (o una generación específica)
- Transforma los datos al formato compatible con `PokemonSpecies` de Rust
- Genera un archivo JSON en `../server/data/pokedex.json`
- Si se procesa una generación específica, fusiona los datos con el archivo existente (reemplazando esa generación)

**Nota:** El proceso puede tardar varios minutos debido al rate limiting (50ms entre peticiones) para evitar ser baneado por la API. Procesar una generación a la vez es útil para reanudar o actualizar datos específicos.


