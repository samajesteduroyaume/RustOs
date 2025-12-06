// build.rs - Configuration de la compilation pour Rust 1.75.0

fn main() {
    // Active les fonctionnalités nécessaires pour le noyau
    println!("cargo:rustc-cfg=feature=\"no_std\"");
    
    // Active les fonctionnalités spécifiques à la cible
    println!("cargo:rustc-cfg=arch=\"x86_64\"");
    
    // Configuration pour le support du noyau
    println!("cargo:rustc-cfg=kernel");
    
    // Configuration du linker
    println!("cargo:rustc-link-arg=-Tlinker.ld");
    println!("cargo:rerun-if-changed=linker.ld");
    
    // Active les optimisations pour le code du noyau
    println!("cargo:rustc-rust-cfg=target_os=\"none\"");
    
    // Active les fonctionnalités expérimentales nécessaires
    println!("cargo:rustc-cfg=feature=\"naked_functions\"");
    println!("cargo:rustc-cfg=feature=\"asm\"");
    
}
