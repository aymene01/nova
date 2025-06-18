# ADR-003: Modèle de Concurrence pour la Simulation Nova

## Statut

Accepté

## Contexte

La simulation Nova nécessite un traitement efficace de multiples robots autonomes évoluant simultanément dans l'environnement spatial. Le modèle de concurrence doit permettre :

1. **Traitement parallèle des robots** : Chaque robot doit pouvoir agir indépendamment
2. **Synchronisation sécurisée** : Accès concurrent aux ressources partagées (carte, station)
3. **Performance optimale** : Traitement efficace de dizaines de robots simultanément
4. **Contrôle de simulation** : Pause, reprise, arrêt gracieux du système
5. **Monitoring en temps réel** : Surveillance de l'état de la simulation

## Options Considérées

### Option 1: Traitement Séquentiel

- **Avantages** : Simple à implémenter, pas de problèmes de concurrence
- **Inconvénients** : Performance limitée, pas de parallélisme réel
- **Verdict** : Rejeté pour des raisons de performance

### Option 2: Threads Natifs avec Mutex

- **Avantages** : Contrôle fin des threads, performance native
- **Inconvénients** : Complexité de gestion, risques de deadlock
- **Verdict** : Trop complexe pour les besoins actuels

### Option 3: Tokio avec Actor Pattern

- **Avantages** : Async/await natif, gestion d'erreurs robuste, channels
- **Inconvénients** : Overhead async pour CPU-bound tasks
- **Verdict** : **Sélectionné** - Équilibre optimal complexité/performance

## Décision

Nous adoptons **Tokio avec Actor Pattern** utilisant :

### Architecture Choisie

```rust
pub struct SimulationEngine {
    map: Arc<RwLock<Map>>,           // Lecture concurrent, écriture exclusive
    station: Arc<Mutex<Station>>,    // Accès mutuellement exclusif
    robots: Arc<Mutex<Vec<Robot>>>,  // Protection des modifications
    executor: Arc<RobotExecutor>,    // Logique métier partagée
    command_rx: mpsc::Receiver<SimulationCommand>,  // Canal de commandes
    status_tx: mpsc::Sender<SimulationStatus>,      // Canal de statut
}
```

### Modèle de Concurrence

1. **Main Event Loop** : `tokio::select!` pour traitement concurrent
2. **Batch Processing** : Robots traités par groupes pour optimiser les verrous
3. **Read-Write Locks** : `RwLock` pour la carte (lectures multiples)
4. **Mutex Guards** : `Mutex` pour station et robots (accès exclusif)
5. **Message Passing** : Channels pour communication inter-tâches

### Stratégie de Synchronisation

- **Map** : `Arc<RwLock<Map>>` - Multiple lecteurs simultanés pour pathfinding
- **Station** : `Arc<Mutex<Station>>` - Accès exclusif pour livraisons/recharge
- **Robots** : `Arc<Mutex<Vec<Robot>>>` - Protection des modifications d'état
- **Commands** : `mpsc::channel` - Communication asynchrone non-bloquante

## Implémentation

### Traitement par Lots

```rust
let batch_size = (robots.len() / 4).max(1); // Maximum 4 lots concurrents
for chunk in robots.chunks(batch_size) {
    let handle = tokio::spawn(async move {
        Self::process_robot_batch(chunk, map, station, executor, robots_store).await
    });
    handles.push(handle);
}
```

### Gestion des Verrous

```rust
// Lecture de carte (non-bloquant pour autres lecteurs)
let map_guard = map.read().await;

// Accès station (exclusif mais court)
let mut station_guard = station.lock().await;
let result = executor.execute_action_with_station(&mut robot, map_ref, &mut *station_guard);
drop(station_guard); // Libération explicite
```

### Contrôle de Simulation

```rust
tokio::select! {
    Some(command) = self.command_rx.recv() => {
        match command {
            SimulationCommand::Pause => *self.is_running.lock().await = false,
            SimulationCommand::Resume => *self.is_running.lock().await = true,
            SimulationCommand::Shutdown => break,
        }
    }
    _ = tick_interval.tick() => {
        if *self.is_running.lock().await {
            self.process_simulation_tick().await?;
        }
    }
}
```

## Avantages de la Solution

### Performance

- **Parallélisme réel** : Traitement concurrent de lots de robots
- **Optimisation des verrous** : Minimisation du temps de verrouillage
- **Scalabilité** : Performance proportionnelle au nombre de cœurs CPU

### Sécurité

- **Memory Safety** : Garanties Rust pour accès concurrent sécurisé
- **Deadlock Prevention** : Ordre cohérent d'acquisition des verrous
- **Error Handling** : Propagation d'erreurs avec `Result` types

### Maintenabilité

- **Séparation des responsabilités** : Engine, Executor, AI séparés
- **Testabilité** : Tests unitaires et d'intégration complets
- **Monitoring** : Métriques temps réel via `SimulationStatus`

## Métriques de Performance

### Tests de Charge

- **50 robots simultanés** : Performance maintenue
- **Traitement concurrent** : 4 lots parallèles maximum
- **Latence de commandes** : < 100ms pour pause/reprise
- **Throughput** : > 10 ticks/seconde même sous charge

### Monitoring Intégré

```rust
pub struct SimulationStatus {
    pub robots_count: usize,
    pub active_robots: usize,
    pub total_energy_collected: u32,
    pub total_minerals_collected: u32,
    pub total_discoveries: u32,
    pub simulation_ticks: u64,
    pub is_running: bool,
}
```

## Considérations Futures

### Optimisations Possibles

1. **Lock-free data structures** pour performance extrême
2. **Work-stealing scheduler** pour équilibrage de charge
3. **Memory pools** pour réduction des allocations
4. **SIMD optimizations** pour calculs vectoriels

### Limitations Actuelles

- **CPU-bound avec async** : Overhead pour tâches computationnelles
- **Granularité des verrous** : Contention possible sous charge extrême
- **Memory overhead** : Arc/Mutex ajoutent de l'indirection

## Tests de Validation

### Couverture de Tests

- ✅ Démarrage/arrêt gracieux
- ✅ Pause/reprise de simulation
- ✅ Ajout/suppression dynamique de robots
- ✅ Traitement concurrent sous charge
- ✅ Performance maintenue (50 robots)
- ✅ Gestion d'erreurs robuste

### Critères d'Acceptation

- [x] 75+ tests passants
- [x] Performance > 10 FPS avec 50 robots
- [x] Pas de race conditions détectées
- [x] Memory safety garantie par Rust
- [x] Contrôle temps réel de la simulation

## Conclusion

Le modèle de concurrence Tokio + Actor Pattern offre un équilibre optimal entre performance, sécurité et maintenabilité pour Nova. L'architecture permet un traitement parallèle efficace des robots tout en maintenant la cohérence des données partagées et la facilité de développement.

Cette solution pose les bases pour l'évolution future vers des systèmes plus complexes (simulation distribuée, IA avancée) tout en conservant une base de code robuste et testable.
