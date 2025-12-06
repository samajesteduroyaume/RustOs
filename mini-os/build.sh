#!/bin/bash

# Arrêter en cas d'erreur
set -e

# Construire le noyau
cargo build --release

# Créer la structure de répertoires pour l'ISO
mkdir -p isodir/boot/grub

# Copier le noyau et la configuration GRUB
cp target/x86_64-blog_os/release/mini-os isodir/boot/kernel.elf
cp grub.cfg isodir/boot/grub/

# Créer l'ISO bootable
grub-mkrescue -o mini-os.iso isodir

echo "Image ISO créée : mini-os.iso"
