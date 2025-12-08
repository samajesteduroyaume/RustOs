#!/bin/bash
set -e
echo "🧹 Nettoyage du projet..."
cargo clean

echo "💿 Création de RustOS Main ISO"
cargo bootimage --bin mini-os --release --target x86_64-blog_os.json -Z build-std=core,alloc -Z build-std-features=compiler-builtins-mem

SOURCE_IMG="target/x86_64-blog_os/release/bootimage-mini-os.bin"
DEST_IMG="rustos_main.img"

if [ ! -f "$SOURCE_IMG" ]; then
    echo "❌ Erreur : Image bootimage non générée."
    exit 1
fi

cp "$SOURCE_IMG" "$DEST_IMG"
truncate -s 10M "$DEST_IMG"
echo "✅ Image bootable créée : $DEST_IMG (Redimensionnée à 10MB)"

echo "🔍 Inspection de l'image :"
ls -lh "$DEST_IMG"
file "$DEST_IMG"
fdisk -l "$DEST_IMG" || true

echo "🚀 Lancement de QEMU..."

qemu-system-x86_64 \
    -drive format=raw,file="$DEST_IMG" \
    -m 2G \
    -serial file:serial.log \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -no-reboot \
    -no-shutdown &
QEMU_PID=$!
echo "QEMU lancé avec PID $QEMU_PID. Logs dans serial.log"
wait $QEMU_PID
