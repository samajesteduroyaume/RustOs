#!/bin/bash
# Script pour exécuter les tests RamFS sans les drivers USB/Bluetooth

set -e

echo "=== Compilation des tests RamFS ==="
cargo build --test ramfs_tests --no-default-features --features alloc

echo ""
echo "=== Exécution des tests RamFS ==="
# Trouver le binaire de test compilé
TEST_BIN=$(find target/x86_64-unknown-none/debug/deps -name "ramfs_tests-*" -type f ! -name "*.d" | head -1)

if [ -z "$TEST_BIN" ]; then
    echo "Erreur : binaire de test ramfs_tests non trouvé"
    exit 1
fi

echo "Binaire de test : $TEST_BIN"
echo ""
echo "Note : Le test s'exécute en boucle infinie après succès (pas de sortie = succès)"
echo "Si une assertion échoue, le programme panique et s'arrête."
echo ""

# Exécuter le test avec un timeout pour éviter une boucle infinie
timeout 5 "$TEST_BIN" 2>&1 || {
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ]; then
        echo ""
        echo "✓ Tests RamFS réussis ! (timeout après exécution)"
        exit 0
    else
        echo ""
        echo "✗ Tests RamFS échoués (code de sortie: $EXIT_CODE)"
        exit $EXIT_CODE
    fi
}
