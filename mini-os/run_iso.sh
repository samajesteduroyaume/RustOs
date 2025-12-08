#!/bin/bash
# Script pour compiler et exécuter RustOS avec QEMU
# Simplifié pour utiliser directement l'image générée par bootimage

set -e

echo "💿 Compilation et exécution de RustOS"
echo "=================================="

# Vérifier les dépendances
REQUIRED_PKGS=("qemu-system-x86")
for pkg in "${REQUIRED_PKGS[@]}"; do
    if ! dpkg -l | grep -q "^ii.*$pkg"; then
        echo "⚠️  Installation du paquet requis: $pkg"
        sudo apt-get install -y "$pkg"
    fi
done

# Nettoyage optionnel (commenté pour la vitesse)
# echo "🧹 Nettoyage..."
# cargo clean

# Compilation avec cargo bootimage
echo "🛠️  Compilation avec bootimage..."
# Ceci génère target/x86_64-test-kernel/debug/bootimage-test-kernel.bin
# Cette image contient déjà un bootloader (créé par la crate bootloader) et le kernel.
cargo bootimage --bin test-kernel --target x86_64-test-kernel.json -Z build-std=core,alloc

# Chemin de l'image disque bootable générée
BOOT_IMAGE="target/x86_64-test-kernel/debug/bootimage-test-kernel.bin"

if [ ! -f "$BOOT_IMAGE" ]; then
    echo "❌ Erreur: L'image $BOOT_IMAGE n'a pas été créée."
    exit 1
fi

echo "✅ Image disque trouvée: $BOOT_IMAGE"

# Lancer QEMU avec l'image disque générée
echo "🚀 Lancement de QEMU..."
# On utilise l'image fournie par bootimage directement
qemu-system-x86_64 \
    -drive format=raw,file="$BOOT_IMAGE" \
    -m 2G \
    -serial stdio \
    -display gtk \
    -vga std \
    -machine q35 \
    -smp 2 \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -no-reboot \
    -no-shutdown
