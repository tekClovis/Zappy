# GUI Zappy (`zappy_gui`, C++ / raylib)

Le client graphique observe la partie en temps réel. Il ne joue pas : il se connecte au serveur comme un spectateur et affiche l'état du monde que celui-ci lui pousse.

---

## 1. Architecture

La GUI se connecte en tant qu'équipe spéciale `GRAPHIC`, puis demande l'état initial du monde (`msz`, `mct`, `tna`, `sgt`) avant de traiter en continu les notifications poussées par le serveur. Le rendu est assuré par **raylib**, avec deux moteurs de rendu interchangeables (`Renderer2D` / `Renderer3D`) implémentant la même interface `IRenderer`.

---

## 2. Fonctionnalités

### 2.1 Connexion et chargement

Au lancement, la GUI s'authentifie avec le nom d'équipe réservé `GRAPHIC`, puis demande successivement la taille du monde (`msz`), le contenu complet de la carte (`mct`), la liste des équipes (`tna`) et l'unité de temps courante (`sgt`). Un écran "Connecting..." puis "Loading map..." s'affiche tant que ces informations n'ont pas été reçues.

### 2.2 Rendu 2D (par défaut)

- carte vue du dessus, une case = un carré vert, avec les ressources représentées par des petits points colorés (jusqu'à leur quantité affichée en chiffre si la case est suffisamment grande) ;
- joueurs représentés par un triangle coloré par équipe, orienté selon leur direction, avec leur niveau affiché au-dessus ;
- déplacement des joueurs **interpolé** entre deux positions reçues, pour un mouvement fluide plutôt que des sauts case par case (avec gestion du cas où le joueur traverse un bord du monde torique : on "snap" plutôt que de le faire glisser à travers tout l'écran) ;
- légende des 7 ressources affichée en permanence en haut à gauche.

### 2.3 Rendu 3D (bascule avec `Tab`)

- caméra libre en orbite autour de la carte : clic droit + glisser pour tourner, molette pour zoomer, clic du milieu (ou flèches du clavier) pour faire un panoramique, `R` pour réinitialiser la caméra ;
- le sol est un damier de cubes plats, les ressources sont de petits cubes colorés, les joueurs sont des cubes orientés (avec une petite excroissance noire indiquant la direction regardée), et les œufs sont des sphères ;
- sélection d'un joueur par lancer de rayon (raycast) depuis la position de la souris.

### 2.4 Effets visuels animés

| Événement serveur | Effet GUI |
|-|-|
| Incantation en cours (`pic`) | Halo doré pulsant sur la case du rituel + auras jaunes autour des participants |
| Fin d'incantation (`pie`) | Flash circulaire vert (succès) ou rouge (échec), qui s'agrandit et s'estompe |
| Broadcast (`pbc`) | Onde sonore bleue qui s'étend depuis l'émetteur + bulle de texte affichant le message |
| Éjection (`pex`) | Halo orange bref sur la case du joueur éjecté |
| Ponte d'un œuf (`enw`) | Apparition en "pop" (petite sphère blanche qui grossit) |
| Éclosion (`ebo`) | Anneau vert qui s'étend puis disparaît |
| Mort d'un œuf (`edi`) | Anneau gris qui s'étend puis disparaît |
| Fin de partie (`seg`) | Bandeau plein écran "GAME OVER - Winner: \<équipe\>" |

### 2.5 Panneau d'information (HUD)

Un panneau latéral, toujours visible, affiche :

- taille de la carte, nombre de joueurs connectés, nombre d'œufs en attente d'éclosion ;
- unité de temps courante (modifiable au clavier, voir contrôles) ;
- liste des équipes avec un point de couleur et le nombre de joueurs vivants par équipe ;
- au clic gauche sur un joueur : sa fiche détaillée (identifiant, équipe, niveau, position, direction, quantité de nourriture **et** estimation du temps de vie restant en secondes, et son inventaire complet) ;
- un journal défilant des 6 derniers messages serveur (`smg`).

### 2.6 Contrôles clavier / souris

| Touche / action | Effet |
|-|-|
| `Tab` | Bascule entre le rendu 2D et le rendu 3D |
| `+` / `-` | Accélère / ralentit l'unité de temps du serveur (commande `sst`) |
| `F11` | Plein écran sans bordure |
| Clic gauche sur un joueur | Sélectionne le joueur et affiche sa fiche détaillée |
| *(mode 3D)* clic droit + glisser | Rotation de la caméra |
| *(mode 3D)* molette | Zoom |
| *(mode 3D)* clic milieu / flèches | Déplacement du point de vue |
| *(mode 3D)* `R` | Réinitialise la caméra |

### 2.7 Protocole serveur → GUI implémenté

La GUI implémente le protocole graphique de référence du sujet (même nomenclature de tags à 3 lettres) :

| Tag | Signification |
|-|-|
| `msz` | Taille de la carte |
| `bct` | Contenu (ressources) d'une case |
| `tna` | Nom d'une équipe |
| `sgt` / `sst` | Lecture / modification de l'unité de temps |
| `seg` | Fin de partie + équipe gagnante |
| `smg` | Message serveur (log) |
| `suc` / `sbp` | Commande inconnue / paramètre invalide |
| `pnw` / `ppo` / `plv` / `pin` / `pdi` | Apparition / position / niveau / inventaire / mort d'un joueur |
| `pic` / `pie` | Début / fin d'incantation |
| `pdr` / `pgt` | Dépôt / ramassage d'un objet par un joueur |
| `pbc` | Broadcast émis par un joueur |
| `pex` | Éjection |
| `enw` / `ebo` / `edi` | Ponte / éclosion / mort d'un œuf |
