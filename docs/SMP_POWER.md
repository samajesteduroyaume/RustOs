# Documentation SMP & Gestion de l'Énergie

## Vue d'ensemble
RustOS supporte désormais le Symmetric Multi-Processing (SMP), lui permettant d'utiliser plusieurs cœurs CPU détectés via ACPI. Il inclut également une gestion de base de l'énergie pour l'extinction et le redémarrage.

## Architecture SMP

### Détection ACPI
Le système utilise la table **MADT** (Multiple APIC Description Table) pour énumérer tous les APIC Locaux disponibles, correspondant aux cœurs physiques ou logiques.
- **RSDP/RSDT/XSDT** sont analysés pour localiser la MADT.
- **Adresse LAPIC** est récupérée depuis la MADT.
- Les entrées **Processor Local APIC** décrivent les CPU valides.

### Processus de Démarrage (APs)
Les Processeurs d'Application (APs) sont démarrés par le Processeur d'Amorçage (BSP) en utilisant la séquence suivante :
1.  **Trampoline** : Un bloc de code assembleur 16-bit est copié à `0x8000` (mémoire basse).
2.  **INIT IPI** : Envoyé à l'AP cible via son ID LAPIC.
3.  **SIPI (Start-up IPI)** : Envoyé avec le vecteur pointant vers `0x80` (adresse `0x8000`).
4.  **Transition de Mode** : Le trampoline bascule l'AP en Mode Protégé (32-bit), puis en Long Mode (64-bit).
5.  **Entrée Rust** : L'AP saute vers `ap_entry()` dans `src/smp/mod.rs`.

### Données Per-CPU
Pour gérer la synchronisation et le contexte sans verrous globaux excessifs, chaque CPU possède sa propre structure de données :
- **Stockage** : Structure `PerCpuData`.
- **Accès** : Accessible via le registre de segment `GS` (`GS Base`).
- **Contenu** :
    - `lapic_id` : ID du CPU courant.
    - `current_thread` : Pointeur Arc vers le thread actuellement exécuté sur ce cœur.

### Planificateur (Scheduler)
Le `CFSScheduler` (Completely Fair Scheduler) a été adapté :
- Il est désormais sans état vis-à-vis du thread "courant" (stocké dans `PerCpuData`).
- `schedule()` prend le thread courant en argument et retourne le prochain thread.
- Un verrou global protège la `Runqueue`, mais plusieurs CPU peuvent planifier à partir de celle-ci.

## Gestion de l'Énergie

### Extinction (Shutdown)
L'extinction est tentée via plusieurs méthodes :
1.  **ACPI** : Écriture dans le registre `PM1a_CNT_BLK` (défini dans la FADT) avec `SLP_TYP` (S5) et `SLP_EN`.
2.  **QEMU Legacy** : Écriture de `0x2000` sur le port `0x604`.

### Redémarrage (Reboot)
Le redémarrage déclenche une réinitialisation système :
1.  **Contrôleur Clavier** : Impulsion sur la ligne de reset via le port `0x64`.
2.  **Triple Fault** : Provoque délibérément une triple faute en chargeant une IDT invalide.

### Boucle d'Attente (Idle Loop)
Lorsque le planificateur n'a aucun thread à exécuter, il retourne `None`. La boucle principale (ou la boucle `ap_entry`) exécute l'instruction `hlt` pour attendre la prochaine interruption (Timer, Clavier, etc.), réduisant ainsi la consommation d'énergie.
