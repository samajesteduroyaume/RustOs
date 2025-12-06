# Guide du développeur

## Structure du code

```
src/
├── lib.rs           # Point d'entrée de la bibliothèque
├── ufat/           # Implémentation principale
│   ├── mod.rs      # Déclarations de modules
│   ├── superblock.rs # Gestion du superbloc
│   ├── inode.rs    # Gestion des inodes
│   ├── dir.rs      # Gestion des répertoires
│   ├── block.rs    # Gestion des blocs
│   └── journal.rs  # Journalisation
└── utils/          # Utilitaires
```

## Normes de code

- Suivre le [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Documenter toutes les API publiques
- Écrire des tests unitaires pour chaque fonctionnalité
- Utiliser `rustfmt` pour le formatage
- Vérifier avec `clippy` avant de soumettre des modifications

## Workflow de développement

1. Créer une branche pour la fonctionnalité :
   ```bash
   git checkout -b feature/ma-nouvelle-fonctionnalite
   ```

2. Implémenter les changements

3. Exécuter les tests :
   ```bash
   cargo test
   ```

4. Vérifier le formatage :
   ```bash
   cargo fmt -- --check
   ```

5. Vérifier les avertissements :
   ```bash
   cargo clippy -- -D warnings
   ```

6. Créer une pull request

## Débogage

Activer les logs de débogage :
```rust
env_logger::Builder::new()
    .filter_level(log::LevelFilter::Debug)
    .init();
```

## Tests

Exécuter tous les tests :
```bash
cargo test -- --test-threads=1
```

Exécuter un test spécifique :
```bash
cargo test test_nom_du_test
```

## Génération de la documentation

Générer la documentation :
```bash
cargo doc --no-deps --open
```
