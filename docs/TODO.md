# Liste des Tâches Techniques (TODO)

Ce fichier recense les fonctionnalités manquantes, les dettes techniques et les stubs (bouchons) identifiés lors de l'audit du code source (07/12/2025).

## 🚨 Priorité Haute (Fixes & Intégration)

### Réseau (Network Stack)
- [ ] **Unification de la Pile** : Le projet contient deux modules réseau conflictuels :
  - `src/net/` (Semble plus complet, ~10KB TCP).
  - `src/network/` (Utilisé par `main.rs`, mais contient des STUBS "TODO: Envoyer SYN").
  - **Action** : Migrer `main.rs` pour utiliser `src/net`, supprimer `src/network`, et vérifier le fonctionnement réel TCP.
- [ ] **Implémentation TCP** : Remplacer les stubs de `connect`/`accept` par la machine à états réelle.
- [ ] **Checksum UDP** : Implémenter le calcul du checksum dans `udp.rs`.

### Gestionnaire de Périphériques (Device Manager)
- [ ] **PCI Enumerator** : Le fichier `src/device_manager/pci.rs` contient des TODOs critiques.
  - `read_config` retourne `0`. L'énumération via ce module est donc inopérante.
  - **Action** : Importer la logique fonctionnelle de `src/hardware.rs` (Legacy) vers `src/device_manager/pci.rs`.

### Système de Fichiers (VFS)
- [ ] **Persistance** : Seul `RamFS` est pleinement inscriptible.
- [ ] **UFAT** : Implémenter `read_file`, `write_file`, `create_dir` (Actuellement : `Err(NotImplemented)`).
- [ ] **Ext2** : Vérifier et activer le support en écriture (actuellement axé lecture).

## ✨ Fonctionnalités Manquantes

### USB & Bluetooth
- [ ] **Transport USB** : L'architecture `usb_controller.rs` détecte les contrôleurs (UHCI/EHCI) mais ne transmet pas de paquets.
  - Implémenter les Ring Buffers pour XHCI/EHCI.
  - Implémenter l'énumération des devices connectés.
- [ ] **Stack Bluetooth** : Structures HCI présentes mais aucune communication avec le contrôleur.

### Noyau & Syscalls
- [ ] **Appels Système** : `syscall/mod.rs` contient de nombreux TODOs.
  - Implémenter `fork()` complet (copie espace mémoire).
  - Implémenter `exec()` (chargement ELF).
  - Implémenter `pipe()` pour les redirections Shell réelles.
- [ ] **Signaux** : Implémenter la gestion des signaux (Kill, Stop, Cont).

### Divers
- [ ] **IPv6** : Support totalement absent. À créer (`src/net/ipv6.rs`).
- [ ] **DNS** : Parsing des noms de domaine incomplet ("TODO parse name").
- [ ] **Audio/Vidéo** : Drivers non implémentés (Uniquement détection de classe).

## 🧹 Refactoring & Dette Technique

- [ ] **Nettoyage Hardware Legacy** : Fusionner `src/hardware.rs` avec le nouveau `device_manager`.
- [ ] **Tests** : Réactiver les tests unitaires désactivés dans `src/scheduler`.
- [ ] **Documentation** : Mettre à jour les exemples de code obsolètes.

---
*Généré automatiquement par l'agent Antigravity après analyse statique du code.*
