# Robot Simulation

Simulation multi-agents en Rust : des robots éclaireurs explorent une carte générée
procéduralement, découvrent des ressources et des obstacles, puis des robots
collecteurs vont récolter ces ressources et les rapportent à une base centrale.
Le tout s'affiche en temps réel dans le terminal grâce à [Ratatui](https://ratatui.rs/).

## Prérequis

- [Rust](https://www.rust-lang.org/tools/install) (édition 2024, via `rustup`)
- Un terminal qui supporte les couleurs ANSI et fait au moins **80 colonnes x 32 lignes**
  (Windows Terminal, PowerShell, un terminal Linux/macOS classique...)

### Windows : linker manquant

Si `cargo build` échoue avec une erreur du type `linker 'link.exe' not found`, c'est que
les Build Tools C++ de Visual Studio ne sont pas installés. Deux solutions :

- Installer le composant **"Desktop development with C++"** depuis le Visual Studio Installer, **ou**
- Utiliser le toolchain GNU à la place (plus léger) :
  ```powershell
  winget install --id BrechtSanders.WinLibs.POSIX.UCRT -e
  rustup target add x86_64-pc-windows-gnu
  rustup override set stable-x86_64-pc-windows-gnu   # dans le dossier du projet
  ```

## Lancer le projet

```bash
git clone <url-du-repo>
cd robot_simulation
cargo run --release
```

Le mode `--release` est recommandé : la simulation tourne plus fluide (le mode debug
fonctionne aussi mais consomme plus de CPU).

**Quitter** : appuyer sur n'importe quelle touche restaure le terminal proprement.

## Légende

| Élément             | Caractère | Couleur          |
|---------------------|-----------|------------------|
| Case vide           | `.`       | Gris             |
| Obstacle            | `O`       | Cyan clair       |
| Base                | `#`       | Vert clair       |
| Énergie             | `E`       | Vert             |
| Cristal             | `C`       | Magenta clair    |
| Robot éclaireur     | `x`       | Rouge            |
| Robot collecteur    | `o`       | Magenta          |

Le panneau du bas affiche l'énergie/les cristaux collectés, le nombre de robots actifs,
les obstacles/ressources connus de la base, et les derniers événements.

## Fonctionnement

- La carte (80x30) est générée avec du bruit de Perlin (obstacles), une base centrale
  et des gisements d'énergie/cristaux placés aléatoirement.
- Chaque robot (éclaireur ou collecteur) tourne dans son propre thread. Les éclaireurs
  explorent au hasard et signalent leurs découvertes ; les collecteurs réservent une
  ressource connue, s'y rendent par recherche de chemin (BFS), la récoltent puis la
  rapportent à la base.
- La communication entre robots et base se fait par messages (`RobotMessage`) envoyés
  via un canal `mpsc` ; la carte et la base sont protégées respectivement par
  `RwLock`/`Mutex` pour un accès concurrent sûr.
- L'interface Ratatui redessine l'état de la simulation en temps réel.

## Structure du code

```
src/
├── main.rs        # mise en place du terminal et boucle principale (rendu + clavier)
├── simulation.rs   # threads des robots, état partagé, canal de messages
├── ui.rs           # rendu Ratatui (carte + statistiques)
├── message.rs      # messages échangés entre robots et base
├── map.rs           # génération de la carte (bruit de Perlin, ressources, base)
├── cell.rs           # types de cases de la carte
├── position.rs      # coordonnées (x, y)
├── resource.rs       # ressources (énergie / cristaux)
├── robot.rs           # robot éclaireur
├── collector.rs        # robot collecteur + pathfinding (BFS)
└── base.rs              # connaissances de la base, compteurs, journal d'événements
```

## Dépendances principales

- [`ratatui`](https://crates.io/crates/ratatui) — interface terminal
- [`crossterm`](https://crates.io/crates/crossterm) — gestion du terminal et du clavier
- [`noise`](https://crates.io/crates/noise) — génération de la carte (bruit de Perlin)
- [`rand`](https://crates.io/crates/rand) — placement aléatoire et déplacements
