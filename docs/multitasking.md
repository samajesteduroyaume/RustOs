# Système de Multitâche et Gestion des Processus

## Vue d'ensemble

Le système d'exploitation RustOS implémente maintenant un système de multitâche préemptif avec support de la mémoire virtuelle et de la copie sur écriture (Copy-On-Write).

## Architecture

### 1. Gestion des Processus (`src/process/mod.rs`)

#### Structure Process
- **pid** : Identifiant unique du processus
- **name** : Nom du processus
- **state** : État actuel (Ready, Running, Blocked, Terminated)
- **context** : Contexte d'exécution (registres, pile, etc.)
- **priority** : Priorité du processus
- **kstack** : Pile noyau
- **address_space_id** : Identifiant de l'espace d'adressage
- **cow_pages** : Pages marquées pour la copie sur écriture

#### Opérations principales
- `new()` : Crée un nouveau processus
- `fork()` : Duplique le processus avec CoW
- `save_context()` : Sauvegarde le contexte d'exécution
- `restore_context()` : Restaure le contexte d'exécution

### 2. Planificateur (`src/scheduler/mod.rs`)

#### Politiques de planification
- **RoundRobin** : Tourniquet (par défaut)
- **Priority** : Basée sur la priorité
- **Fifo** : Premier arrivé, premier servi

#### Fonctionnalités
- Changement de contexte préemptif
- Gestion du quantum (temps alloué par processus)
- Sélection du prochain processus à exécuter

### 3. Appels Système (`src/syscall/mod.rs`)

#### Appels système implémentés
- `Exit` : Terminer le processus
- `Fork` : Créer un processus enfant
- `Read` : Lire depuis un descripteur de fichier
- `Write` : Écrire vers un descripteur de fichier
- `Open` : Ouvrir un fichier
- `Close` : Fermer un fichier
- `Exec` : Exécuter un programme
- `Wait` : Attendre la fin d'un processus
- `GetPid` : Obtenir le PID du processus

### 4. Gestion de la Mémoire Virtuelle (`src/memory/vm/mod.rs`)

#### Gestionnaire de cadre physique
- Allocation et libération de cadres physiques
- Suivi des cadres utilisés
- Support pour les régions de mémoire utilisables

#### Espace d'adressage
- Mappage des pages virtuelles aux cadres physiques
- Isolation de la mémoire entre processus
- Support pour la copie sur écriture

### 5. Copie sur Écriture (CoW) (`src/memory/vm/cow.rs`)

#### Principe
Les pages sont initialement partagées en lecture seule lors d'un fork(). Lors d'une tentative d'écriture, une exception de page se produit, déclenchant la duplication de la page.

#### Avantages
- Économie de mémoire lors de fork()
- Performance améliorée
- Transparence pour l'application

#### Implémentation
- Suivi des pages partagées avec compteur de références
- Gestion des défauts de page de protection
- Duplication automatique lors de l'écriture

### 6. Gestionnaire d'Interruptions (`src/interrupts.rs`)

#### Interruptions gérées
- **Timer (32)** : Tick du planificateur
- **Keyboard (33)** : Entrée clavier
- **Page Fault (14)** : Défaut de page

#### Gestionnaire de défaut de page
- Détecte les violations de protection
- Gère la copie sur écriture
- Signale les erreurs non gérées

## Flux d'exécution

### Démarrage du système
1. Initialisation du tas (heap)
2. Initialisation de la table des interruptions (IDT)
3. Activation des interruptions
4. Création du gestionnaire de processus
5. Création du processus initial (init)
6. Initialisation du planificateur
7. Démarrage du planificateur (boucle infinie)

### Changement de contexte
1. Interruption timer
2. Appel du gestionnaire d'interruption
3. Sauvegarde du contexte du processus actuel
4. Sélection du prochain processus
5. Restauration du contexte du nouveau processus
6. Retour du contrôle au nouveau processus

### Gestion de fork()
1. Appel système fork()
2. Création d'un nouvel espace d'adressage
3. Copie des mappages de page (avec CoW)
4. Marquage des pages comme lecture seule
5. Retour du PID enfant au parent (0 à l'enfant)

## État actuel

### Implémenté
- ✅ Structure de base des processus
- ✅ Gestionnaire de processus
- ✅ Planificateur avec round-robin
- ✅ Appels système (stubs)
- ✅ Gestion de la mémoire virtuelle
- ✅ Copie sur écriture (CoW)
- ✅ Gestionnaire d'interruptions

### À implémenter
- ⏳ Synchronisation (sémaphores, mutex)
- ⏳ Files d'attente de messages
- ⏳ Descripteurs de fichiers
- ⏳ Niveaux de privilège (ring 0/3)
- ⏳ Vérification des permissions
- ⏳ Cache TLB
- ⏳ Planification avec priorité dynamique

## Exemple d'utilisation

```rust
// Créer un processus
let mut pm = ProcessManager::new();
let pid = pm.create_process("mon_app", mon_app_main, 1)?;

// Initialiser le planificateur
let scheduler = Scheduler::new(Arc::new(Mutex::new(pm)), SchedulerPolicy::RoundRobin);

// Démarrer le multitâche
scheduler.run(); // Ne retourne jamais
```

## Performance

- **Changement de contexte** : O(1) avec round-robin
- **Allocation de processus** : O(1)
- **Copie sur écriture** : Lazy (à la première écriture)

## Sécurité

- Isolation de la mémoire entre processus
- Vérification des limites de pile
- Gestion des défauts de page
- Support pour les niveaux de privilège (à implémenter)

## Références

- [x86-64 Architecture](https://en.wikipedia.org/wiki/X86-64)
- [Copy-on-Write](https://en.wikipedia.org/wiki/Copy-on-write)
- [Process Management](https://en.wikipedia.org/wiki/Process_(computing))
