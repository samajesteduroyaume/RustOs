#!/bin/bash
# Script pour exécuter les tests RustOS via bootimage
set -e

echo ""

# Vérifier que QEMU est installé
if ! command -v qemu-system-x86_64 &> /dev/null; then
    echo "❌ Erreur: qemu-system-x86_64 n'est pas installé"
    echo ""
    echo "Pour installer QEMU:"
    echo "  Ubuntu/Debian: sudo apt install qemu-system-x86"
    echo "  Fedora: sudo dnf install qemu-system-x86"
    echo "  Arch: sudo pacman -S qemu"
    echo ""
    exit 1
fi

# Compiler le kernel de test
echo "📦 Compilation du kernel de test..."
cargo build --bin test-kernel --release --target x86_64-unknown-none

# Vérifier que la compilation a réussi
if [ $? -ne 0 ]; then
    echo "❌ Erreur de compilation"
    exit 1
fi

echo "✅ Compilation réussie"
echo ""

# Créer une image bootable avec bootimage
echo "🔨 Création de l'image bootable..."
if ! command -v bootimage &> /dev/null; then
    echo "⚠️  bootimage n'est pas installé, installation..."
    cargo install bootimage
fi

# Build bootimage pour le binaire de test
cargo bootimage --bin test-kernel --release --target x86_64-unknown-none

if [ $? -ne 0 ]; then
    echo "❌ Erreur lors de la création de l'image bootable"
    echo ""
    echo "Note: Pour l'instant, les tests QEMU nécessitent une configuration"
    echo "      plus avancée. Vous pouvez exécuter les tests d'intégration avec:"
    echo "      ./run_ramfs_tests.sh"
    exit 1
fi

echo "✅ Image bootable créée"
echo ""

# Lancer QEMU avec les bons paramètres
echo "🚀 Lancement de QEMU..."
echo "----------------------------------------"

qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-unknown-none/release/bootimage-test-kernel.bin \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -serial stdio \
    -display none \
    -no-reboot

# Capturer le code de sortie
EXIT_CODE=$?

echo "----------------------------------------"
echo ""

# Interpréter le code de sortie
# QEMU exit code = (value << 1) | 1
# Success (0x10) → (0x10 << 1) | 1 = 33
# Failed (0x11) → (0x11 << 1) | 1 = 35

if [ $EXIT_CODE -eq 33 ]; then
    echo "✅ Tous les tests ont réussi!"
    exit 0
elif [ $EXIT_CODE -eq 35 ]; then
    echo "❌ Des tests ont échoué"
    exit 1
else
    echo "⚠️  Code de sortie inattendu: $EXIT_CODE"
    echo "   (QEMU peut ne pas être configuré correctement)"
    exit 1
fi
