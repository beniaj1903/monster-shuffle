import { exec } from 'child_process';
import { promisify } from 'util';
import * as path from 'path';
import { fileURLToPath } from 'url';

const execAsync = promisify(exec);

// Obtener el directorio actual en ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Colores para la consola
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  blue: '\x1b[34m',
  yellow: '\x1b[33m',
  cyan: '\x1b[36m',
};

// Scripts a ejecutar en orden
const scripts = [
  { name: 'seed_pokedex', file: 'seed_pokedex.ts', emoji: '🔵' },
  { name: 'seed_moves', file: 'seed_moves.ts', emoji: '🟢' },
  { name: 'seed_items', file: 'seed_items.ts', emoji: '🟡' },
];

/**
 * Ejecuta un script de seed y muestra su salida
 */
async function runSeedScript(script: { name: string; file: string; emoji: string }): Promise<void> {
  const scriptPath = path.join(__dirname, script.file);
  
  console.log(`${script.emoji} Ejecutando ${script.name}...`);
  console.time(`${script.emoji} ${script.name}`);
  
  try {
    // Ejecutar el script usando tsx
    const { stdout, stderr } = await execAsync(`tsx "${scriptPath}"`, {
      cwd: __dirname,
      maxBuffer: 10 * 1024 * 1024, // 10MB buffer para salida grande
    });

    // Mostrar la salida del script
    if (stdout) {
      console.log(stdout);
    }
    
    if (stderr) {
      // Algunos scripts pueden escribir a stderr sin ser errores
      // Solo mostrar si no es un warning común
      if (!stderr.includes('ExperimentalWarning')) {
        console.log(`${colors.yellow}⚠️  ${stderr}${colors.reset}`);
      }
    }

    console.timeEnd(`${script.emoji} ${script.name}`);
    console.log(`${colors.green}✅ ${script.name} completado exitosamente${colors.reset}\n`);
  } catch (error: any) {
    console.timeEnd(`${script.emoji} ${script.name}`);
    
    // Mostrar el error en rojo
    console.error(`${colors.red}❌ Error ejecutando ${script.name}:${colors.reset}`);
    console.error(`${colors.red}${error.message}${colors.reset}`);
    
    // Si hay salida de stderr, mostrarla
    if (error.stderr) {
      console.error(`${colors.red}${error.stderr}${colors.reset}`);
    }
    
    // Si hay salida de stdout, mostrarla también (puede contener información útil)
    if (error.stdout) {
      console.error(`${colors.yellow}${error.stdout}${colors.reset}`);
    }
    
    console.error(`\n${colors.red}🛑 Deteniendo la ejecución.${colors.reset}`);
    process.exit(1);
  }
}

/**
 * Función principal que ejecuta todos los scripts en secuencia
 */
async function main(): Promise<void> {
  console.log(`${colors.cyan}═══════════════════════════════════════════════════════${colors.reset}`);
  console.log(`${colors.cyan}🚀 Iniciando proceso de seed completo${colors.reset}`);
  console.log(`${colors.cyan}═══════════════════════════════════════════════════════${colors.reset}\n`);

  const startTime = Date.now();

  try {
    for (const script of scripts) {
      await runSeedScript(script);
    }

    const totalTime = ((Date.now() - startTime) / 1000).toFixed(2);
    
    console.log(`${colors.cyan}═══════════════════════════════════════════════════════${colors.reset}`);
    console.log(`${colors.green}✨ Todos los scripts se ejecutaron exitosamente${colors.reset}`);
    console.log(`${colors.cyan}⏱️  Tiempo total: ${totalTime}s${colors.reset}`);
    console.log(`${colors.cyan}═══════════════════════════════════════════════════════${colors.reset}\n`);
  } catch (error: any) {
    // Este catch debería ser innecesario ya que runSeedScript maneja los errores,
    // pero lo dejamos como seguridad
    console.error(`${colors.red}❌ Error inesperado:${colors.reset}`);
    console.error(`${colors.red}${error.message}${colors.reset}`);
    process.exit(1);
  }
}

// Ejecutar la función principal
main().catch((error) => {
  console.error(`${colors.red}❌ Error fatal:${colors.reset}`);
  console.error(error);
  process.exit(1);
});

