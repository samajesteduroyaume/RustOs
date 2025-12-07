#!/bin/bash
# Test l'ISO fraîchement créée

echo "🧪 Test de l'ISO rustos.iso avec QEMU..."

# Timeout de 10s pour éviter bloquage
timeout 10 qemu-system-x86_64 \
    -cdrom rustos.iso \
    -nographic \
    -serial mon:stdio \
    -no-reboot
