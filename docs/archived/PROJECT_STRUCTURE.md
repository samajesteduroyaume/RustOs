# 📁 Structure du Projet RustOS v1.0.0

## Vue d'ensemble

```
RustOS/
├── mini-os/                          # Code source du système d'exploitation
│   ├── src/
│   │   ├── shell/                   # Shell Bash Minimal
│   │   ├── terminal/                # Terminal/Console
│   │   ├── libc/                    # Librairie Standard
│   │   ├── drivers/                 # Drivers Matériels
│   │   ├── network/                 # Pile Réseau
│   │   ├── process/                 # Gestion des Processus (v0.2.0)
│   │   ├── scheduler/               # Planificateur (v0.2.0)
│   │   ├── sync/                    # Synchronisation (v0.2.0)
│   │   ├── memory/                  # Gestion de la Mémoire
│   │   ├── interrupts.rs            # Gestionnaire d'Interruptions
│   │   ├── vga_buffer.rs            # Buffer VGA
│   │   ├── keyboard.rs              # Driver Clavier
│   │   ├── main.rs                  # Point d'entrée
│   │   └── lib.rs                   # Librairie
│   ├── Cargo.toml                   # Configuration Cargo
│   └── build.sh                     # Script de compilation
│
├── docs/                            # Documentation
│   ├── README.md                    # Index de documentation
│   ├── guides/                      # Guides d'utilisation
│   │   ├── shell-guide.md          # Guide du Shell
│   │   ├── network-guide.md        # Guide Réseau
│   │   └── driver-guide.md         # Guide des Drivers
│   ├── api/                         # Documentation API
│   │   ├── shell-api.md            # API Shell
│   │   ├── libc-api.md             # API libc
│   │   ├── network-api.md          # API Réseau
│   │   └── driver-api.md           # API Drivers
│   ├── architecture/                # Documentation Architecture
│   │   ├── overview.md             # Vue d'ensemble
│   │   ├── kernel.md               # Architecture Noyau
│   │   ├── memory.md               # Gestion Mémoire
│   │   └── networking.md           # Architecture Réseau
│   ├── proposals/                   # Propositions et Planification
│   │   ├── stack-proposal.md       # Propositions de Pile
│   │   ├── roadmap.md              # Feuille de Route
│   │   └── comparison.md           # Comparaison avec autres OS
│   ├── summaries/                   # Résumés par Phase
│   │   ├── phase1-summary.md       # Phase 1 Résumé
│   │   ├── phase2-summary.md       # Phase 2 Résumé
│   │   ├── phase3-summary.md       # Phase 3 Résumé
│   │   ├── phase4-summary.md       # Phase 4 Résumé
│   │   └── project-summary.md      # Résumé Complet
│   └── archived/                    # Documentation Archivée
│       ├── old-proposals.md        # Anciennes Propositions
│       └── deprecated.md           # Fichiers Dépréciés
│
├── README.md                        # Point d'entrée principal
├── CHANGELOG.md                     # Historique des modifications
├── DOCUMENTATION_INDEX.md           # Index de documentation
├── PROJECT_STRUCTURE.md             # Ce fichier
├── LICENSE                          # Licence MIT
│
└── build.sh                         # Script de compilation global

```

---

## 📂 Détails des Répertoires

### `/mini-os/src/` - Code Source

#### Modules Principaux

| Module | Fichier | Lignes | Description |
|--------|---------|--------|-------------|
| Shell | `shell/mod.rs` | 500 | Shell Bash Minimal |
| Terminal | `terminal/mod.rs` | 400 | Terminal/Console |
| libc | `libc/{stdio,stdlib,string}.rs` | 650 | Librairie Standard |
| Drivers | `drivers/{mod,disk,network}.rs` | 950 | Drivers Matériels |
| Network | `network/{ipv4,icmp,udp,tcp,dns,tools}.rs` | 1100 | Pile Réseau |
| Process | `process/mod.rs` | 300 | Gestion des Processus |
| Scheduler | `scheduler/mod.rs` | 200 | Planificateur |
| Sync | `sync/mod.rs` | 300 | Synchronisation |

#### Fichiers Système

| Fichier | Description |
|---------|-------------|
| `main.rs` | Point d'entrée du noyau |
| `lib.rs` | Exports de librairie |
| `interrupts.rs` | Gestionnaire d'interruptions |
| `vga_buffer.rs` | Buffer VGA |
| `keyboard.rs` | Driver clavier |
| `memory.rs` | Gestion mémoire |
| `paging.rs` | Pagination |

### `/docs/` - Documentation

#### `/docs/guides/` - Guides d'Utilisation
- **shell-guide.md** - Guide complet du shell avec exemples
- **network-guide.md** - Guide de la pile réseau
- **driver-guide.md** - Guide des drivers

#### `/docs/api/` - Documentation API
- **shell-api.md** - Référence API Shell
- **libc-api.md** - Référence API libc
- **network-api.md** - Référence API Réseau
- **driver-api.md** - Référence API Drivers

#### `/docs/architecture/` - Architecture
- **overview.md** - Vue d'ensemble du système
- **kernel.md** - Architecture du noyau
- **memory.md** - Gestion de la mémoire
- **networking.md** - Architecture réseau

#### `/docs/proposals/` - Propositions
- **stack-proposal.md** - Propositions de pile logicielle
- **roadmap.md** - Feuille de route du projet
- **comparison.md** - Comparaison avec Linux, Windows, macOS

#### `/docs/summaries/` - Résumés
- **phase1-summary.md** - Résumé Phase 1
- **phase2-summary.md** - Résumé Phase 2
- **phase3-summary.md** - Résumé Phase 3
- **phase4-summary.md** - Résumé Phase 4
- **project-summary.md** - Résumé complet du projet

#### `/docs/archived/` - Archives
- Anciennes propositions
- Fichiers dépréciés
- Documentation obsolète

### Fichiers Racine

| Fichier | Description |
|---------|-------------|
| `README.md` | Point d'entrée principal du projet |
| `CHANGELOG.md` | Historique complet des modifications |
| `DOCUMENTATION_INDEX.md` | Index de navigation de la documentation |
| `PROJECT_STRUCTURE.md` | Ce fichier - Structure du projet |
| `LICENSE` | Licence MIT |

---

## 🎯 Navigation

### Pour Commencer
1. Lire [README.md](README.md)
2. Consulter [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)
3. Parcourir [docs/README.md](docs/README.md)

### Pour Développer
1. Consulter [docs/guides/](docs/guides/)
2. Lire [docs/api/](docs/api/)
3. Étudier [docs/architecture/](docs/architecture/)

### Pour Comprendre le Projet
1. Lire [docs/proposals/](docs/proposals/)
2. Consulter [docs/summaries/](docs/summaries/)
3. Vérifier [CHANGELOG.md](CHANGELOG.md)

---

## 📊 Statistiques

### Code Source
- **Total** : 6400 lignes
- **Modules** : 15
- **Structures** : 24
- **Fonctions** : 170+
- **Tests** : 70

### Documentation
- **Guides** : 3 fichiers
- **API** : 4 fichiers
- **Architecture** : 4 fichiers
- **Propositions** : 3 fichiers
- **Résumés** : 5 fichiers
- **Total** : 2000+ lignes

---

## 🔄 Flux de Travail

### Compilation
```bash
cd mini-os
cargo build --release
```

### Tests
```bash
cd mini-os
cargo test
```

### Documentation
Tous les fichiers de documentation sont dans `/docs/`

---

## 📝 Conventions

### Nommage des Fichiers

| Pattern | Utilisation |
|---------|-------------|
| `*-guide.md` | Guides d'utilisation |
| `*-api.md` | Documentation API |
| `*-summary.md` | Résumés |
| `PHASE*_*.md` | Documentation par phase |
| `*_COMPLETE.txt` | Résumés visuels |

### Structure des Répertoires

- `/docs/guides/` - Guides pratiques
- `/docs/api/` - Documentation API
- `/docs/architecture/` - Architecture système
- `/docs/proposals/` - Propositions et planification
- `/docs/summaries/` - Résumés et synthèses
- `/docs/archived/` - Documentation archivée

---

## 🚀 Prochaines Étapes

1. **Organiser la documentation** dans `/docs/`
2. **Créer les guides** dans `/docs/guides/`
3. **Documenter les APIs** dans `/docs/api/`
4. **Archiver** les anciens fichiers dans `/docs/archived/`

---

**Version**: RustOS v1.0.0
**Date**: 6 Décembre 2025
**Statut**: ✅ Structure Définie

