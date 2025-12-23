# Plan de Adaptación: Frontend Battle Screen para VGC Completo

## Objetivo
Adaptar la pantalla de batalla del frontend para mostrar TODAS las mecánicas VGC implementadas en el servidor (Fases 1-4), incluyendo estados volátiles avanzados, protecciones, redirección y Trick Room.

---

## Análisis de Estado Actual

### ✅ Lo que YA muestra el Frontend
- HP (HealthBar con colores dinámicos)
- Status Conditions básicos (BRN, FRZ, PAR, PSN, SLP)
- Battle Stages (modificadores de stats -6 a +6)
- Held Items (sprites de PokeAPI)
- Abilities (nombre formateado)
- Weather (overlay con emoji e indicador)
- Terrain (overlay con emoji e indicador)

### ❌ Lo que FALTA mostrar
**20+ mecánicas completamente implementadas en el servidor pero invisibles en frontend:**

1. **Volatile Status Avanzado** (Fase 1.3):
   - Infatuation (Attract) - 50% chance no atacar
   - Leech Seed - daño 1/8 HP + curación
   - Substitute - HP del substitute absorbiendo daño
   - Perish Song - contador de turnos hasta KO
   - Confusion - estado confuso
   - Must Recharge - debe recargar (Hyper Beam)
   - Charging Move - movimiento en carga (Solar Beam)
   - Bad Poison escalante - contador de turnos

2. **Protecciones Avanzadas** (Fase 2.1):
   - Wide Guard - protege de spread moves
   - Quick Guard - protege de priority moves
   - Mat Block - protege en turno 1
   - Crafty Shield - protege de status moves
   - Protect básico - indicador visual

3. **Redirection System** (Fase 1.2):
   - Follow Me activo
   - Rage Powder activo
   - Spotlight activo
   - Indicador visual de quién redirige

4. **Trick Room** (Fase 1.4):
   - Estado activo/inactivo
   - Turnos restantes (máx 5)

---

## Estrategia de Implementación

### Fase A: Actualizar Tipos TypeScript
**Archivo:** `web/src/types.ts`

Agregar interfaces faltantes para que el frontend reconozca los datos del servidor:

```typescript
// 1. Nueva interface VolatileStatus (completa)
export interface VolatileStatus {
  // Básicos
  flinched: boolean;
  confused: boolean;
  crit_stage: number;
  protected: boolean;
  protect_counter: number;
  must_recharge: boolean;
  charging_move: string | null;
  badly_poisoned_turns: number;

  // Avanzados (Fase 1.3)
  infatuated_by: string | null;
  leech_seeded: boolean;
  leech_seed_source: string | null;
  substitute_hp: number;
  perish_count: number | null;

  // Protecciones (Fase 2.1)
  wide_guard_active: boolean;
  quick_guard_active: boolean;
  mat_block_active: boolean;
  crafty_shield_active: boolean;

  // Otros
  forced_switch: boolean;
}

// 2. Nueva interface RedirectionState
export interface RedirectionState {
  redirector_position: FieldPosition;
  redirection_type: string; // "follow-me" | "rage-powder" | "spotlight"
  opponent_only: boolean;
}

// 3. Actualizar PokemonInstance
export interface PokemonInstance {
  // ... campos existentes ...
  volatile_status: VolatileStatus | null; // ← NUEVO
}

// 4. Actualizar BattleState
export interface BattleState {
  // ... campos existentes ...
  redirection: RedirectionState | null; // ← NUEVO
  trick_room_active: boolean; // ← NUEVO
  trick_room_turns_left: number; // ← NUEVO
}
```

**Justificación:** El servidor ya envía estos datos en el JSON, pero el frontend los ignora porque no están tipificados.

---

### Fase B: Crear Componentes de Indicadores

Seguir los patrones existentes de WeatherIndicator y StatusBadge.

#### B.1: VolatileStatusIndicators.tsx
**Ubicación:** `web/src/components/VolatileStatusIndicators.tsx`

**Propósito:** Mostrar estados volátiles avanzados de forma compacta

**Diseño:**
- Row de badges pequeños (similar a StatModifiers)
- Usar emojis Unicode para iconos
- Tooltips nativos con `title` attribute
- Tailwind classes para consistencia

**Estados a mostrar:**
- ❤️ Infatuated (si `infatuated_by` presente)
- 🌿 Leech Seed (si `leech_seeded`)
- 📦 Substitute (si `substitute_hp > 0`, mostrar HP)
- ☠️ Perish Song (si `perish_count` presente, mostrar contador)
- 😵 Confused (si `confused`)
- 💤 Recharging (si `must_recharge`)
- ⚡ Charging (si `charging_move` presente)
- 🤢 Bad Poison (si `badly_poisoned_turns > 0`, mostrar turnos)

**Estructura:**
```typescript
interface VolatileStatusIndicatorsProps {
  volatile: VolatileStatus;
}

export function VolatileStatusIndicators({ volatile }: VolatileStatusIndicatorsProps) {
  const indicators = [];

  if (volatile.infatuated_by) {
    indicators.push({ emoji: '❤️', label: 'Attract', color: 'pink' });
  }

  if (volatile.leech_seeded) {
    indicators.push({ emoji: '🌿', label: 'Leech Seed', color: 'green' });
  }

  // ... más estados ...

  return (
    <div className="flex flex-wrap gap-1 mt-1">
      {indicators.map(ind => (
        <span
          key={ind.label}
          className={`px-1.5 py-0.5 rounded text-xs border ${ind.color}`}
          title={ind.label}
        >
          {ind.emoji}
        </span>
      ))}
    </div>
  );
}
```

---

#### B.2: ProtectionIndicators.tsx
**Ubicación:** `web/src/components/ProtectionIndicators.tsx`

**Propósito:** Mostrar protecciones activas (Wide Guard, Quick Guard, etc.)

**Diseño:**
- Badges con escudos como iconos
- Solo se muestran si están activos (este turno)
- Colores distintivos por tipo de protección

**Estados a mostrar:**
- 🛡️ Protected (si `protected`)
- 🛡️↔️ Wide Guard (si `wide_guard_active`)
- 🛡️⚡ Quick Guard (si `quick_guard_active`)
- 🛡️1️⃣ Mat Block (si `mat_block_active`)
- 🛡️✨ Crafty Shield (si `crafty_shield_active`)

**Estructura similar a VolatileStatusIndicators pero con colores de protección (azul/cyan)**

---

#### B.3: RedirectionIndicator.tsx
**Ubicación:** `web/src/components/RedirectionIndicator.tsx`

**Propósito:** Mostrar visualmente que hay redirección activa (Follow Me, Rage Powder, Spotlight)

**Diseño:**
- Overlay similar a WeatherIndicator/TerrainIndicator
- Position absolute en esquina superior derecha
- Muestra tipo de redirección y posición del redirector

**Estructura:**
```typescript
interface RedirectionIndicatorProps {
  redirection: RedirectionState;
}

export function RedirectionIndicator({ redirection }: RedirectionIndicatorProps) {
  const getConfig = () => {
    switch (redirection.redirection_type) {
      case 'follow-me':
        return { icon: '👋', name: 'Follow Me', color: 'bg-blue-100 border-blue-400' };
      case 'rage-powder':
        return { icon: '🍄', name: 'Rage Powder', color: 'bg-green-100 border-green-400' };
      case 'spotlight':
        return { icon: '💡', name: 'Spotlight', color: 'bg-yellow-100 border-yellow-400' };
    }
  };

  const config = getConfig();

  return (
    <div className={`absolute top-4 right-4 ${config.color} border-2 rounded-lg px-3 py-2 shadow-lg z-10`}>
      <span className="text-2xl">{config.icon}</span>
      <span className="text-sm font-bold ml-2">{config.name}</span>
      <div className="text-xs">→ {redirection.redirector_position}</div>
    </div>
  );
}
```

---

#### B.4: TrickRoomIndicator.tsx
**Ubicación:** `web/src/components/TrickRoomIndicator.tsx`

**Propósito:** Mostrar estado de Trick Room (invierte orden de velocidad)

**Diseño:**
- Overlay similar a WeatherIndicator
- Position absolute centrado arriba o debajo de Weather
- Muestra turnos restantes

**Estructura:**
```typescript
interface TrickRoomIndicatorProps {
  active: boolean;
  turnsLeft: number;
}

export function TrickRoomIndicator({ active, turnsLeft }: TrickRoomIndicatorProps) {
  if (!active) return null;

  return (
    <div className="absolute top-1/2 left-1/2 transform -translate-x-1/2 translate-y-12
                    bg-purple-100 border-purple-400 border-2 rounded-lg px-4 py-2 shadow-lg z-10">
      <span className="text-2xl">🔄</span>
      <span className="text-sm font-bold ml-2">Trick Room</span>
      <span className="text-xs ml-2">({turnsLeft} turns)</span>
    </div>
  );
}
```

---

### Fase C: Integrar en BattleScreen.tsx

**Archivo:** `web/src/components/BattleScreen.tsx`

#### C.1: Importar nuevos componentes
```typescript
import { VolatileStatusIndicators } from './VolatileStatusIndicators';
import { ProtectionIndicators } from './ProtectionIndicators';
import { RedirectionIndicator } from './RedirectionIndicator';
import { TrickRoomIndicator } from './TrickRoomIndicator';
```

#### C.2: Actualizar renderPokemonSlot()
Después de `<StatModifiers stages={pokemon.battle_stages} />` (línea ~287), agregar:

```typescript
{/* Volatile Status Row */}
{pokemon.volatile_status && (
  <VolatileStatusIndicators volatile={pokemon.volatile_status} />
)}

{/* Protection Indicators */}
{pokemon.volatile_status && (
  <ProtectionIndicators volatile={pokemon.volatile_status} />
)}
```

#### C.3: Agregar indicadores globales
Después de los indicadores de Weather y Terrain (línea ~307), agregar:

```typescript
{/* Redirection Indicator */}
{session.battle?.redirection && (
  <RedirectionIndicator redirection={session.battle.redirection} />
)}

{/* Trick Room Indicator */}
{session.battle && (
  <TrickRoomIndicator
    active={session.battle.trick_room_active || false}
    turnsLeft={session.battle.trick_room_turns_left || 0}
  />
)}
```

---

## Decisiones de Diseño

### Iconografía
**Estrategia:** Emojis Unicode (siguiendo patrón de WeatherIndicator/TerrainIndicator)

| Estado | Emoji | Justificación |
|--------|-------|---------------|
| Infatuated | ❤️ | Corazón - enamoramiento |
| Leech Seed | 🌿 | Planta - seed |
| Substitute | 📦 | Caja - muñeco substituto |
| Perish Song | ☠️ | Calavera - muerte inminente |
| Confused | 😵 | Confundido |
| Recharging | 💤 | Durmiendo - recargando |
| Charging | ⚡ | Rayo - cargando energía |
| Bad Poison | 🤢 | Enfermo - veneno |
| Protected | 🛡️ | Escudo básico |
| Wide Guard | 🛡️↔️ | Escudo + flechas (área) |
| Quick Guard | 🛡️⚡ | Escudo + rayo (velocidad) |
| Mat Block | 🛡️1️⃣ | Escudo + 1 (solo turno 1) |
| Crafty Shield | 🛡️✨ | Escudo + brillos (mágico) |
| Follow Me | 👋 | Mano saludando |
| Rage Powder | 🍄 | Hongo - spores |
| Spotlight | 💡 | Luz - atención |
| Trick Room | 🔄 | Flechas circulares - inversión |

### Colores Tailwind
Seguir paleta existente:
- **Protecciones:** `blue-*` (azul)
- **Estados negativos:** `red-*` (rojo)
- **Estados neutros:** `gray-*` (gris)
- **Redirección:** `green-*` / `yellow-*` según tipo
- **Trick Room:** `purple-*` (púrpura)

### Tooltips
Usar atributo HTML nativo `title` (siguiendo patrón de ItemIcon):
```typescript
<span title="Infatuated - 50% chance to not attack">❤️</span>
```

### Posicionamiento
- **Indicadores de Pokémon:** Dentro del info box, como rows adicionales
- **Indicadores globales:** Position absolute con z-index 10
  - Weather: centro pantalla
  - Terrain: centro pantalla + offset
  - Redirection: esquina superior derecha
  - Trick Room: debajo de Weather

---

## Archivos a Modificar/Crear

### Nuevos Archivos (4)
1. `web/src/components/VolatileStatusIndicators.tsx` (~80 líneas)
2. `web/src/components/ProtectionIndicators.tsx` (~60 líneas)
3. `web/src/components/RedirectionIndicator.tsx` (~50 líneas)
4. `web/src/components/TrickRoomIndicator.tsx` (~40 líneas)

### Archivos a Modificar (2)
1. `web/src/types.ts` - Agregar interfaces (VolatileStatus, RedirectionState, actualizar BattleState y PokemonInstance)
2. `web/src/components/BattleScreen.tsx` - Integrar nuevos componentes en renderizado

---

## Estimación de Complejidad

**Complejidad:** Baja-Media
**Líneas de código:** ~300-350 líneas nuevas
**Riesgo:** Bajo (no modifica lógica de batalla, solo visualización)

**Ventajas:**
- Reutiliza patrones existentes 100%
- No requiere cambios al backend
- Componentes pequeños y desacoplados
- Fácil de probar y debuggear

---

## Orden de Implementación Recomendado

1. **Actualizar types.ts** (base para todo lo demás)
2. **VolatileStatusIndicators.tsx** (más común, mayor visibilidad)
3. **ProtectionIndicators.tsx** (similar al anterior)
4. **TrickRoomIndicator.tsx** (más simple, overlay global)
5. **RedirectionIndicator.tsx** (overlay global)
6. **Integrar en BattleScreen.tsx** (prueba end-to-end)

---

## Testing Manual

Después de implementar, verificar:
- ✅ Infatuation muestra ❤️ cuando Attract está activo
- ✅ Leech Seed muestra 🌿
- ✅ Perish Song muestra contador ☠️ 3 → 2 → 1
- ✅ Substitute muestra HP restante 📦
- ✅ Wide Guard muestra 🛡️↔️ durante el turno
- ✅ Follow Me muestra overlay 👋 con posición
- ✅ Trick Room muestra 🔄 con turnos restantes
- ✅ Todos los estados se ocultan cuando no están activos
- ✅ Layout no se rompe en Single vs Double battles
- ✅ Info boxes no se desbordan con muchos indicadores

---

## Notas Finales

Este plan permite visualizar **100% de las mecánicas VGC** implementadas en el servidor, cerrando el gap visual entre backend y frontend. El diseño es consistente con los patrones existentes y escalable para futuras mecánicas.
