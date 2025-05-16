#[cfg(test)]
mod map_tests {
    use crate::simulation::entities::{Map, ResourceType};
    use crate::simulation::map::TerrainType;

    #[test]
    fn test_map_creation() {
        let width = 20;
        let height = 15;
        let seed = 42;

        let map = Map::new(width, height, seed);

        // Verify dimensions
        assert_eq!(map.width, width);
        assert_eq!(map.height, height);
        assert_eq!(map.seed, seed);

        // Verify terrain generation
        assert!(!map.terrain.is_empty());
        assert_eq!(map.terrain.len(), height);
        assert_eq!(map.terrain[0].len(), width);

        // Verify discovery state
        assert!(!map.discovered.is_empty());
        assert_eq!(map.discovered.len(), height);
        assert_eq!(map.discovered[0].len(), width);

        // Verify resources exist
        assert!(!map.resources.is_empty());
    }

    #[test]
    fn test_terrain_distribution() {
        let width = 50;
        let height = 50;
        let seed = 42;

        let map = Map::new(width, height, seed);

        // Count terrain types
        let mut plains = 0;
        let mut hills = 0;
        let mut mountains = 0;
        let mut canyons = 0;

        for y in 0..height {
            for x in 0..width {
                match TerrainType::from(map.terrain[y][x]) {
                    TerrainType::Plain => plains += 1,
                    TerrainType::Hill => hills += 1,
                    TerrainType::Mountain => mountains += 1,
                    TerrainType::Canyon => canyons += 1,
                }
            }
        }

        let total = width * height;

        // Verify all terrain types exist
        assert!(plains > 0);
        assert!(hills > 0);
        assert!(mountains > 0);
        assert!(canyons > 0);

        // Verify terrain distribution (approximate)
        assert!((plains as f64 / total as f64) > 0.2); // At least 20% plains
        assert!((canyons as f64 / total as f64) < 0.3); // Less than 30% canyons
    }

    #[test]
    fn test_resource_distribution() {
        let width = 50;
        let height = 50;
        let seed = 42;

        let map = Map::new(width, height, seed);

        // Count resource types
        let mut energy = 0;
        let mut minerals = 0;
        let mut scientific = 0;

        for ((_, _), (res_type, _)) in &map.resources {
            match res_type {
                ResourceType::Energy => energy += 1,
                ResourceType::Mineral => minerals += 1,
                ResourceType::ScientificInterest => scientific += 1,
            }
        }

        // Verify all resource types exist
        assert!(energy > 0);
        assert!(minerals > 0);
        assert!(scientific > 0);

        // Energy should be most common, scientific interest least common
        assert!(energy > minerals);
        assert!(minerals > scientific);
    }

    #[test]
    fn test_map_reproducibility() {
        let width = 30;
        let height = 30;
        let seed = 12345;

        // Create two maps with the same seed
        let map1 = Map::new(width, height, seed);
        let map2 = Map::new(width, height, seed);

        // Verify terrain is identical
        for y in 0..height {
            for x in 0..width {
                assert_eq!(map1.terrain[y][x], map2.terrain[y][x]);
            }
        }

        // Verify resources are identical
        assert_eq!(map1.resources.len(), map2.resources.len());

        for ((x, y), (res_type, amount)) in &map1.resources {
            let resource2 = map2.resources.get(&(*x, *y));
            assert!(resource2.is_some());
            let (res_type2, amount2) = resource2.unwrap();
            assert_eq!(res_type, res_type2);
            assert_eq!(amount, amount2);
        }
    }

    #[test]
    fn test_resource_collection() {
        let width = 20;
        let height = 20;
        let seed = 42;

        let mut map = Map::new(width, height, seed);

        // Find a position with resources
        let resource_pos = if let Some(((x, y), (res_type, amount))) = map.resources.iter().next() {
            Some((*x, *y, res_type.clone(), *amount))
        } else {
            None
        };

        // Verify we found a resource
        assert!(resource_pos.is_some());

        let (x, y, res_type, amount) = resource_pos.unwrap();

        // Collect half the resource
        let collect_amount = amount / 2;
        let result = map.collect_resource(x, y, collect_amount);

        // Verify collection succeeded
        assert!(result.is_ok());
        let (collected_type, collected_amount) = result.unwrap();
        assert_eq!(collected_type, res_type);
        assert_eq!(collected_amount, collect_amount);

        // Verify resource was reduced
        let remaining = map.resources.get(&(x, y)).unwrap();
        assert_eq!(remaining.1, amount - collect_amount);

        // Collect the rest
        let result = map.collect_resource(x, y, amount - collect_amount);
        assert!(result.is_ok());

        // Verify resource is gone
        assert!(!map.resources.contains_key(&(x, y)));

        // Try to collect more (should fail)
        let result = map.collect_resource(x, y, 1);
        assert!(result.is_err());
    }
}
