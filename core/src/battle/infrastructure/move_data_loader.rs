//! Cargador de datos de movimientos
//!
//! Este módulo proporciona funciones para resolver y cargar datos de movimientos
//! desde el pool global o crear movimientos de fallback como Struggle.

use std::collections::HashMap;
use crate::models::MoveData;
use super::super::checks::create_struggle_move;

/// Resuelve los datos de un movimiento desde el pool global
///
/// Si el movimiento no se encuentra en el pool, o si el move_id es "struggle",
/// retorna un movimiento Struggle como fallback.
///
/// # Argumentos
/// * `move_id` - ID del movimiento a resolver
/// * `pool` - Pool opcional de movimientos disponibles
///
/// # Retorna
/// `MoveData` del movimiento solicitado o Struggle si no se encuentra
pub fn resolve_move_data(move_id: &str, pool: Option<&HashMap<String, MoveData>>) -> MoveData {
    // Si es struggle, retornar directamente
    if move_id == "struggle" {
        return create_struggle_move();
    }

    // Intentar obtener del pool
    if let Some(p) = pool {
        if let Some(data) = p.get(move_id) {
            return data.clone();
        }
    }

    // Fallback de emergencia: Struggle
    create_struggle_move()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_struggle() {
        let move_data = resolve_move_data("struggle", None);
        assert_eq!(move_data.id, "struggle");
        assert_eq!(move_data.power, Some(50));
    }

    #[test]
    fn test_resolve_from_empty_pool() {
        let pool = HashMap::new();
        let move_data = resolve_move_data("tackle", Some(&pool));
        // Should fallback to struggle
        assert_eq!(move_data.id, "struggle");
    }

    #[test]
    fn test_resolve_from_pool() {
        let mut pool = HashMap::new();
        let tackle = MoveData {
            id: "tackle".to_string(),
            name: "Tackle".to_string(),
            power: Some(40),
            pp: 35,
            ..Default::default()
        };
        pool.insert("tackle".to_string(), tackle.clone());

        let move_data = resolve_move_data("tackle", Some(&pool));
        assert_eq!(move_data.id, "tackle");
        assert_eq!(move_data.power, Some(40));
    }
}
