#!/bin/bash
# Script pour créer une ISO bootable

set -e

echo "💿 Création de RustOS ISO"
echo "========================="

# Vérifier la présence de grub-pc-bin pour le support BIOS legacy
if ! dpkg -l | grep -q "grub-pc-bin"; then
    echo "⚠️  ATTENTION : Le paquet 'grub-pc-bin' est manquant."
    echo "    L'ISO générée ne sera probablement PAS bootable sur QEMU par défaut (BIOS)."
    echo "    Elle ne fonctionnera qu'en mode UEFI."
    echo ""
    echo "    Pour corriger ce problème, installez le paquet :"
    echo "    👉 sudo apt-get install grub-pc-bin"
    echo ""
    echo "    (Appuyez sur Entrée pour continuer quand même, ou Ctrl+C pour annuler)"
    read -t 5 || true
fi

# Utiliser cargo bootimage qui gère correctement le passage 32-bit -> 64-bit
# contrairement à une ISO GRUB manuelle qui nécessiterait un trampoline assembleur.

echo "📦 Création de l'image disque bootable (RustOS)..."

# 1. Compiler avec bootimage
cargo bootimage --bin test-kernel --release --target x86_64-test-kernel.json -Z build-std=core,alloc -Z build-std-features=compiler-builtins-mem

# 2. Récupérer l'image générée
SOURCE_IMG="target/x86_64-test-kernel/release/bootimage-test-kernel.bin"
DEST_IMG="rustos.img"

if [ ! -f "$SOURCE_IMG" ]; then
    echo "❌ Erreur : Image bootimage non générée."
    exit 1
fi

cp "$SOURCE_IMG" "$DEST_IMG"

echo "✅ Image bootable créée : $DEST_IMG"
echo "   (Format : Image Disque RAW / HDD)"
echo ""
echo "🚀 Lancement de QEMU..."

# 3. Lancer QEMU en mode Disque Dur (pas CDROM)
qemu-system-x86_64 \
    -drive format=raw,file="$DEST_IMG" \
    -serial mon:stdio \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04
