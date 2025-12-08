#!/bin/bash
set -e

IMAGE="rustos_multiboot2.img"
KERNEL="target/x86_64-rustos/debug/mini-os"

echo "✅ Kernel déjà compilé"

if [ ! -f "$KERNEL" ]; then
    echo "❌ Kernel non trouvé"
    exit 1
fi

echo "💾 Création image disque (256MB)..."
dd if=/dev/zero of=$IMAGE bs=1M count=256 status=none

echo "📋 Table MBR..."
sfdisk $IMAGE << EOF
label: dos
start=2048, type=83, bootable
EOF

LOOP=$(sudo losetup -f --show -P $IMAGE)
sleep 1

echo "🛠️ Format EXT2..."
sudo mkfs.ext2 -F -L "RustOS" $LOOP"p1"

MOUNT=$(mktemp -d)
sudo mount $LOOP"p1" $MOUNT

echo "📂 Installation fichiers..."
sudo mkdir -p $MOUNT/boot/grub
sudo cp $KERNEL $MOUNT/boot/kernel.elf

cat << 'EOF' | sudo tee $MOUNT/boot/grub/grub.cfg
set timeout=5
set default=0

menuentry "RustOS (Multiboot2)" {
    multiboot2 /boot/kernel.elf
    boot
}
EOF

echo "💿 Installation GRUB..."
sudo grub-install --target=i386-pc --boot-directory=$MOUNT/boot $LOOP

sudo umount $MOUNT
sudo losetup -d $LOOP
rmdir $MOUNT

echo ""
echo "✅ Image créée: $IMAGE"
echo "🚀 Lancement QEMU..."
qemu-system-x86_64 -drive file=$IMAGE,format=raw -m 512M
