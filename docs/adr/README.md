# Architecture Decision Records (ADRs)

Ce dossier contient les Architecture Decision Records (ADRs) du projet Nova. Les ADRs documentent les décisions architecturales importantes prises au cours du développement du projet.

## Qu'est-ce qu'un ADR ?

Un Architecture Decision Record (ADR) est un document qui capture une décision architecturale importante, le contexte qui l'a motivée, et les conséquences de cette décision. Il aide à maintenir la traçabilité des décisions et facilite la compréhension du projet par les nouveaux développeurs.

## Structure des ADRs

Chaque ADR suit une structure standardisée :

- **Statut** : Accepté, En cours, Rejeté, etc.
- **Contexte** : Problématique et exigences
- **Options considérées** : Alternatives évaluées
- **Décision** : Solution choisie et justification
- **Conséquences** : Impact sur l'architecture
- **Références** : Liens vers la documentation

## ADRs du Projet Nova

### [ADR-0001: Map Generation System](./0001-map-generation-system.md)
**Statut** : Accepté  
**Date** : 2025-05-07  
**Sujet** : Système de génération de cartes procédurales

Décrit l'implémentation du système de génération de cartes utilisant le bruit de Perlin pour créer des terrains et distribuer des ressources de manière déterministe et reproductible.

**Décisions clés** :
- Utilisation du bruit de Perlin pour la génération de terrain
- Distribution des ressources basée sur des seuils de bruit
- Système de sérialisation/désérialisation pour la persistance
- Gestion des coûts de mouvement selon le type de terrain

### [ADR-0002: Visualization System](./0002-visualization-system.md)
**Statut** : Accepté (Mis à jour)  
**Date** : 2025-05-28 (Initial), 2025-01-15 (Mise à jour)  
**Sujet** : Système de visualisation dual-mode avec TUI temps réel

Documente l'architecture de visualisation qui s'adapte automatiquement à l'environnement d'exécution, offrant une interface TUI interactive ou un mode fallback pour l'automatisation.

**Décisions clés** :
- Architecture dual-mode (TUI interactive / Fallback texte)
- Détection automatique de l'environnement terminal
- Rendu par viewport pour les grandes cartes
- Visualisation temps réel des mouvements de robots

### [ADR-0003: Concurrency Model](./0003-concurrency-model.md)
**Statut** : Accepté (Mis à jour)  
**Date** : 2025-01-15  
**Sujet** : Modèle de concurrence pour la simulation

Décrit l'architecture de concurrence choisie pour gérer les robots autonomes et la simulation en temps réel.

**Décisions clés** :
- Architecture hybride pour la simulation temps réel
- Boucle séquentielle pour TUI et contrôles
- Gestion sécurisée de l'état du terminal
- Évolution possible vers architecture Tokio asynchrone

### [ADR-0004: Robot AI Algorithms](./0004-robot-ai-algorithms.md)
**Statut** : Accepté  
**Date** : 2024-01-15  
**Sujet** : Algorithmes d'intelligence artificielle des robots

Documente les algorithmes de prise de décision et les comportements spécialisés pour chaque type de robot.

**Décisions clés** :
- Système de règles basé sur les priorités
- Comportements spécialisés par type de robot
- Gestion intelligente de l'énergie
- Algorithmes de pathfinding A* optimisés

### [ADR-0005: Information Merging System](./0005-information-merging-system.md)
**Statut** : Accepté  
**Date** : 2024-01-15  
**Sujet** : Système de fusion d'informations Git-like

Décrit le système sophistiqué de fusion d'informations pour gérer les découvertes conflictuelles des robots.

**Décisions clés** :
- Résolution automatique intelligente des conflits
- Détection de différents types de conflits
- Stratégies de fusion basées sur la confiance
- Système de statistiques et métriques

### [ADR-0006: Real-Time Simulation System](./0006-real-time-simulation-system.md)
**Statut** : Accepté  
**Date** : 2025-01-15  
**Sujet** : Système de simulation en temps réel avec TUI persistante

Documente l'implémentation du système de simulation en temps réel avec visualisation interactive persistante.

**Décisions clés** :
- Architecture hybride pour simplicité et performance
- Gestion d'énergie dynamique en temps réel
- Contrôles utilisateur réactifs
- Nettoyage graciel des ressources

## Évolution des ADRs

Les ADRs peuvent être mis à jour au fil du temps pour refléter l'évolution du projet. Les modifications importantes sont documentées dans la section "Change Log" de chaque ADR.

## Utilisation des ADRs

### Pour les Développeurs
- Consultez les ADRs avant de modifier l'architecture
- Créez un nouvel ADR pour les décisions architecturales importantes
- Mettez à jour les ADRs existants si nécessaire

### Pour les Nouveaux Membres
- Lisez les ADRs pour comprendre l'architecture du projet
- Consultez les références pour approfondir les concepts
- Posez des questions sur les décisions documentées

## Création d'un Nouvel ADR

Pour créer un nouvel ADR :

1. Utilisez le template suivant :
```markdown
# ADR-XXXX: Titre de la Décision

**Statut** : En cours  
**Date** : YYYY-MM-DD  
**Auteurs** : Équipe Nova  
**Décideurs** : Équipe de développement

## Contexte et Problématique

## Options Considérées

## Décision

## Conséquences

## Références
```

2. Numérotez l'ADR de manière séquentielle
3. Ajoutez une entrée dans ce README
4. Faites réviser par l'équipe

## Références

- [ADR GitHub](https://adr.github.io/) - Documentation sur les ADRs
- [ADR Tools](https://github.com/joelparkerhenderson/architecture_decision_record) - Outils pour gérer les ADRs
- [Documentation Rust](https://doc.rust-lang.org/) - Référence pour les concepts Rust 