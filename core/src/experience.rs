use crate::game::GameConfig;
use crate::models::{PokemonInstance, PokemonSpecies};
use crate::factory::compute_stats;

/// Aplica la subida de nivel por hitos después de una victoria
/// 
/// Sistema de Milestone Leveling:
/// - Batalla salvaje: sube `10 / gym_interval` niveles
/// - Batalla de gimnasio: sube 5 niveles
/// - Nivel máximo: 100
/// 
/// Retorna el número de niveles subidos
pub fn apply_victory_level_up(
    pokemon: &mut PokemonInstance,
    _species: &PokemonSpecies,
    game_config: &GameConfig,
    is_gym_victory: bool,
) -> u8 {
    let old_level = pokemon.level;
    
    // Calcular niveles a añadir
    let levels_to_add = if is_gym_victory {
        5u8
    } else {
        // Batalla salvaje: 10 / gym_interval
        // Usar división entera y asegurar mínimo 1
        let interval = game_config.gym_interval.max(1);
        (10u32 / interval) as u8
    };
    
    // Calcular nuevo nivel (máximo 100)
    let new_level = (old_level as u16 + levels_to_add as u16).min(100) as u8;
    
    // Si el nivel cambió, recalcular stats
    if new_level > old_level {
        let levels_gained = new_level - old_level;
        
        // Calcular ratio de HP actual para preservarlo
        let hp_ratio = if pokemon.base_computed_stats.hp > 0 {
            pokemon.current_hp as f32 / pokemon.base_computed_stats.hp as f32
        } else {
            1.0
        };
        
        // Recalcular stats usando los mismos IVs y EVs, pero con el nuevo nivel
        let new_stats = compute_stats(
            &pokemon.species.base_stats,
            &pokemon.individual_values,
            &pokemon.effort_values,
            new_level,
        );
        
        // Obtener el nuevo HP máximo
        let new_max_hp = new_stats.hp;
        
        // Actualizar el nivel y los stats
        pokemon.level = new_level;
        pokemon.base_computed_stats = new_stats;
        
        // Aplicar el ratio de HP al nuevo HP máximo
        pokemon.current_hp = (new_max_hp as f32 * hp_ratio) as u16;
        
        levels_gained
    } else {
        0
    }
}

