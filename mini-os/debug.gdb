# Script GDB pour déboguer RustOS
set pagination off
set confirm off

# Charger les symboles du kernel
file target/x86_64-test-kernel/release/test-kernel

# Connexion à QEMU
target remote :1234

# Mettre un breakpoint à _start
break _start

# Continuer l'exécution jusqu'au breakpoint
continue

# Si on atteint le breakpoint, afficher des infos
echo \n=== BREAKPOINT AT _START REACHED ===\n
info registers
backtrace

# Désassembler les prochaines instructions
x/10i $pc

# Essayer d'avancer un peu
stepi
stepi

echo \n=== AFTER 2 INSTRUCTIONS ===\n
info registers

# Quitter
quit
