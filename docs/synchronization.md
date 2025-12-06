# Synchronisation et Primitives de Synchronisation

## Vue d'ensemble

Le système d'exploitation RustOS fournit plusieurs primitives de synchronisation pour permettre la coordination entre processus et éviter les conditions de course.

## Primitives de Synchronisation

### 1. Sémaphore

Un sémaphore est un compteur qui peut être incrémenté ou décrémenté de manière atomique.

#### Opérations
- **wait(pid)** : Décrémente le sémaphore. Si la valeur est négative, le processus est bloqué.
- **signal()** : Incrémente le sémaphore et réveille un processus en attente.

#### Utilisation
```rust
let sem = Semaphore::new(1); // Sémaphore binaire

// Processus 1
sem.wait(pid1)?;
// Section critique
sem.signal()?;

// Processus 2
sem.wait(pid2)?;
// Section critique
sem.signal()?;
```

### 2. Mutex (Mutual Exclusion)

Un mutex assure que seul un processus peut accéder à une ressource à la fois.

#### Opérations
- **lock(pid)** : Acquiert le mutex. Si déjà verrouillé, le processus est bloqué.
- **unlock(pid)** : Libère le mutex et réveille un processus en attente.
- **is_locked()** : Vérifie si le mutex est verrouillé.

#### Utilisation
```rust
let mutex = MutexLock::new();

// Processus 1
mutex.lock(pid1)?;
// Section critique
mutex.unlock(pid1)?;

// Processus 2
mutex.lock(pid2)?;
// Section critique
mutex.unlock(pid2)?;
```

### 3. Variable de Condition

Une variable de condition permet aux processus d'attendre jusqu'à ce qu'une condition soit satisfaite.

#### Opérations
- **wait(pid, mutex)** : Attend sur la variable de condition et libère le mutex.
- **signal()** : Réveille un processus en attente.
- **broadcast()** : Réveille tous les processus en attente.

#### Utilisation
```rust
let cond = ConditionVariable::new();
let mutex = MutexLock::new();

// Producteur
mutex.lock(producer_pid)?;
// Produire une donnée
cond.signal()?;
mutex.unlock(producer_pid)?;

// Consommateur
mutex.lock(consumer_pid)?;
while !data_available {
    cond.wait(consumer_pid, &mutex)?;
}
// Consommer la donnée
mutex.unlock(consumer_pid)?;
```

### 4. Barrière

Une barrière synchronise plusieurs processus à un point donné.

#### Opérations
- **wait(pid)** : Attend à la barrière. Retourne quand tous les processus sont arrivés.

#### Utilisation
```rust
let barrier = Barrier::new(4); // Attendre 4 processus

// Chaque processus appelle
barrier.wait(pid)?; // Bloqué jusqu'à ce que tous les 4 soient arrivés
```

## Gestionnaire de Descripteurs de Fichiers

### Structure

Le gestionnaire de descripteurs de fichiers (`fs/fd.rs`) gère l'accès aux fichiers pour chaque processus.

### Opérations

- **open(path, mode, size)** : Ouvre un fichier et retourne un descripteur
- **close(fd)** : Ferme un descripteur
- **get(fd)** : Obtient un descripteur (lecture seule)
- **get_mut(fd)** : Obtient un descripteur mutable
- **dup2(old_fd, new_fd)** : Duplique un descripteur

### Modes d'ouverture

- **ReadOnly** : Lecture seule
- **WriteOnly** : Écriture seule
- **ReadWrite** : Lecture et écriture

### Descripteurs réservés

- **0** : stdin (entrée standard)
- **1** : stdout (sortie standard)
- **2** : stderr (sortie d'erreur)

### Utilisation

```rust
let mut table = FileDescriptorTable::new();

// Ouvrir un fichier
let fd = table.open("/test.txt", OpenMode::ReadOnly, 1024)?;

// Obtenir le descripteur
let descriptor = table.get(fd)?;
println!("Fichier: {}", descriptor.path);

// Fermer le fichier
table.close(fd)?;
```

## Patterns de Synchronisation Courants

### 1. Producteur-Consommateur

```rust
let buffer = Arc::new(Mutex::new(Vec::new()));
let cond = Arc::new(ConditionVariable::new());

// Producteur
loop {
    let item = produce();
    let mut buf = buffer.lock();
    buf.push(item);
    cond.signal()?;
}

// Consommateur
loop {
    let mut buf = buffer.lock();
    while buf.is_empty() {
        cond.wait(pid, &buf)?;
    }
    let item = buf.pop().unwrap();
}
```

### 2. Lecteurs-Rédacteurs

```rust
let data = Arc::new(Mutex::new(Data::new()));
let read_count = Arc::new(Mutex::new(0));
let write_lock = Arc::new(MutexLock::new());

// Lecteur
let mut count = read_count.lock();
*count += 1;
if *count == 1 {
    write_lock.lock(pid)?;
}
drop(count);

// Lire les données
let d = data.lock();
// Utiliser d

drop(d);
let mut count = read_count.lock();
*count -= 1;
if *count == 0 {
    write_lock.unlock(pid)?;
}

// Rédacteur
write_lock.lock(pid)?;
let mut d = data.lock();
// Modifier d
drop(d);
write_lock.unlock(pid)?;
```

### 3. Rendez-vous

```rust
let barrier = Arc::new(Barrier::new(n));

// Chaque processus
barrier.wait(pid)?;
// Tous les processus sont maintenant synchronisés
```

## État actuel

### Implémenté
- ✅ Sémaphore
- ✅ Mutex
- ✅ Variable de condition
- ✅ Barrière
- ✅ Gestionnaire de descripteurs de fichiers

### À implémenter
- ⏳ Intégration avec le planificateur pour bloquer/réveiller les processus
- ⏳ Détection de deadlock
- ⏳ Timeouts sur les opérations de synchronisation
- ⏳ Priorité d'héritage pour les mutex

## Performance

- **Acquisition de mutex** : O(1)
- **Libération de mutex** : O(1) ou O(n) si plusieurs processus attendent
- **Signal sur condition** : O(1)
- **Broadcast sur condition** : O(n) où n est le nombre de processus en attente

## Sécurité

- Prévention des conditions de course
- Prévention des accès concurrents
- Détection des violations de propriété (un processus ne peut libérer qu'un mutex qu'il possède)

## Références

- [Semaphore](https://en.wikipedia.org/wiki/Semaphore_(programming))
- [Mutex](https://en.wikipedia.org/wiki/Lock_(computer_science))
- [Condition Variable](https://en.wikipedia.org/wiki/Monitor_(synchronization))
- [Barrier](https://en.wikipedia.org/wiki/Barrier_(computer_science))
