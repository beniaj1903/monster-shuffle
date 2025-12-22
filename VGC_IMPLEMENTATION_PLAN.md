# Plan de Implementación VGC - Monster Shuffle

## Estado Actual: ~35-40% de cobertura VGC completa

Basado en análisis exhaustivo del motor de batalla, este documento presenta un roadmap priorizado para alcanzar cobertura completa de mecánicas VGC oficiales.

---

## Resumen Ejecutivo

### Arquitectura Actual (Bien Implementada)
- ✅ Sistema modular por capas (systems/, orchestration/, infrastructure/)
- ✅ Damage calculation Gen 7+ correcto
- ✅ Type effectiveness completo
- ✅ Basic targeting para doubles
- ✅ Weather/Terrain base
- ✅ Status conditions principales
- ✅ Stat stages y modificadores

### Gaps Críticos Identificados
- ❌ **Sistema de Items**: 0% implementado (Choice, Life Orb, Berries, etc.)
- ❌ **Redirection**: Follow Me, Spotlight, Ally Switch
- ❌ **Volatile Status avanzado**: Confusion, Infatuation, Leech Seed, Substitute
- ❌ **Protección avanzada**: Wide Guard, Quick Guard, Mat Block
- ❌ **Trick Room**: Inversión de velocidad
- ❌ **Abilities críticas**: ~45+ habilidades faltantes
- ⚠️ **Targeting**: Redirection automática no existe
- ⚠️ **Bad Poison**: No escala correctamente

---

## FASE 1: Mecánicas Críticas (Bloquean juego competitivo)

**Duración estimada**: 2-3 semanas
**Objetivo**: Habilitar estrategias VGC básicas

### 1.1 Sistema de Held Items
**Prioridad**: 🔴 CRÍTICA

#### Arquitectura propuesta
```rust
// core/src/battle/systems/item_system/mod.rs
pub mod item_effects;
pub mod item_triggers;

pub enum ItemTrigger {
    OnDamageTaken,        // Weakness Policy, Rocky Helmet
    OnDamageDealt,        // Life Orb
    OnStatusApplied,      // Lum Berry
    OnHPThreshold,        // Sitrus Berry
    OnTurnStart,          // Choice Items
    OnMoveUsed,           // Choice Items
    OnTypeEffectiveness,  // Type berries (Occa, Chople, etc.)
}

pub enum ItemEffect {
    BoostDamage { multiplier: f32, recoil_percent: Option<f32> },  // Life Orb
    BoostStat { stat: String, multiplier: f32 },                    // Assault Vest, Eviolite
    LockMove { locked_move_id: Option<String> },                    // Choice Items
    CureStatus { conditions: Vec<StatusCondition> },                 // Lum Berry
    RestoreHP { amount: u16 },                                       // Sitrus Berry
    BoostStatsOnHit { stats: Vec<(String, i8)> },                   // Weakness Policy
    DealDamageOnContact { damage_percent: f32 },                     // Rocky Helmet
    ReduceEffectiveness { type_name: String, multiplier: f32 },     // Type berries
}
```

#### Items a implementar (Prioridad)
1. **Choice Band/Specs/Scarf** (x1.5 Atk/SpA, x1.5 Spe, lock move)
   - Modificar `execute_single_action` para trackear last_move
   - Validar que solo pueda usar ese movimiento

2. **Life Orb** (x1.3 daño, -10% recoil)
   - Modificar `calculate_damage` para aplicar multiplicador
   - Aplicar recoil en `execute_single_action`

3. **Assault Vest** (+50% SpDef, no puede usar status moves)
   - Modificar stat calculation
   - Validar en `can_pokemon_move`

4. **Sitrus Berry** (Cura 25% HP cuando < 50%)
   - Trigger en `apply_residual_effects`
   - Check HP threshold

5. **Lum Berry** (Cura todos los status)
   - Trigger en `apply_status_condition`

6. **Weakness Policy** (+2 Atk/SpA cuando golpeado super efectivo)
   - Trigger en `execute_single_action` después de calcular daño
   - Check type effectiveness

#### Ubicación de cambios
```
core/src/battle/systems/item_system/
├── mod.rs                    (exports + registry)
├── item_effects.rs           (lógica de efectos)
├── item_triggers.rs          (triggers + conditions)
└── item_processor.rs         (aplicación en pipeline)

core/src/battle/systems/damage_system/calculator.rs
├── apply_item_damage_multiplier()  (Life Orb, Choice Band)

core/src/battle/pipeline.rs
├── check_item_triggers()     (verificar activación)
├── apply_item_effects()      (aplicar efectos)
```

#### Tests requeridos
- Choice items bloquean movimientos después del primer uso
- Life Orb aplica recoil correctamente
- Assault Vest bloquea movimientos de estado
- Sitrus Berry activa en threshold correcto
- Lum Berry cura todos los estados
- Weakness Policy activa solo con super efectivo

---

### 1.2 Redirection (Follow Me, Ally Switch, Rage Powder)
**Prioridad**: 🔴 CRÍTICA

#### Problema actual
```rust
// targeting.rs - Línea 180
// No hay lógica de redirección automática
pub fn resolve_targets(...) -> Vec<FieldPosition> {
    // Targeting directo al objetivo seleccionado
    // Falta: verificar si alguien usó Follow Me/Spotlight
}
```

#### Solución propuesta
```rust
// En BattleState, agregar:
pub struct RedirectionState {
    pub redirector_position: Option<FieldPosition>,
    pub redirector_team: Option<bool>,  // true = player, false = opponent
    pub active_until_end_of_turn: bool,
}

// En pipeline.rs, después de execute_single_action:
if move_id == "follow-me" || move_id == "spotlight" || move_id == "rage-powder" {
    battle_state.redirection = Some(RedirectionState {
        redirector_position: Some(user_position),
        redirector_team: Some(is_player),
        active_until_end_of_turn: true,
    });
}

// En targeting.rs, antes de resolver objetivos:
pub fn resolve_targets_with_redirection(
    user_pos: FieldPosition,
    move_target_type: &str,
    selected_target: Option<FieldPosition>,
    battle_state: &BattleState,
    redirection: &Option<RedirectionState>,
) -> Vec<FieldPosition> {
    // Si hay redirection activa y el movimiento targetea oponente
    if let Some(redir) = redirection {
        if should_redirect(user_pos, selected_target, redir) {
            return vec![redir.redirector_position.unwrap()];
        }
    }
    // ... lógica normal
}
```

#### Movimientos a implementar
1. **Follow Me** - Redirige todos los ataques single-target del oponente
2. **Spotlight** - Redirige todos los ataques al objetivo marcado
3. **Rage Powder** - Como Follow Me pero no afecta Grass types
4. **Ally Switch** - Intercambia posiciones con aliado

#### Ubicación de cambios
```
core/src/game.rs (BattleState)
├── RedirectionState struct

core/src/battle/systems/move_system/targeting.rs
├── resolve_targets_with_redirection()
├── should_redirect()

core/src/battle/pipeline.rs
├── apply_redirection_moves()
├── clear_redirection_at_end_of_turn()
```

---

### 1.3 Volatile Status Avanzado
**Prioridad**: 🔴 CRÍTICA

#### Estados a implementar

**1. Confusion** (Ya existe flag pero sin lógica)
```rust
// models.rs línea 278 - YA EXISTE
pub confused: bool,

// Falta implementar en can_pokemon_move():
if pokemon.volatile_status.confused {
    logs.push(format!("{} está confundido!", pokemon.species.display_name));
    if rng.gen_bool(0.5) {  // 50% chance
        logs.push(format!("{} se golpeó a sí mismo en confusión!", pokemon.species.display_name));
        // Aplicar daño a sí mismo (40 power, tipo físico)
        let self_damage = calculate_confusion_damage(pokemon);
        pokemon.current_hp = pokemon.current_hp.saturating_sub(self_damage);
        return (false, logs);
    }
}
```

**2. Infatuation** (No existe)
```rust
// Agregar a VolatileStatus:
pub infatuated_by: Option<FieldPosition>,

// En can_pokemon_move():
if let Some(source) = pokemon.volatile_status.infatuated_by {
    if rng.gen_bool(0.5) {  // 50% chance
        logs.push(format!("{} está enamorado y no puede atacar!", pokemon.species.display_name));
        return (false, logs);
    }
}
```

**3. Leech Seed** (No existe)
```rust
// Agregar a VolatileStatus:
pub leech_seeded: bool,
pub leech_seed_source: Option<FieldPosition>,

// En process_end_of_turn_residuals():
if pokemon.volatile_status.leech_seeded {
    let damage = pokemon.base_computed_stats.hp / 8;
    pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
    logs.push(format!("{} pierde {} HP por Leech Seed", pokemon.species.display_name, damage));

    // Curar al source
    if let Some(source_pos) = pokemon.volatile_status.leech_seed_source {
        // get_pokemon_mut(source_pos).current_hp += damage;
    }
}
```

**4. Substitute** (No existe)
```rust
// Agregar a VolatileStatus:
pub substitute_hp: Option<u16>,

// En execute_single_action, antes de aplicar daño:
if target.volatile_status.substitute_hp.is_some() {
    let sub_hp = target.volatile_status.substitute_hp.unwrap();
    if damage >= sub_hp {
        damage -= sub_hp;
        target.volatile_status.substitute_hp = None;
        logs.push("¡El sustituto se rompió!");
    } else {
        target.volatile_status.substitute_hp = Some(sub_hp - damage);
        damage = 0;
        logs.push("¡El sustituto aguantó el golpe!");
    }
}
```

**5. Perish Song** (No existe)
```rust
// Agregar a VolatileStatus:
pub perish_count: Option<u8>,

// Al usar Perish Song:
for pokemon in all_pokemon_on_field() {
    pokemon.volatile_status.perish_count = Some(3);
}

// En process_end_of_turn_residuals():
if let Some(count) = pokemon.volatile_status.perish_count {
    if count == 0 {
        pokemon.current_hp = 0;
        logs.push(format!("{} se debilitó por Perish Song!", pokemon.species.display_name));
    } else {
        pokemon.volatile_status.perish_count = Some(count - 1);
    }
}
```

#### Ubicación de cambios
```
core/src/models.rs
├── VolatileStatus: agregar campos (infatuated_by, leech_seeded, substitute_hp, perish_count)

core/src/battle/checks.rs
├── can_pokemon_move(): agregar checks de confusion e infatuation

core/src/battle/systems/effect_system/effects_handler.rs
├── apply_confusion_damage()
├── apply_leech_seed_damage()
├── process_substitute()
├── process_perish_song()

core/src/battle/pipeline.rs
├── apply_volatile_status_effects()
```

---

### 1.4 Trick Room
**Prioridad**: 🟡 ALTA

#### Implementación
```rust
// En BattleState:
pub trick_room_active: bool,
pub trick_room_turns_left: u8,

// En sort_candidates() - pipeline.rs línea 451:
fn sort_candidates(candidates: &mut Vec<ActionCandidate>, rng: &mut StdRng, trick_room: bool) {
    with_random.sort_by(|a, b| {
        match b.0.priority.cmp(&a.0.priority) {
            std::cmp::Ordering::Equal => {
                // CAMBIO: invertir si Trick Room activo
                if trick_room {
                    a.0.speed.cmp(&b.0.speed)  // Más lento va primero
                } else {
                    b.0.speed.cmp(&a.0.speed)  // Más rápido va primero (normal)
                }
            }
            other => other,
        }
    });
}

// En process_end_of_turn_residuals():
if battle_state.trick_room_active {
    battle_state.trick_room_turns_left -= 1;
    if battle_state.trick_room_turns_left == 0 {
        battle_state.trick_room_active = false;
        logs.push("¡Trick Room se desactivó!");
    }
}
```

---

## FASE 2: Mecánicas Competitivas (Esenciales para meta)

**Duración estimada**: 3-4 semanas
**Objetivo**: Habilitar estrategias avanzadas

### 2.1 Protecciones Avanzadas
**Prioridad**: 🟡 ALTA

#### Wide Guard
```rust
// Bloquea spread moves (Earthquake, Discharge, Rock Slide)
// En execute_single_action, antes de aplicar daño:
if target_team_has_wide_guard_active() && is_spread_move(move_data) {
    logs.push("¡Wide Guard protegió al equipo!");
    return; // No aplicar daño
}

fn is_spread_move(move_data: &MoveData) -> bool {
    move_data.target == "all-opponents" || move_data.target == "all-other-pokemon"
}
```

#### Quick Guard
```rust
// Bloquea movimientos con prioridad positiva
if target_team_has_quick_guard_active() && move_priority > 0 {
    logs.push("¡Quick Guard protegió al equipo!");
    return;
}
```

#### Mat Block
```rust
// Solo funciona el primer turno del usuario
// Agregar a VolatileStatus:
pub mat_block_active: bool,
pub turns_since_entered_field: u8,

// Al usar Mat Block:
if pokemon.volatile_status.turns_since_entered_field == 0 {
    team_protection.mat_block_active = true;
} else {
    logs.push("¡Mat Block falló!");
}
```

---

### 2.2 Abilities Críticas Faltantes
**Prioridad**: 🟡 ALTA

#### Download
```rust
// En apply_on_entry_stat_change() - pipeline.rs:
if ability == "download" {
    // Comparar Def vs SpDef del oponente
    let opponent = get_opponent_active(battle_state);
    let def = opponent.base_computed_stats.defense;
    let spdef = opponent.base_computed_stats.special_defense;

    if def < spdef {
        apply_stat_stage_change(pokemon, "special_attack", 1);
        logs.push(format!("{} copió la defensa baja! SpA +1", pokemon.species.display_name));
    } else {
        apply_stat_stage_change(pokemon, "attack", 1);
        logs.push(format!("{} copió la defensa especial baja! Atk +1", pokemon.species.display_name));
    }
}
```

#### Solid Rock / Filter
```rust
// En calculate_damage() - calculator.rs:
// Después de calcular type effectiveness
if defender_ability == "solid-rock" || defender_ability == "filter" {
    if type_effectiveness > 1.0 {
        damage = (damage as f32 * 0.75) as u16;  // -25% de daño super efectivo
    }
}
```

#### Sheer Force
```rust
// En calculate_damage():
if attacker_ability == "sheer-force" && move_has_secondary_effect(move_data) {
    damage = (damage as f32 * 1.3) as u16;
    // Y eliminar efectos secundarios en apply_move_effects()
}
```

#### Technician
```rust
// En calculate_damage():
if attacker_ability == "technician" {
    if let Some(power) = move_data.power {
        if power <= 60 {
            damage = (damage as f32 * 1.5) as u16;
        }
    }
}
```

#### Regenerator
```rust
// Actualmente solo está mencionado pero no funciona
// En switch_pokemon():
if pokemon.ability == "regenerator" {
    let heal_amount = pokemon.base_computed_stats.hp / 3;
    pokemon.current_hp = (pokemon.current_hp + heal_amount).min(pokemon.base_computed_stats.hp);
    logs.push(format!("{} se curó 1/3 de su HP!", pokemon.species.display_name));
}
```

---

### 2.3 Movimientos de Switch Forzado
**Prioridad**: 🟡 ALTA

#### Dragon Tail / Roar
```rust
// En apply_move_effects():
if move_id == "dragon-tail" || move_id == "roar" {
    // Forzar switch del objetivo
    force_switch(target_position, battle_state, logs);
}

fn force_switch(
    target_pos: FieldPosition,
    battle_state: &mut BattleState,
    logs: &mut Vec<String>,
) {
    // Si es oponente y hay más Pokémon
    if battle_state.opponent_team.len() > 1 {
        let next_index = battle_state.get_next_available_opponent();
        battle_state.switch_opponent(target_pos.team_index, next_index);
        logs.push(format!("¡{} fue forzado a salir!", target.species.display_name));
    }
}
```

---

### 2.4 Fixes a Mecánicas Existentes

#### Fix: Bad Poison escalante
```rust
// En models.rs, agregar:
pub struct VolatileStatus {
    // ...
    pub badly_poisoned_turns: u8,  // Contador de turnos con BadPoison
}

// En apply_residual_effects() - effects.rs:
StatusCondition::BadPoison => {
    let turns = pokemon.volatile_status.badly_poisoned_turns + 1;
    let damage = (max_hp / 16) * turns as u16;  // Escala: 1/16, 2/16, 3/16...
    pokemon.current_hp = pokemon.current_hp.saturating_sub(damage);
    pokemon.volatile_status.badly_poisoned_turns = turns;
    logs.push(format!("{} pierde {} HP por envenenamiento grave", pokemon.species.display_name, damage));
}
```

#### Fix: Intimidate en Doubles afecta múltiples
```rust
// pipeline.rs línea 637 - actualmente solo afecta uno
// ANTES:
for &opp_idx in &battle_state.opponent_active_indices {
    if let Some(opponent) = battle_state.opponent_team.get_mut(opp_idx) {
        apply_stat_stage_change(opponent, "attack", -1);
        break;  // ❌ Solo afecta al primero
    }
}

// DESPUÉS:
for &opp_idx in &battle_state.opponent_active_indices {
    if let Some(opponent) = battle_state.opponent_team.get_mut(opp_idx) {
        // Verificar inmunidad (Clear Body, Hyper Cutter, etc.)
        if !has_immunity_to_stat_drop(opponent, "attack") {
            apply_stat_stage_change(opponent, "attack", -1);
            logs.push(format!("¡{} intimidó a {}!", pokemon.species.display_name, opponent.species.display_name));
        }
    }
    // ✅ Continuar al siguiente
}
```

#### Fix: Grassy Terrain curación
```rust
// En process_end_of_turn_residuals():
if let Some(terrain) = &battle_state.terrain {
    if terrain.terrain_type == TerrainType::Grassy {
        for pokemon in all_active_pokemon() {
            if is_grounded(pokemon) {
                let heal = pokemon.base_computed_stats.hp / 16;
                pokemon.current_hp = (pokemon.current_hp + heal).min(pokemon.base_computed_stats.hp);
                logs.push(format!("{} se curó {} HP por Grassy Terrain", pokemon.species.display_name, heal));
            }
        }
    }
}
```

---

## FASE 3: Refinamiento y Optimización

**Duración estimada**: 2-3 semanas
**Objetivo**: Pulir detalles y edge cases

### 3.1 Items Avanzados
- Focus Sash (sobrevive con 1 HP si tenía HP completo)
- Rocky Helmet (daño contraataque por contacto)
- Air Balloon (Levitate hasta ser golpeado)
- Eject Button (forced switch al ser golpeado)
- Mental Herb (cura Infatuation, Taunt, etc.)

### 3.2 Abilities Restantes
- Iron Fist, Reckless, Poison Heal, Magic Guard
- Multiscale, Weak Armor, Friend Guard
- Symbiosis, Receiver, Power of Alchemy

### 3.3 Movimientos Especiales
- Parting Shot, U-turn, Volt Switch (switch después de atacar)
- Final Gambit (HP actual como daño, siempre fainta)
- Destiny Bond (ambos se debilitan)
- Protect variants (Baneful Bunker)

### 3.4 Interacciones Complejas
- Weather + Ability + Item combinations
- Terrain + Type immunity interactions
- Priority system con Trick Room
- Spread damage calculation refinement

---

## FASE 4: Testing y Balanceo

**Duración estimada**: 1-2 semanas
**Objetivo**: Garantizar estabilidad y precisión

### 4.1 Test Coverage
- Unit tests para cada item
- Integration tests para combos (Weather + Ability + Item)
- VGC scenario tests (Follow Me + spread moves, Trick Room + slow attackers)

### 4.2 Damage Calculator Validation
- Comparar con Pokémon Showdown damage calculator
- Verificar edge cases (min/max rolls, critical hits, etc.)

### 4.3 Performance Optimization
- Profiling del motor de batalla
- Optimizar loops en resolve_targets y apply_effects
- Cache de cálculos repetidos

---

## Estimación Total

| Fase | Duración | Complejidad | Cobertura Final |
|------|----------|-------------|-----------------|
| Fase 1 | 2-3 semanas | Alta | ~60% |
| Fase 2 | 3-4 semanas | Media-Alta | ~80% |
| Fase 3 | 2-3 semanas | Media | ~95% |
| Fase 4 | 1-2 semanas | Baja | 100% |
| **TOTAL** | **8-12 semanas** | - | **100%** |

---

## Prioridades Inmediatas (Next Sprint)

1. ✅ **Items System** - Critical path blocker
   - Choice items primero (más usado)
   - Life Orb segundo
   - Assault Vest tercero

2. ✅ **Redirection** - Core VGC mechanic
   - Follow Me / Spotlight
   - Rage Powder

3. ✅ **Volatile Status** - Gameplay depth
   - Confusion (campo ya existe)
   - Infatuation
   - Leech Seed

4. ✅ **Trick Room** - Speed control essential
   - Modificar sort_candidates
   - Agregar counter de turnos

5. ⚠️ **Fixes Críticos**
   - Bad Poison escalante
   - Intimidate múltiple
   - Grassy Terrain curación

---

## Arquitectura Propuesta para Nuevos Sistemas

```
core/src/battle/systems/
├── item_system/
│   ├── mod.rs                    (registry + exports)
│   ├── item_effects.rs           (lógica de efectos)
│   ├── item_triggers.rs          (triggers + conditions)
│   └── item_processor.rs         (aplicación en pipeline)
├── redirection_system/
│   ├── mod.rs
│   ├── redirection_state.rs      (state tracking)
│   └── redirection_resolver.rs   (targeting override)
├── volatile_system/               (extender effect_system)
│   ├── confusion.rs
│   ├── infatuation.rs
│   ├── leech_seed.rs
│   └── substitute.rs
└── field_effects_system/          (extender effect_system)
    ├── trick_room.rs
    ├── tailwind.rs
    └── terrain_residuals.rs
```

---

## Métricas de Éxito

- ✅ Compilación limpia sin warnings
- ✅ 90%+ test coverage en nuevos sistemas
- ✅ Damage calculations match Pokémon Showdown
- ✅ VGC 2024 ruleset completamente soportado
- ✅ Performance: <50ms por turno en battles de 4 Pokémon
- ✅ Documentación completa de todas las mecánicas

---

## Notas de Implementación

### Reglas de Oro
1. **Backward Compatibility**: No romper funcionalidad existente
2. **Test-Driven**: Tests unitarios antes de implementar
3. **Modular**: Cada sistema independiente con interfaces claras
4. **Documentación**: Comentarios explicando lógica VGC oficial
5. **Performance**: Profiling antes y después de cada feature

### Referencias
- [Bulbapedia - Battle mechanics](https://bulbapedia.bulbagarden.net/wiki/Stat)
- [Smogon VGC 2024 ruleset](https://www.smogon.com/dex/sv/formats/vgc24/)
- [Pokémon Showdown source](https://github.com/smogon/pokemon-showdown)
- Damage calculator: https://calc.pokemonshowdown.com/

---

**Última actualización**: 2025-12-22
**Analista**: Claude Sonnet 4.5
**Estado**: Plan aprobado, listo para implementación
