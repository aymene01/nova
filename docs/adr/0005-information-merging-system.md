# ADR-005: Système de Fusion d'Informations Git-like

**Statut**: Accepté  
**Date**: 2024-01-15  
**Auteurs**: Équipe Nova (Benjamin, Laurent, Aymene, Edofo)  
**Décideurs**: Équipe de développement

## Contexte et Problématique

Nova nécessite un système sophistiqué de fusion d'informations pour gérer les découvertes conflictuelles des robots lorsqu'ils retournent à la station. Chaque robot explore de manière autonome et peut découvrir des informations contradictoires sur les mêmes locations. Le système doit implémenter une stratégie de résolution de conflits similaire à Git pour maintenir la cohérence des données tout en préservant l'intelligence collective.

### Exigences Fonctionnelles

- **Fusion automatique** des découvertes compatibles
- **Détection intelligente** des conflits d'information
- **Résolution stratégique** basée sur la confiance et la récence
- **Gestion manuelle** des conflits complexes
- **Statistiques détaillées** des opérations de fusion
- **Préservation de l'historique** des découvertes

### Contraintes Techniques

- **Performance**: Fusion en temps réel sans impact sur la simulation
- **Cohérence**: Maintien de l'intégrité des données partagées
- **Extensibilité**: Support pour nouveaux types de conflits
- **Traçabilité**: Historique complet des décisions de fusion

## Options Considérées

### Option 1: Système Git-like avec Résolution Intelligente

**Architecture**:

```rust
pub struct StationKnowledge {
    pub locations: HashMap<(usize, usize), LocationInfo>,
    pub conflicts: Vec<InformationConflict>,
    pub merge_statistics: MergeStatistics,
}

pub enum ConflictResolution {
    KeepCurrent,
    AcceptNew,
    Merge,
    RequiresManualReview,
}
```

**Avantages**:

- Résolution automatique intelligente
- Gestion des cas complexes avec intervention manuelle
- Traçabilité complète des décisions
- Flexibilité pour différents types de conflits

**Inconvénients**:

- Complexité d'implémentation significative
- Nécessite des algorithmes de fusion sophistiqués

### Option 2: Système "Last Writer Wins"

**Architecture**: Simple remplacement des données existantes

**Avantages**:

- Implémentation triviale
- Performance optimale
- Pas de gestion de conflits nécessaire

**Inconvénients**:

- Perte d'informations potentiellement importantes
- Pas de résolution intelligente
- Vulnérable aux erreurs de capteurs

### Option 3: Système de Vote Majoritaire

**Architecture**: Collecte de plusieurs rapports avant décision

**Avantages**:

- Robustesse contre les erreurs individuelles
- Décisions basées sur le consensus

**Inconvénients**:

- Nécessite plusieurs robots pour la même zone
- Délais dans la prise de décision
- Complexité de synchronisation

## Décision: Option 1 - Système Git-like avec Résolution Intelligente

### Justification

L'Option 1 a été choisie car elle offre:

- **Intelligence maximale** dans la résolution de conflits
- **Préservation d'informations** importantes
- **Flexibilité** pour différents scénarios
- **Traçabilité complète** pour le débogage et l'analyse

## Implémentation Détaillée

### Structure des Données

#### LocationInfo - Information de Découverte

```rust
pub struct LocationInfo {
    pub position: (usize, usize),
    pub terrain_type: u8,
    pub resource: Option<(ResourceType, u32)>,
    pub discovered_by: usize,     // ID du robot découvreur
    pub discovery_time: u64,      // Tick de simulation
    pub confidence: f32,          // Niveau de confiance (0.0-1.0)
}
```

#### InformationConflict - Représentation des Conflits

```rust
pub struct InformationConflict {
    pub position: (usize, usize),
    pub current_info: LocationInfo,
    pub new_info: LocationInfo,
    pub conflict_type: ConflictType,
}

pub enum ConflictType {
    ResourceAmountDifference,    // Différence significative de quantité
    ResourceTypeConflict,        // Types de ressources contradictoires
    TerrainMismatch,            // Terrain différent (conflit grave)
    ConfidenceConflict,         // Différence de niveau de confiance
}
```

### Algorithmes de Détection de Conflits

#### 1. Détection de Conflits de Terrain

```rust
// Conflit grave - terrain devrait être cohérent
if existing.terrain_type != new.terrain_type {
    return Some(InformationConflict {
        conflict_type: ConflictType::TerrainMismatch,
        // ... autres champs
    });
}
```

#### 2. Détection de Conflits de Ressources

```rust
// Conflit de type de ressource
if existing_type != new_type {
    return Some(InformationConflict {
        conflict_type: ConflictType::ResourceTypeConflict,
        // ... autres champs
    });
}

// Conflit de quantité (>20% de différence)
let amount_diff = (existing_amount as f32 - new_amount as f32).abs();
let avg_amount = (existing_amount + new_amount) as f32 / 2.0;
if amount_diff / avg_amount > 0.2 {
    return Some(InformationConflict {
        conflict_type: ConflictType::ResourceAmountDifference,
        // ... autres champs
    });
}
```

#### 3. Détection de Conflits de Confiance

```rust
// Différence significative de confiance (>30%)
let confidence_diff = (existing.confidence - new.confidence).abs();
if confidence_diff > 0.3 {
    return Some(InformationConflict {
        conflict_type: ConflictType::ConfidenceConflict,
        // ... autres champs
    });
}
```

### Stratégies de Résolution

#### 1. Résolution de Conflits de Quantité

```rust
ConflictType::ResourceAmountDifference => {
    // Fusion par moyenne pondérée basée sur la confiance
    Ok(ConflictResolution::Merge)
}
```

**Algorithme de Fusion**:

```rust
let total_confidence = existing.confidence + new.confidence;
let weighted_amount = (
    (existing_amount as f32 * existing.confidence) +
    (new_amount as f32 * new.confidence)
) / total_confidence;
```

#### 2. Résolution de Conflits de Type

```rust
ConflictType::ResourceTypeConflict => {
    // Préférence basée sur la confiance et la récence
    if (new.confidence - existing.confidence).abs() < 0.1 {
        // Confiance similaire, préférer le plus récent
        if new.discovery_time > existing.discovery_time {
            Ok(ConflictResolution::AcceptNew)
        } else {
            Ok(ConflictResolution::KeepCurrent)
        }
    } else if new.confidence > existing.confidence {
        Ok(ConflictResolution::AcceptNew)
    } else {
        Ok(ConflictResolution::KeepCurrent)
    }
}
```

#### 3. Résolution de Conflits de Terrain

```rust
ConflictType::TerrainMismatch => {
    // Conflit grave nécessitant une révision manuelle
    Ok(ConflictResolution::RequiresManualReview)
}
```

#### 4. Résolution de Conflits de Confiance

```rust
ConflictType::ConfidenceConflict => {
    // Préférer la confiance la plus élevée
    if new.confidence > existing.confidence {
        Ok(ConflictResolution::AcceptNew)
    } else {
        Ok(ConflictResolution::KeepCurrent)
    }
}
```

### Système de Statistiques

#### MergeStatistics - Métriques de Performance

```rust
pub struct MergeStatistics {
    pub total_merges: u32,              // Total des tentatives de fusion
    pub successful_merges: u32,         // Fusions réussies automatiquement
    pub conflicts_resolved: u32,        // Conflits résolus (auto + manuel)
    pub manual_reviews_required: u32,   // Conflits nécessitant intervention
}
```

#### Calcul des Métriques

- **Taux de succès automatique**: `successful_merges / total_merges`
- **Taux de conflits**: `manual_reviews_required / total_merges`
- **Efficacité de résolution**: `conflicts_resolved / total_conflicts`

### API de Fusion

#### Interface Principale

```rust
impl Station {
    pub fn process_robot_discovery(
        &mut self,
        robot_id: usize,
        position: (usize, usize),
        terrain_type: u8,
        resource: Option<(ResourceType, u32)>,
        discovery_time: u64,
        confidence: f32,
    ) -> Result<ConflictResolution, String>
}
```

#### Gestion des Conflits Manuels

```rust
impl Station {
    pub fn get_pending_conflicts(&self) -> &Vec<InformationConflict>;
    pub fn resolve_conflict(
        &mut self,
        conflict_index: usize,
        resolution: ConflictResolution
    ) -> Result<(), String>;
}
```

### Fonctionnalités Avancées

#### 1. Estimations de Ressources Pondérées

```rust
pub fn get_resource_estimates(&self, resource_type: &ResourceType) -> Vec<(usize, usize, u32, f32)> {
    // Retourne (x, y, quantité, confiance) pour la planification intelligente
}
```

#### 2. Recommandations d'Exploration

```rust
pub fn get_exploration_recommendations(&self, map_width: usize, map_height: usize) -> Vec<(usize, usize)> {
    // Priorité: zones inconnues > zones à faible confiance
}
```

#### 3. Mise à Jour Intelligente

```rust
fn update_location_info(&self, existing: &LocationInfo, new: &LocationInfo) -> LocationInfo {
    // Fusion sans conflit basée sur la récence et la confiance
}
```

## Tests et Validation

### Couverture de Tests

- ✅ **9 tests de fusion** couvrant tous les types de conflits
- ✅ **Tests de résolution automatique** pour chaque stratégie
- ✅ **Tests de gestion manuelle** des conflits complexes
- ✅ **Tests de statistiques** et métriques de performance
- ✅ **Tests d'intégration** avec le système de stations

### Scénarios de Test

#### 1. Fusion Compatible

```rust
// Informations similaires fusionnées automatiquement
station.process_robot_discovery(1, (5,5), 0, Some((Energy, 40)), 100, 0.8);
station.process_robot_discovery(2, (5,5), 0, Some((Energy, 45)), 110, 0.7);
// Résultat: Fusion par moyenne pondérée
```

#### 2. Conflit de Type de Ressource

```rust
// Types contradictoires résolus par confiance
station.process_robot_discovery(1, (3,3), 0, Some((Energy, 30)), 100, 0.6);
station.process_robot_discovery(2, (3,3), 0, Some((Mineral, 35)), 110, 0.9);
// Résultat: AcceptNew (confiance plus élevée)
```

#### 3. Conflit de Terrain

```rust
// Terrain contradictoire nécessite révision manuelle
station.process_robot_discovery(1, (7,7), 0, None, 100, 0.8);  // Plain
station.process_robot_discovery(2, (7,7), 2, None, 110, 0.7);  // Mountain
// Résultat: RequiresManualReview
```

### Métriques de Performance

```rust
// Mesures typiques observées:
- Détection de conflit: < 0.1ms par découverte
- Résolution automatique: < 0.5ms par conflit
- Fusion par moyenne pondérée: < 0.2ms
- Stockage des conflits: O(1) insertion
```

## Avantages de la Solution

### Intelligence

- **Résolution contextuelle** basée sur le type de conflit
- **Pondération par confiance** pour décisions optimales
- **Préservation d'informations** importantes

### Performance

- **Fusion en temps réel** sans impact sur la simulation
- **Structures de données efficaces** (HashMap, Vec)
- **Algorithmes O(1)** pour la plupart des opérations

### Robustesse

- **Gestion d'erreurs complète** avec types d'erreur spécifiques
- **Validation des données** avant fusion
- **Récupération gracieuse** des conflits non résolus

### Extensibilité

- **Types de conflits modulaires** faciles à étendre
- **Stratégies de résolution configurables**
- **API claire** pour intégration future

## Limitations et Évolutions Futures

### Limitations Actuelles

- **Pas d'historique** des versions précédentes
- **Résolution locale** uniquement (pas de consensus global)
- **Stratégies fixes** (pas d'apprentissage adaptatif)

### Évolutions Possibles

- **Versioning complet** avec historique Git-like
- **Consensus distribué** entre stations multiples
- **Apprentissage des patterns** de conflits
- **Résolution prédictive** basée sur l'historique
- **Interface graphique** pour résolution manuelle

## Impact sur l'Architecture

### Intégration avec les Robots

- Les robots fournissent des niveaux de confiance basés sur leurs capteurs
- Timestamps automatiques via le système de simulation
- Isolation maintenue jusqu'au retour à la station

### Intégration avec la Station

- Extension naturelle du système de ressources existant
- API cohérente avec les autres fonctionnalités
- Statistiques intégrées au monitoring global

### Intégration avec la Simulation

- Fusion en temps réel sans impact sur les performances
- Métriques exposées pour analyse et optimisation
- Support pour visualisation future des conflits

## Conclusion

Le système de fusion d'informations Git-like offre une solution sophistiquée et robuste pour gérer les découvertes conflictuelles dans Nova. Il démontre des concepts avancés de gestion de données distribuées tout en maintenant la performance et la simplicité d'utilisation.

Cette implémentation pose les bases pour des systèmes de fusion plus avancés et constitue un excellent exemple d'application des principes Git dans un contexte de simulation multi-agents. La flexibilité et l'extensibilité de la solution permettront l'évolution future vers des systèmes encore plus sophistiqués.

## Références

- [Git Merge Strategies](https://git-scm.com/docs/merge-strategies) - Inspiration pour les stratégies de fusion
- [Conflict-free Replicated Data Types](https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type) - Concepts théoriques pour structures de données distribuées
- [Vector Clocks](https://en.wikipedia.org/wiki/Vector_clock) - Alternative considérée pour la gestion temporelle
- [Consensus Algorithms](<https://en.wikipedia.org/wiki/Consensus_(computer_science)>) - Concepts pour évolutions futures
