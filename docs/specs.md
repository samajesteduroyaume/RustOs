# Spécifications techniques UFAT

## Structure du système de fichiers

### Disposition sur le disque
```
+------------------+
|     Superbloc    |  (1 bloc)
+------------------+
| Table des groupes|  (N blocs)
+------------------+
|  Bitmap blocs    |  (1 bloc par groupe)
+------------------+
| Bitmap inodes    |  (1 bloc par groupe)
+------------------+
|  Table inodes    |  (N blocs par groupe)
+------------------+
|    Données       |  (N blocs par groupe)
+------------------+
```

### Superbloc
Le superbloc contient les métadonnées globales du système de fichiers. Taille : 1024 octets.

### Groupes de blocs
Chaque groupe contient :
- Un bitmap des blocs libres
- Un bitmap des inodes libres
- Une table d'inodes
- Les blocs de données

### Inodes
Chaque inode contient :
- Métadonnées du fichier
- 12 pointeurs directs vers des blocs de données
- 1 pointeur simple d'indirection
- 1 pointeur double d'indirection
- 1 pointeur triple d'indirection

## Limites

| Caractéristique | Limite |
|----------------|---------|
| Taille max fichier | 16 To |
| Taille max système de fichiers | 1 Exaoctet |
| Longueur max nom de fichier | 255 caractères |
| Taille de bloc | 1, 2, 4 ou 8 Ko |
| Nombre max de fichiers | 4 milliards |

## Journalisation
Le système utilise un journal par défaut de 32 Mo pour assurer l'intégrité des données en cas de panne.
