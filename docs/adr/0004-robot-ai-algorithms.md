# ADR-004: Algorithmes d'Intelligence Artificielle des Robots

**Statut**: Accepté  
**Date**: 2024-01-15  
**Auteurs**: Équipe Nova (Edofo, Laurent, Aymene, Benjamin)  
**Décideurs**: Équipe de développement

## Contexte et Problématique

Nova nécessite trois types de robots autonomes avec des comportements spécialisés et des algorithmes d'IA distincts. Chaque robot doit prendre des décisions intelligentes basées sur son type, l'état de l'environnement, et ses contraintes énergétiques. Le système doit démontrer des concepts avancés d'IA multi-agents avec des comportements émergents.

### Exigences Fonctionnelles

- **Trois types de robots spécialisés** avec comportements distincts
- **Prise de décision autonome** basée sur l'état et l'environnement
- **Gestion intelligente de l'énergie** avec seuils adaptatifs
- **Pathfinding optimal** utilisant l'algorithme A\*
- **Comportements émergents** sans contrôle centralisé
- **Isolation d'information** jusqu'au retour à la station

### Contraintes Techniques

- **Performance**: Décisions en temps réel pour 50+ robots
- **Mémoire**: Utilisation efficace sans fuites
- **Extensibilité**: Facilité d'ajout de nouveaux comportements
- **Testabilité**: Comportements déterministes et vérifiables

## Options Considérées

### Option 1: Système de Règles Basé sur Priorités

**Architecture**:

```rust
trait RobotBehavior {
    fn decide_action(&self, robot: &Robot, map: &Map) -> RobotAction;
    fn decide_action_with_station(&self, robot: &Robot, map: &Map, station: &Station) -> RobotAction;
}
```

**Avantages**:

- Comportements prévisibles et déterministes
- Facilité de débogage et de test
- Performance optimale (O(1) pour la plupart des décisions)
- Séparation claire des responsabilités

**Inconvénients**:

- Moins de flexibilité que les approches d'apprentissage
- Nécessite un réglage manuel des priorités

### Option 2: Réseaux de Neurones / Apprentissage par Renforcement

**Architecture**: Utilisation de bibliothèques comme `candle` ou `tch`

**Avantages**:

- Comportements adaptatifs et d'apprentissage
- Potentiel pour des stratégies optimales émergentes

**Inconvénients**:

- Complexité d'implémentation significative
- Temps d'entraînement requis
- Comportements non déterministes (difficiles à tester)
- Overhead de performance important

### Option 3: Systèmes Multi-Agents avec Communication

**Architecture**: Robots communiquant directement entre eux

**Avantages**:

- Coordination avancée possible
- Optimisation globale des tâches

**Inconvénients**:

- Complexité de synchronisation
- Violation du principe d'isolation d'information
- Difficultés de test et de débogage

## Décision: Option 1 - Système de Règles Basé sur Priorités

### Justification

L'Option 1 a été choisie car elle offre le meilleur équilibre entre:

- **Simplicité d'implémentation** et de maintenance
- **Performance** pour traitement temps réel
- **Déterminisme** nécessaire pour les tests
- **Respect des contraintes** du projet (isolation d'information)

## Implémentation Détaillée

### Architecture des Comportements

```rust
// Trait principal pour tous les comportements
pub trait RobotBehavior {
    fn decide_action(&self, robot: &Robot, map: &Map) -> RobotAction;
    fn decide_action_with_station(&self, robot: &Robot, map: &Map, station: &Station) -> RobotAction;
    fn can_execute(&self, robot: &Robot, action: &RobotAction) -> bool;
}

// Actions possibles pour les robots
pub enum RobotAction {
    Move(Direction),
    Idle,
    CollectResource,
    ReturnToStation,
}
```

### Algorithmes par Type de Robot

#### 1. Explorer Robot - Algorithme de Cartographie

**Objectif**: Exploration systématique et cartographie d'zones inconnues

**Stratégie de Décision**:

```rust
Priority 9: Return to station (energy < 20 OR carrying resources)
Priority 7: Explore unexplored areas (radius 5)
Priority 5: Random exploration (deterministic based on robot state)
```

**Algorithme d'Exploration**:

- **Recherche en spirale** autour de la position actuelle
- **Évitement des zones déjà explorées** par d'autres robots
- **Pathfinding A\*** vers les zones cibles
- **Exploration déterministe** basée sur l'ID du robot pour éviter les conflits

**Optimisations**:

- Coût énergétique le plus faible (2 unités/action)
- Seuil de retour conservateur (20 unités)
- Rayon de recherche optimal (5 unités)

#### 2. Harvester Robot - Algorithme de Collecte de Ressources

**Objectif**: Collecte efficace de ressources Energy et Mineral

**Stratégie de Décision**:

```rust
Priority 10: Return to station (energy < 15 OR carrying resources)
Priority 8:  Harvest preferred resources (Energy/Mineral)
Priority 6:  Explore for new resource locations
```

**Algorithme de Recherche de Ressources**:

- **Pathfinding A\*** vers la ressource la plus proche
- **Filtrage par type** (Energy, Mineral uniquement)
- **Évaluation distance vs. quantité** pour optimiser l'efficacité
- **Exploration ciblée** quand aucune ressource connue n'est disponible

**Optimisations**:

- Coût énergétique modéré (3 unités/action)
- Seuil de retour agressif (15 unités) pour maximiser la collecte
- Rayon de recherche équilibré (4 unités)

#### 3. Scientist Robot - Algorithme de Recherche Scientifique

**Objectif**: Analyse de points d'intérêt scientifique

**Stratégie de Décision**:

```rust
Priority 9: Return to station (energy < 25 OR carrying data)
Priority 8: Analyze scientific interests (Chemical analysis)
Priority 6: Systematic scientific exploration
```

**Algorithme d'Analyse**:

- **Recherche spécialisée** pour points ScientificInterest uniquement
- **Pathfinding A\*** vers les sites d'analyse
- **Exploration systématique** en grille pour couverture complète
- **Analyse de type chimique** basée sur la densité de ressources

**Optimisations**:

- Coût énergétique le plus élevé (4 unités/action)
- Seuil de retour prudent (25 unités) pour missions longues
- Rayon de recherche étendu (6 unités) pour la recherche

### Gestion Intelligente de l'Énergie

#### Algorithmes de Décision Énergétique

```rust
// Évaluation si le robot peut continuer sa mission
pub fn should_continue_mission(&self, station_position: (usize, usize)) -> bool {
    if self.carrying.is_some() { return false; }
    if !self.can_return_to_station(station_position) { return false; }

    let energy_to_return = self.energy_to_return(station_position);
    let safety_margin = 20;
    self.energy > energy_to_return + safety_margin
}

// Calcul de l'énergie nécessaire pour retourner
pub fn energy_to_return(&self, station_position: (usize, usize)) -> u32 {
    let distance = self.manhattan_distance_to(station_position);
    distance * MOVE_ENERGY_COST + 10 // Marge de sécurité
}
```

#### Stratégies par Type:

- **Explorer**: Seuil conservateur (20) pour exploration continue
- **Harvester**: Seuil agressif (15) pour maximiser les cycles de collecte
- **Scientist**: Seuil prudent (25) pour missions d'analyse longues

### Algorithme de Pathfinding A\*

#### Implémentation Optimisée

```rust
pub fn find_path(start: (usize, usize), goal: (usize, usize), map: &Map) -> Option<Vec<(usize, usize)>> {
    // Utilisation d'un BinaryHeap pour l'open set
    // Heuristique: Distance de Manhattan
    // Coût: 1 pour mouvements cardinaux, √2 pour diagonaux
    // Terminaison anticipée quand le goal est atteint
}
```

#### Optimisations:

- **Heuristique admissible** (distance Manhattan)
- **Structures de données efficaces** (BinaryHeap, HashMap)
- **Terminaison anticipée** dès que le goal est atteint
- **Gestion des obstacles** dynamique
- **Support 8-directionnel** pour navigation fluide

### Comportements Émergents

#### Patterns Observés:

- **Spécialisation naturelle**: Chaque type se concentre sur sa mission
- **Évitement de conflits**: Exploration déterministe évite les collisions
- **Efficacité collective**: Optimisation locale mène à performance globale
- **Adaptation dynamique**: Réponse aux changements d'environnement

## Tests et Validation

### Couverture de Tests

- ✅ **25 tests comportementaux** pour les 3 types de robots
- ✅ **Tests de pathfinding** (4 tests A\*)
- ✅ **Tests de gestion énergétique** (15 tests)
- ✅ **Tests d'intégration** robot-station (8 tests)
- ✅ **Tests de performance** sous charge (5 tests)

### Critères de Validation

- [x] Comportements déterministes et reproductibles
- [x] Performance > 10 FPS avec 50 robots
- [x] Pas de deadlocks ou comportements bloquants
- [x] Gestion d'énergie sans robots "perdus"
- [x] Pathfinding optimal sans boucles infinies

### Métriques de Performance

```rust
// Mesures typiques observées:
- Décision comportementale: < 1ms par robot
- Pathfinding A*: < 5ms pour chemins de 20 cases
- Traitement concurrent: 50 robots en < 100ms
- Mémoire: < 50MB pour simulation complète
```

## Avantages de la Solution

### Performance

- **Décisions O(1)** pour la plupart des cas
- **Pathfinding optimal** avec A\*
- **Traitement concurrent** sans overhead significatif

### Maintenabilité

- **Séparation claire** des comportements par type
- **Code modulaire** facile à étendre
- **Tests déterministes** pour validation continue

### Extensibilité

- **Trait RobotBehavior** permet l'ajout facile de nouveaux types
- **Architecture modulaire** pour nouvelles stratégies
- **Système de priorités** flexible et configurable

## Limitations et Évolutions Futures

### Limitations Actuelles

- **Pas d'apprentissage adaptatif** (comportements fixes)
- **Communication limitée** entre robots (par design)
- **Optimisation locale** uniquement (pas globale)

### Évolutions Possibles

- **Apprentissage par renforcement** pour optimisation des stratégies
- **Communication inter-robots** pour coordination avancée
- **Planification multi-objectifs** pour optimisation globale
- **Comportements adaptatifs** basés sur l'historique

## Conclusion

L'architecture basée sur des règles de priorités offre une solution robuste et performante pour l'IA des robots Nova. Elle respecte toutes les contraintes du projet tout en démontrant des concepts avancés d'intelligence artificielle multi-agents.

Cette approche permet des comportements émergents complexes à partir de règles simples, illustrant parfaitement les principes des systèmes multi-agents autonomes. La performance et la maintenabilité de la solution en font une base solide pour l'évolution future du système.

## Références

- [A\* Search Algorithm](https://en.wikipedia.org/wiki/A*_search_algorithm) - Algorithme de pathfinding utilisé
- [Multi-Agent Systems](https://en.wikipedia.org/wiki/Multi-agent_system) - Concepts théoriques appliqués
- [Rust Performance Guide](https://nnethercote.github.io/perf-book/) - Optimisations performance Rust
- [Behavior Trees](<https://en.wikipedia.org/wiki/Behavior_tree_(artificial_intelligence)>) - Alternative considérée pour les comportements
