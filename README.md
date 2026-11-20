# Projet « squares » Programmation bas niveau

**Cours :** Programmation bas niveau (UPEC)  
**Outil :** Rust + [Piston](https://www.piston.rs/)  
**Binaire :** `squares`

## 1. Objectif (énoncé)

Réaliser une application graphique qui :

1. Ouvre une fenêtre et affiche un ou plusieurs **carrés** déplaçables au clavier.
2. Lit des **arguments en ligne de commande** : chaque groupe de **4 lettres** décrit un carré et les **4 touches** qui le contrôlent (une touche par direction).
3. N’accepte la ligne de commande que si le format est respecté et qu’**aucune lettre n’est répétée** entre tous les carrés.

Exemple demandé par l’enseignant :

```bash
squares AZER QSDF WXCV
```

→ 3 carrés : touches `A,Z,E,R` pour le 1er, `Q,S,D,F` pour le 2ᵉ, `W,X,C,V` pour le 3ᵉ.

Ce projet est une **étape avant une version réseau** (chapitres 16, 18 et 20 du polycopié).

## 2. Prérequis

- [Rust](https://www.rust-lang.org/tools/install) (avec `cargo`)
- Sous Linux : pilotes OpenGL et bibliothèques pour fenêtre (Glutin / Wayland ou X11), en général déjà présents sur une machine de bureau

## 3. Compilation et exécution

```bash
cd mini_jeu
cargo build --release
cargo run -- AZER QSDF WXCV
```

- **Échap** : fermer la fenêtre.
- En cas d’arguments invalides : message sur la sortie d’erreur et code de sortie `1`.

Exemples de rejets :

```bash
cargo run --                    # aucun argument
cargo run -- AZE                 # pas 4 lettres
cargo run -- AZER AZER           # lettre « A » en double
```

## 4. Convention des 4 lettres

L’énoncé ne fixe pas l’ordre des 4 lettres par rapport aux directions. Le projet documente **deux conventions** dans `src/main.rs` :

| Constante `KEY_CONVENTION` | Signification du bloc `AZER` |
|-----------------------------|------------------------------|
| `HautBasGaucheDroite` **(par défaut)** | A=haut, Z=bas, E=gauche, R=droite |
| `RangeeAzerty` | A=gauche, Z=haut, E=bas, R=droite (4 touches lues comme sur une rangée AZERTY) |

Pour changer de convention, ouvrir `src/main.rs` et modifier :

```rust
const KEY_CONVENTION: KeyConvention = KeyConvention::HautBasGaucheDroite;
// ou
const KEY_CONVENTION: KeyConvention = KeyConvention::RangeeAzerty;
```

La fonction unique `controls_from_keys` centralise le mapping ; on peut y ajouter une variante si le prof en impose une autre.

## 5. Architecture du programme

```
main()
  ├─ parse_square_groups()     ← argv → Vec<Controls> + validation
  ├─ création des Square       ← position + couleur
  ├─ fenêtre Glutin + OpenGL
  └─ boucle Events
        ├─ press / release clavier → on_key()
        ├─ update (dt)           → déplacement
        └─ render                → dessin des carrés
```

**Bibliothèques :**

| Crate | Rôle |
|--------|------|
| `piston` | Boucle d’événements, abstractions clavier / temps |
| `pistoncore-glutin_window` | Fenêtre système (Glutin) |
| `piston2d-opengl_graphics` | Dessin 2D via OpenGL |
| `piston2d-graphics` | Primitives (rectangle, transformations) |

## 6. Comprendre `main.rs` (par blocs)

### Bloc A Imports et constantes (l. 8–23)

- `std::env::args` (via `main`) : arguments du terminal.
- `HashSet` : détecter les lettres déjà utilisées.
- Constantes `WINDOW_*`, `SQUARE_SIZE`, `MOVE_SPEED` : taille fenêtre, carré, vitesse en pixels/seconde.

### Bloc B Convention des touches (l. 25–48)

- `KeyConvention` : enum des interprétations possibles d’un bloc de 4 caractères.
- `KEY_CONVENTION` : choix actuel (un seul endroit à modifier).
- `controls_from_keys` : transforme `[Key; 4]` en struct `Controls` (up/down/left/right).

### Bloc C Structures `Controls` et `Square` (l. 50-121)

- **`Controls`** : les 4 touches Piston (`Key::A`, etc.) pour un carré.
- **`Square`** : position `(x, y)`, couleur RGBA, contrôles, et 4 booléens « touche maintenue ».

**`Square::new`** : place les carrés sur une grille dans la fenêtre (répartition selon le nombre total) et attribue une couleur distincte.

**`apply_key`** : si la touche correspond à une des 4 du carré, met à jour le booléen de mouvement (true à l’appui, false au relâchement).

**`step`** : à chaque frame, calcule `dx`/`dy` avec `MOVE_SPEED * dt`, met à jour la position, **clamp** pour rester dans la fenêtre.

### Bloc D  `key_from_char` (l. 124-154)

Convertit une lettre `A`–`Z` (minuscule acceptée) en `piston::input::Key`. Piston ne fournit pas de conversion automatique char → `Key` pour toutes les touches ; le `match` est explicite (exigence de compréhension du code).

### Bloc E `parse_square_groups` (l. 156-203)

1. Refuse l’absence d’arguments.
2. Pour chaque argument : exactement 4 lettres alphabétiques ASCII.
3. Vérifie l’unicité globale des lettres (insensible à la casse pour la détection des doublons).
4. Produit un `Vec<Controls>` via `controls_from_keys`.

Retourne `Result<_, String>` pour des messages d’erreur lisibles.

### Bloc F `App` : rendu et logique jeu (l. 205-236)

- **`render`** : `GlGraphics::draw` efface l’écran (fond sombre), dessine chaque carré avec `graphics::rectangle` et une transformation `trans(x, y)`.
- **`update`** : appelle `step` sur chaque carré avec `args.dt` (delta temps en secondes — fourni par Piston).
- **`on_key`** : propage chaque événement clavier à **tous** les carrés ; seuls ceux qui possèdent cette touche bougent (les autres ignorent dans `apply_key`).

### Bloc G `main` (l. 238-281)

1. Parse les arguments ; `exit(1)` si erreur.
2. Construit les `Square`.
3. Crée la fenêtre : `WindowSettings::new`, API OpenGL 3.2, `exit_on_esc(true)`.
4. Boucle **`while let Some(e) = events.next(&mut window)`** - cœur Piston :
   - `press_args` / `release_args` : clavier
   - `render_args` : redessiner
   - `update_args` : physique / mouvement

Tant que la fenêtre est ouverte, cette boucle tourne ; elle se termine à la fermeture ou sur Échap.

## 7. Périmètre et limites

- Un seul fichier source pour faciliter la relecture pédagogique (pas de version réseau dans cette livraison).
- Pas de gestion de caractères accentués ou symboles dans les arguments (lettres A–Z uniquement).
- L’ordre des 4 lettres ↔ directions est **configurable** mais doit être aligné avec la correction du professeur.

## 8. Auteur et honnêteté académique

À compléter : **nom, prénom, groupe, date**.

Le correcteur attend que tu **comprennes** chaque partie (boucle d’événements, parsing, validation, dessin OpenGL). Ce README et les commentaires dans le code sont là pour t’aider à préparer l’oral ou les questions.

## 9. Références

- [Piston — site officiel](https://www.piston.rs/)
- [Tutoriel « getting started » Piston](https://github.com/PistonDevelopers/Piston-Tutorials/tree/master/getting-started)