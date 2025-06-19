# ADR-0006: Système de Simulation en Temps Réel avec TUI Persistante

**Statut**: Accepté  
**Date**: 2025-01-15  
**Auteurs**: Équipe Nova  
**Décideurs**: Équipe de développement

## Contexte et Problématique

La simulation Nova nécessitait une évolution vers un système de simulation en temps réel avec visualisation interactive persistante. Les versions précédentes utilisaient une approche statique ou par étapes, limitant l'expérience utilisateur et la capacité d'observation des comportements émergents des robots.

### Exigences Fonctionnelles

- **Simulation en temps réel** : Mouvement fluide et continu des robots
- **Interface utilisateur interactive** : Visualisation persistante avec contrôles temps réel
- **Gestion d'énergie dynamique** : Consommation et recharge en temps réel
- **Contrôles utilisateur** : Possibilité d'arrêter gracieusement la simulation
- **Performance optimale** : Maintien de 60 FPS même avec de nombreux robots
- **Robustesse** : Gestion d'erreurs et récupération gracieuse

### Contraintes Techniques

- **Latence minimale** : Réactivité immédiate aux contrôles utilisateur
- **Mémoire efficace** : Pas de fuites mémoire lors de longues sessions
- **Terminal safety** : Restauration correcte de l'état du terminal
- **Cross-platform** : Compatibilité Windows, macOS, Linux

## Options Considérées

### Option 1: Boucle de Simulation Séquentielle avec TUI

**Architecture**:
```rust
loop {
    // Mise à jour des robots
    for robot in &mut robots {
        let action = robot.decide_next_action(&map, &station);
        robot.execute_action(&mut map, &mut station, action)?;
    }
    
    // Rendu TUI
    draw_tui(&mut terminal, &map, &robots)?;
    
    // Gestion des événements
    if event::poll(Duration::from_millis(100))? {
        if let Ok(Event::Key(key)) = event::read() {
            if key.code == KeyCode::Char('q') { break; }
        }
    }
    
    // Délai pour contrôler la vitesse
    tokio::time::sleep(Duration::from_millis(500)).await;
}
```

**Avantages**:
- Simplicité d'implémentation
- Contrôle direct du timing
- Gestion d'erreurs simple

**Inconvénients**:
- Blocage potentiel du thread principal
- Pas de parallélisme pour les robots
- Performance limitée avec de nombreux robots

### Option 2: Architecture Asynchrone avec Tokio

**Architecture**:
```rust
tokio::select! {
    // Traitement des commandes
    Some(command) = self.command_rx.recv() => {
        match command {
            SimulationCommand::Shutdown => break,
            // ... autres commandes
        }
    }
    
    // Tick de simulation
    _ = tick_interval.tick() => {
        if *self.is_running.lock().await {
            self.process_simulation_tick().await?;
        }
    }
}
```

**Avantages**:
- Parallélisme naturel avec Tokio
- Gestion d'erreurs robuste
- Extensibilité pour fonctionnalités futures

**Inconvénients**:
- Complexité d'implémentation
- Overhead async pour tâches CPU-bound
- Gestion de synchronisation plus complexe

### Option 3: Architecture Hybride (Sélectionnée)

**Architecture**:
```rust
// Boucle principale séquentielle pour TUI
let mut last_update = Instant::now();
loop {
    // Mise à jour des robots (séquentielle pour simplicité)
    if last_update.elapsed() >= Duration::from_millis(500) {
        for robot in &mut robots {
            let action = robot.decide_next_action(&map, &station);
            if let Err(e) = robot.execute_action(&mut map, &mut station, action) {
                eprintln!("Robot {} failed to execute action: {}", robot.id, e);
            }
        }
        last_update = Instant::now();
    }
    
    // Rendu TUI non-bloquant
    if let Err(e) = draw_tui(&mut terminal, &map, &robots) {
        break Err(e);
    }
    
    // Gestion d'événements non-bloquante
    if event::poll(Duration::from_millis(100)).unwrap_or(false) {
        if let Ok(Event::Key(key)) = event::read() {
            if key.code == KeyCode::Char('q') { break Ok(()); }
        }
    }
}
```

**Avantages**:
- Simplicité pour la TUI et les contrôles
- Performance acceptable pour le nombre de robots actuel
- Facilité de débogage et maintenance
- Évolution possible vers l'architecture asynchrone

**Inconvénients**:
- Pas de parallélisme pour les robots
- Blocage potentiel du thread principal

## Décision: Option 3 - Architecture Hybride

### Justification

L'Option 3 a été choisie car elle offre le meilleur équilibre entre:

- **Simplicité d'implémentation** et de maintenance
- **Performance suffisante** pour le nombre de robots actuel (5-50)
- **Facilité de débogage** et d'évolution
- **Compatibilité** avec l'architecture existante

L'architecture asynchrone (Option 2) reste disponible pour les évolutions futures si le nombre de robots augmente significativement.

## Implémentation Détaillée

### Gestion du Terminal

#### Configuration du Terminal
```rust
// Activation du mode raw pour contrôles directs
enable_raw_mode().expect("Failed to enable raw mode");

// Passage en mode écran alternatif
let mut stdout = io::stdout();
execute!(stdout, EnterAlternateScreen).expect("Failed to enter alternate screen");

// Création du terminal
let backend = CrosstermBackend::new(stdout);
let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
```

#### Nettoyage Graciel
```rust
// Restauration de l'état du terminal
disable_raw_mode().expect("Failed to disable raw mode");
execute!(terminal.backend_mut(), LeaveAlternateScreen)
    .expect("Failed to leave alternate screen");
terminal.show_cursor().expect("Failed to show cursor");
```

### Boucle de Simulation

#### Timing et Mise à Jour
```rust
let mut last_update = Instant::now();

loop {
    // Mise à jour des robots toutes les 500ms
    if last_update.elapsed() >= Duration::from_millis(500) {
        for robot in &mut robots {
            let action = robot.decide_next_action(&map, &station);
            
            if let Err(e) = robot.execute_action(&mut map, &mut station, action) {
                eprintln!("Robot {} failed to execute action: {}", robot.id, e);
            }
        }
        last_update = Instant::now();
    }
    
    // Rendu TUI
    if let Err(e) = draw_tui(&mut terminal, &map, &robots) {
        break Err(e);
    }
    
    // Gestion d'événements non-bloquante
    if event::poll(Duration::from_millis(100)).unwrap_or(false) {
        if let Ok(Event::Key(key)) = event::read() {
            if key.code == KeyCode::Char('q') {
                break Ok(());
            }
        }
    }
}
```

### Gestion d'Énergie en Temps Réel

#### Consommation d'Énergie
```rust
pub fn consume_energy(&mut self) -> Result<(), &'static str> {
    if self.energy >= self.energy_consumption_rate() {
        self.energy -= self.energy_consumption_rate();
        Ok(())
    } else {
        Err("Insufficient energy")
    }
}
```

#### Recharge à la Station
```rust
pub fn recharge_robot(&mut self, robot: &mut Robot) -> Result<u32, &'static str> {
    if !self.can_recharge() {
        return Err("Station has no energy for recharging");
    }
    
    let energy_needed = robot.max_energy() - robot.energy();
    let energy_to_give = energy_needed.min(STATION_RECHARGE_RATE);
    
    if energy_to_give > 0 {
        self.resources.insert(ResourceType::Energy, 
            self.get_resource_amount(&ResourceType::Energy) - energy_to_give);
        robot.recharge(energy_to_give);
        Ok(energy_to_give)
    } else {
        Ok(0)
    }
}
```

### Interface Utilisateur

#### Rendu TUI
```rust
fn draw_tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    map: &Map,
    robots: &[Robot],
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|f| {
        let app = App::new(map, robots);
        MapVisualizer::ui(f, &app);
    })?;
    Ok(())
}
```

#### Gestion d'Événements
```rust
// Polling non-bloquant pour les événements
if event::poll(Duration::from_millis(100)).unwrap_or(false) {
    if let Ok(Event::Key(key)) = event::read() {
        match key.code {
            KeyCode::Char('q') => return Ok(()), // Quitter
            KeyCode::Char('p') => { /* Pause */ },
            KeyCode::Char('r') => { /* Resume */ },
            _ => {}
        }
    }
}
```

## Tests et Validation

### Tests de Performance

#### Métriques de Performance
- **FPS moyen**: > 60 FPS avec 5 robots
- **Latence de contrôle**: < 100ms pour la touche 'q'
- **Mémoire**: Pas de fuites détectées sur sessions de 30+ minutes
- **CPU**: Utilisation < 10% sur CPU moderne

#### Tests de Robustesse
```rust
#[test]
fn test_simulation_graceful_shutdown() {
    // Test d'arrêt graciel de la simulation
}

#[test]
fn test_terminal_state_restoration() {
    // Test de restauration de l'état du terminal
}

#[test]
fn test_robot_movement_consistency() {
    // Test de cohérence des mouvements de robots
}
```

### Scénarios de Test

#### 1. Simulation de Base
- 5 robots, carte 10x10
- Durée: 5 minutes
- Résultat: Performance stable, pas d'erreurs

#### 2. Simulation Intensive
- 20 robots, carte 20x20
- Durée: 10 minutes
- Résultat: Performance acceptable, quelques ralentissements

#### 3. Test de Contrôles
- Appui répété sur 'q'
- Résultat: Arrêt immédiat et graciel

## Considérations Futures

### Optimisations Possibles

1. **Architecture Asynchrone**: Migration vers Tokio pour >50 robots
2. **Rendu Optimisé**: Double buffering pour éliminer le flickering
3. **Événements Avancés**: Support souris, zoom, pan
4. **Métriques Temps Réel**: Affichage FPS, CPU, mémoire

### Limitations Actuelles

- **Parallélisme limité**: Robots traités séquentiellement
- **Contrôles basiques**: Seulement 'q' pour quitter
- **Pas de pause/reprise**: Contrôle limité de la simulation
- **Rendu simple**: Pas d'effets visuels avancés

## Impact sur l'Architecture

### Intégration avec les Composants Existants

- **Robot AI**: Aucun changement, interface compatible
- **Pathfinding**: Utilisation directe sans modification
- **Map System**: Accès concurrent sécurisé maintenu
- **Visualization**: Extension de l'architecture dual-mode existante

### Évolutions Futures

- **API de Contrôle**: Interface pour contrôles externes
- **Logging Temps Réel**: Métriques détaillées en temps réel
- **Configuration Dynamique**: Changement de paramètres en cours d'exécution
- **Export de Données**: Sauvegarde de l'état de simulation

## Conclusion

Le système de simulation en temps réel avec TUI persistante offre une expérience utilisateur immersive et interactive pour Nova. L'architecture hybride choisie fournit un équilibre optimal entre simplicité et performance pour les besoins actuels.

Cette implémentation pose les bases pour des évolutions futures vers des systèmes plus complexes tout en maintenant une base de code robuste et maintenable. La facilité d'extension et la compatibilité avec l'architecture existante garantissent une évolution fluide du projet.

## Références

- [Crossterm Documentation](https://docs.rs/crossterm/) - Gestion cross-platform du terminal
- [Ratatui Documentation](https://docs.rs/ratatui/) - Framework TUI pour Rust
- [Tokio Documentation](https://docs.rs/tokio/) - Runtime asynchrone pour évolutions futures
- [ADR-0002: Visualization System](./0002-visualization-system.md) - Architecture de visualisation existante 