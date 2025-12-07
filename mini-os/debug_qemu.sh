#!/bin/bash
# Script pour lancer QEMU en mode debug (GDB)

echo "🐞 Lancement de QEMU en mode debug..."
echo "En attente de connexion GDB sur localhost:1234..."

# Utiliser l'image bootimage
IMAGE="target/x86_64-test-kernel/release/bootimage-test-kernel.bin"

if [ ! -f "$IMAGE" ]; then
    echo "❌ Erreur: $IMAGE non trouvé. Lancez cargo bootimage d'abord."
    exit 1
fi

qemu-system-x86_64 \
    -drive format=raw,file="$IMAGE" \
    -s -S \
    -nographic \
    -serial mon:stdio \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -no-reboot &

# Sauvegarder le PID pour pouvoir le tuer plus tard
echo $! > qemu_debug.pid
echo "QEMU lancé (PID: $(cat qemu_debug.pid))"
