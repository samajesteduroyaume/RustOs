# API des Appels Système (Syscalls)

Ce document décrit l'interface des appels système disponibles pour les applications s'exécutant en mode utilisateur (Ring 3) sous RustOS.

## 📋 Table des Matières
- [Vue d'ensemble](#-vue-densemble)
- [Convention d'appel](#-convention-dappel)
- [Liste des Appels Système](#-liste-des-appels-système)
  - [Gestion des Processus](#gestion-des-processus)
  - [Gestion de la Mémoire](#gestion-de-la-mémoire)
  - [Gestion des Fichiers](#gestion-des-fichiers)
  - [Entrées/Sorties](#entréessorties)
  - [Communication Inter-Processus](#communication-inter-processus)
  - [Système](#système)
- [Codes d'Erreur](#-codes-derreur)
- [Exemples](#-exemples)

## 🌐 Vue d'ensemble

Les appels système fournissent une interface sécurisée pour les applications utilisateur afin d'accéder aux fonctionnalités du noyau. Chaque appel système est identifié par un numéro unique et peut prendre jusqu'à 6 paramètres.

## 📝 Convention d'Appel

Les appels système utilisent la convention d'appel suivante :

```rust
// En C
long syscall(long number, ...);

// En assembleur x86_64
// rax = numéro de l'appel système
// rdi, rsi, rdx, r10, r8, r9 = arguments
// syscall
// rax contient la valeur de retour
// rcx et r11 sont écrasés
```

## 📚 Liste des Appels Système

### Gestion des Processus

#### `exit` (1)
Termine le processus appelant.

**Signature :**
```c
void exit(int status);
```

**Paramètres :**
- `status` : Code de sortie du processus

**Valeur de retour :**
- Ne retourne pas en cas de succès

**Erreurs possibles :**
- Aucune (toujours réussi)

---

#### `fork` (2)
Crée un nouveau processus en dupliquant le processus appelant.

**Signature :**
```c
pid_t fork(void);
```

**Valeur de retour :**
- `> 0` : PID du processus fils (dans le processus parent)
- `0` : Dans le processus fils
- `-1` : En cas d'échec

**Erreurs possibles :**
- `EAGAIN` : Trop de processus
- `ENOMEM` : Mémoire insuffisante

---

### Gestion de la Mémoire

#### `mmap` (9)
Mappe des fichiers ou des périphériques en mémoire.

**Signature :**
```c
void *mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset);
```

**Paramètres :**
- `addr` : Adresse de départ (peut être NULL)
- `length` : Taille de la zone à mapper
- `prot` : Protection (PROT_READ, PROT_WRITE, etc.)
- `flags` : Options de mappage
- `fd` : Descripteur de fichier
- `offset` : Décalage dans le fichier

**Valeur de retour :**
- `> 0` : Adresse du mappage
- `MAP_FAILED` : En cas d'échec

---

### Gestion des Fichiers

#### `open` (2)
Ouvre un fichier ou un périphérique.

**Signature :**
```c
int open(const char *pathname, int flags, mode_t mode);
```

**Paramètres :**
- `pathname` : Chemin du fichier
- `flags` : Options d'ouverture
- `mode` : Droits d'accès

**Valeur de retour :**
- `>= 0` : Descripteur de fichier
- `-1` : En cas d'échec

---

#### `read` (0)
Lit depuis un descripteur de fichier.

**Signature :**
```c
ssize_t read(int fd, void *buf, size_t count);
```

**Paramètres :**
- `fd` : Descripteur de fichier
- `buf` : Tampon de destination
- `count` : Nombre d'octets à lire

**Valeur de retour :**
- `> 0` : Nombre d'octets lus
- `0` : Fin de fichier
- `-1` : En cas d'échec

---

## 📝 Codes d'Erreur

| Constante | Valeur | Description |
|-----------|--------|-------------|
| `EPERM`   | 1      | Opération non permise |
| `ENOENT`  | 2      | Fichier ou répertoire inexistant |
| `EINTR`   | 4      | Appel système interrompu |
| `EIO`     | 5      | Erreur d'E/S |
| `ENOMEM`  | 12     | Mémoire insuffisante |
| `EACCES`  | 13     | Permission refusée |
| `EFAULT`  | 14     | Adresse invalide |
| `EBUSY`   | 16     | Ressource occupée |
| `EEXIST`  | 17     | Le fichier existe déjà |
| `ENODEV`  | 19     | Périphérique inexistant |
| `EINVAL`  | 22     | Argument invalide |
| `ENFILE`  | 23     | Trop de fichiers ouverts dans le système |
| `EMFILE`  | 24     | Trop de fichiers ouverts par le processus |

## 🚀 Exemples

### Exemple d'ouverture et lecture d'un fichier

```c
#include <fcntl.h>
#include <unistd.h>
#include <sys/syscall.h>

int main() {
    char buffer[1024];
    int fd = open("/etc/passwd", O_RDONLY);
    if (fd < 0) {
        // Gestion d'erreur
        return 1;
    }
    
    ssize_t bytes_read = read(fd, buffer, sizeof(buffer) - 1);
    if (bytes_read > 0) {
        buffer[bytes_read] = '\0';
        // Traiter le contenu
    }
    
    close(fd);
    return 0;
}
```

### Création d'un processus fils

```c
#include <unistd.h>
#include <sys/wait.h>

int main() {
    pid_t pid = fork();
    if (pid == -1) {
        // Erreur
        return 1;
    } else if (pid == 0) {
        // Code du fils
        _exit(0);
    } else {
        // Code du parent
        wait(NULL);
    }
    return 0;
}
```

## 📚 Voir Aussi

- [Guide du mode utilisateur](../guides/ring3-guide.md)
- [Architecture du noyau](../architecture/kernel.md)
- [Gestion de la mémoire](../architecture/memory.md)
