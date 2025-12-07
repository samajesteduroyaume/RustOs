#!/bin/bash
# Script pour construire et exécuter les tests RustOS avec GRUB
set -e

echo "🧪 RustOS - Tests avec GRUB + QEMU"
echo "===================================="
echo ""

# Vérifier que GRUB est installé
if ! command -v grub-mkrescue &> /dev/null; then
    echo "❌ Erreur: grub-mkrescue n'est pas installé"
    echo ""
    echo "Pour installer GRUB:"
    echo "  Ubuntu/Debian: sudo apt install grub-pc-bin xorriso"
    echo "  Fedora: sudo dnf install grub2-tools xorriso"
    echo "  Arch: sudo pacman -S grub xorriso"
    echo ""
    exit 1
fi

# Compiler le kernel de test
echo "📦 Compilation du kernel de test..."
cargo build --bin test-kernel --release --target x86_64-test-kernel.json -Z build-std=core,alloc -Z build-std-features=compiler-builtins-mem

if [ $? -ne 0 ]; then
    echo "❌ Erreur de compilation"
    exit 1
fi

echo "✅ Compilation réussie"
echo ""

# Créer la structure de répertoires pour l'ISO
echo "🔨 Création de l'ISO de test..."
mkdir -p isodir_test/boot/grub

# Copier le kernel et la configuration GRUB
cp target/x86_64-test-kernel/release/test-kernel isodir_test/boot/test-kernel.elf
cp grub_test.cfg isodir_test/boot/grub/grub.cfg

# Créer l'ISO bootable
grub-mkrescue -o test-kernel.iso isodir_test 2>&1 | grep -v "warning:"

if [ $? -ne 0 ]; then
    echo "❌ Erreur lors de la création de l'ISO"
    exit 1
fi

echo "✅ ISO de test créée: test-kernel.iso"
echo ""

# Lancer QEMU avec l'ISO
echo "🚀 Lancement de QEMU..."
echo "----------------------------------------"

# Lancer QEMU
qemu-system-x86_64 \
    -cdrom test-kernel.iso \
    -nographic \
    -serial mon:stdio \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -no-reboot

# Capturer le code de sortie
EXIT_CODE=$?

echo "----------------------------------------"
echo ""

# Interpréter le code de sortie
if [ $EXIT_CODE -eq 33 ]; then
    echo "✅ Tous les tests ont réussi!"
    exit 0
elif [ $EXIT_CODE -eq 35 ]; then
    echo "❌ Des tests ont échoué"
    exit 1
else
    echo "⚠️  Code de sortie: $EXIT_CODE"
    if [ $EXIT_CODE -eq 0 ] || [ $EXIT_CODE -eq 130 ]; then
        echo "   (Interruption manuelle ou fin normale)"
        exit 0
    else
        echo "   (Code inattendu)"
        exit 1
    fi
fi
