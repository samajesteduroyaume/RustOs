# 📚 Documentation RustOS v1.2.0

Bienvenue dans la documentation complète de RustOS. Ce répertoire contient tous les guides, références API, et documentation architecturale du projet.

## 🗂️ Organisation

### [guides/](guides/) - Guides d'Utilisation
Guides pratiques pour utiliser et développer avec RustOS :
- **shell-guide.md** - Guide complet du shell
- **network-guide.md** - Guide de la pile réseau
- **driver-guide.md** - Guide des drivers
- **ring3-guide.md** - Guide du mode utilisateur (Ring 3)

### [api/](api/) - Documentation API
Références complètes des APIs disponibles :
- **shell-api.md** - API Shell
- **libc-api.md** - API Librairie Standard
- **network-api.md** - API Réseau
- **driver-api.md** - API Drivers
- **syscall-api.md** - API des appels système

### [architecture/](architecture/) - Architecture Système
Documentation de l'architecture interne :
- **overview.md** - Vue d'ensemble du système
- **kernel.md** - Architecture du noyau
- **memory.md** - Gestion de la mémoire
- **ring3-architecture.md** - Architecture du mode utilisateur (Ring 3)
- **networking.md** - Architecture réseau

### [proposals/](proposals/) - Propositions et Planification
Propositions initiales et planification du projet :
- **stack-proposal.md** - Propositions de pile logicielle
- **roadmap.md** - Feuille de route complète
- **comparison.md** - Comparaison avec autres OS

### [summaries/](summaries/) - Résumés par Phase
Résumés détaillés de chaque phase d'implémentation :
- **phase1-summary.md** - Résumé Phase 1 (Fondations)
- **phase2-summary.md** - Résumé Phase 2 (Drivers)
- **phase3-summary.md** - Résumé Phase 3 (Réseau)
- **phase4-summary.md** - Résumé Phase 4 (Optimisation)
- **project-summary.md** - Résumé complet du projet

### [archived/](archived/) - Documentation Archivée
Documentation obsolète ou dépréciée :
- Anciennes propositions
- Fichiers dépréciés
- Documentation obsolète

---

## 🎯 Point de Départ

### Pour les Utilisateurs
1. Lire [../README.md](../README.md) - Vue d'ensemble
2. Consulter [guides/shell-guide.md](guides/shell-guide.md) - Guide du shell
3. Parcourir [guides/network-guide.md](guides/network-guide.md) - Guide réseau

### Pour les Développeurs
1. Lire [architecture/overview.md](architecture/overview.md) - Vue d'ensemble
2. Étudier [architecture/kernel.md](architecture/kernel.md) - Architecture noyau
3. Consulter [api/](api/) - Documentation API

### Pour Comprendre le Projet
1. Lire [proposals/roadmap.md](proposals/roadmap.md) - Feuille de route
2. Consulter [summaries/project-summary.md](summaries/project-summary.md) - Résumé complet
3. Vérifier [../CHANGELOG.md](../CHANGELOG.md) - Historique

---

## 📊 Contenu

### Guides (3 fichiers)
- Shell : Commandes, utilisation, exemples
- Réseau : Protocoles, utilitaires, configuration
- Drivers : Architecture, implémentation, extension

### API (4 fichiers)
- Shell API : Structures, fonctions, exemples
- libc API : Fonctions standard, prototypes
- Réseau API : Structures, protocoles, sockets
- Drivers API : Trait Driver, implémentation

### Architecture (4 fichiers)
- Vue d'ensemble : Couches, modules, flux
- Noyau : Processus, mémoire, interruptions
- Mémoire : Allocation, paging, CoW
- Réseau : Pile TCP/IP, protocoles

### Propositions (3 fichiers)
- Propositions : Pile logicielle complète
- Roadmap : Phases, calendrier, objectifs
- Comparaison : Linux, Windows, macOS

### Résumés (5 fichiers)
- Phase 1 : Fondations (Shell, Terminal, libc)
- Phase 2 : Drivers (Matériels, Réseau)
- Phase 3 : Réseau (IPv4, TCP, UDP, DNS)
- Phase 4 : Optimisation (Utilitaires, Finition)
- Complet : Résumé global du projet

---

## 🔍 Recherche Rapide

### Par Sujet

**Shell**
- [guides/shell-guide.md](guides/shell-guide.md)
- [api/shell-api.md](api/shell-api.md)

**Réseau**
- [guides/network-guide.md](guides/network-guide.md)
- [api/network-api.md](api/network-api.md)
- [architecture/networking.md](architecture/networking.md)

**Drivers**
- [guides/driver-guide.md](guides/driver-guide.md)
- [api/driver-api.md](api/driver-api.md)

**Noyau**
- [architecture/kernel.md](architecture/kernel.md)
- [architecture/memory.md](architecture/memory.md)

**Projet**
- [proposals/roadmap.md](proposals/roadmap.md)
- [summaries/project-summary.md](summaries/project-summary.md)

---

## 📈 Statistiques

```
Documentation Totale: 2000+ lignes
Guides: 3 fichiers
API: 4 fichiers
Architecture: 4 fichiers
Propositions: 3 fichiers
Résumés: 5 fichiers
```

---

## 🚀 Utilisation

### Compilation
```bash
cd ../mini-os
cargo build --release
```

### Tests
```bash
cd ../mini-os
cargo test
```

### Documentation
Tous les fichiers sont en Markdown et peuvent être lus avec n'importe quel éditeur de texte.

---

## 📝 Conventions

### Fichiers de Documentation
- `*-guide.md` - Guides pratiques
- `*-api.md` - Documentation API
- `*-summary.md` - Résumés
- `*-proposal.md` - Propositions

### Structure des Sections
- **Vue d'ensemble** - Introduction
- **Contenu** - Détails techniques
- **Exemples** - Cas d'utilisation
- **Références** - Liens utiles

---

## 🔗 Liens Utiles

- [README Principal](../README.md)
- [Index de Documentation](../DOCUMENTATION_INDEX.md)
- [Structure du Projet](../PROJECT_STRUCTURE.md)
- [CHANGELOG](../CHANGELOG.md)

---

**Version**: RustOS v1.0.0
**Date**: 6 Décembre 2025
**Statut**: ✅ Documentation Organisée

