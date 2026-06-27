# ZAPPY

> simulation d'un monde multijoueur sur **Trantor**

## À propos

**Zappy** est un jeu en réseau dans lequel plusieurs équipes s'affrontent sur une carte de cases truffée de ressources. Chaque joueur est piloté par une IA autonome qui doit se nourrir, explorer, récolter des pierres et accomplir des rituels d'élévation pour grimper dans la hiérarchie Trantorienne.

> **Condition de victoire** : la première équipe ayant **au moins 6 joueurs au niveau maximum (niveau 8)** gagne la partie.

Une fois la partie lancée, **ni l'IA, ni l'interface graphique ne peuvent être pilotées par l'utilisateur** : tout se joue de manière autonome via le protocole réseau.

---

## Architecture du projet

Le projet est composé de **trois binaires indépendants**, qui communiquent exclusivement via **sockets TCP** :

| Binaire | Rôle | Langage |
|-|-|-|
| `zappy_server` | Génère et gère le monde, les joueurs, les ressources, les règles | Rust |
| `zappy_gui` | Client graphique permettant de visualiser la partie en temps réel | C++ (raylib) |
| `zappy_ai` | Client autonome qui pilote un joueur ("drone") sur le serveur | Python |

```
                ┌─────────────┐
                │ zappy_server│  ← un seul process, un seul thread (poll())
                └──────┬──────┘
            TCP socket │ TCP socket
        ┌───────────────┼───────────────┐
        │               │               │
 ┌──────▼─────┐  ┌──────▼─────┐  ┌──────▼─────┐
 │ zappy_ai #1│  │ zappy_ai #2│  │ zappy_gui  │
 │ (équipe A) │  │ (équipe B) │  │ (GRAPHIC)  │
 └────────────┘  └────────────┘  └────────────┘
```

Les clients **ne communiquent jamais entre eux directement** : tout passe par le serveur.

---

## Compilation

Le `Makefile` à la racine expose trois règles principales :

```bash
make
make zappy_server
make zappy_gui
make zappy_ai
make clean
make fclean
make re
```

---

## Utilisation

### `zappy_server`

```bash
./zappy_server -p port -x width -y height -n name1 name2 ... -c clientsNb -f freq
```

| Option | Description |
|-|-|
| `-p` | numéro de port |
| `-x` | largeur du monde |
| `-y` | hauteur du monde |
| `-n` | nom(s) des équipes |
| `-c` | nombre de clients initiaux autorisés par équipe |
| `-f` | fréquence (inverse de l'unité de temps), `100` par défaut |

**Exemple :**

```bash
./zappy_server -p 4242 -x 20 -y 20 -n teamA teamB -c 6 -f 100
```

### `zappy_gui`

```bash
./zappy_gui -p port -h machine
```

| Option | Description |
|-|-|
| `-p` | numéro de port du serveur |
| `-h` | nom de la machine (hostname)|

**Exemple :**

```bash
./zappy_gui -p 4242 -h localhost
```

> La GUI s'authentifie auprès du serveur en envoyant le nom d'équipe réservé **`GRAPHIC`**.

### `zappy_ai`

```bash
./zappy_ai -p port -n name -h machine
```

| Option | Description |
|-|-|
| `-p` | numéro de port du serveur |
| `-n` | nom de l'équipe à laquelle se connecter |
| `-h` | nom de la machine, `localhost` par défaut |

**Exemple :**

```bash
./zappy_ai -p 4242 -n teamA -h localhost
```

### Lancer une partie complète

```bash
# 1. Démarrer le serveur
./zappy_server -p 4242 -x 20 -y 20 -n teamA teamB -c 6 -f 100

# 2. (Optionnel) Lancer l'interface graphique pour observer la partie
./zappy_gui -p 4242 -h localhost

# 3. Connecter les IA d'une équipe (jusqu'à clientsNb)
./zappy_ai -p 4242 -n teamA -h localhost
```

---

## Règles du jeu

### Le monde : Trantor

- Carte **plate, sans relief**, qui se comporte comme un **tore** : sortir par la droite
  ramène à gauche, sortir par le haut ramène en bas, etc.
- Le serveur tourne en **un seul process / un seul thread**, multiplexé via `poll()`
  (aucune attente active autorisée).

### Les ressources

À chaque démarrage du serveur, puis **toutes les 20 unités de temps**, les ressources sont
respawnées et réparties uniformément sur la carte selon la formule :

```
quantité = map_width * map_height * densité
```

| Ressource | Densité |
|-|-|
| food | 0.5 |
| linemate | 0.3 |
| deraumere | 0.15 |
| sibur | 0.1 |
| mendiane | 0.1 |
| phiras | 0.08 |
| thystame | 0.05 |

> Exemple : sur une carte 10×10, on trouve 50 *food* et 5 *thystame*.

La nourriture est la **seule ressource de survie** : 1 unité de food = **126 unités de temps**
de vie. Un joueur démarre avec 10 unités de vie (1260 unités de temps, soit `1260 / f` secondes).

### Le rituel d'élévation

Pour monter de niveau, des joueurs **de même niveau** (peu importe l'équipe) doivent se réunir
sur une même case avec les ressources requises, puis lancer `Incantation`. La vérification des
conditions se fait **au début ET à la fin** du rituel si elles ne sont plus respectées, le
rituel échoue. Pendant l'incantation, **tous les participants sont figés**.

| Élévation | Joueurs requis | linemate | deraumere | sibur | mendiane | phiras | thystame |
|-|-|-|-|-|-|-|-|
| 1 → 2 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| 2 → 3 | 2 | 1 | 1 | 1 | 0 | 0 | 0 |
| 3 → 4 | 2 | 2 | 0 | 1 | 0 | 2 | 0 |
| 4 → 5 | 4 | 1 | 1 | 2 | 0 | 1 | 0 |
| 5 → 6 | 4 | 1 | 2 | 1 | 3 | 0 | 0 |
| 6 → 7 | 6 | 1 | 2 | 3 | 0 | 1 | 0 |
| 7 → 8 | 6 | 2 | 2 | 2 | 2 | 2 | 1 |

Une fois le rituel réussi, les pierres consommées disparaissent de la case.

### La vision

Le champ de vision est **triangulaire**, centré devant le joueur, et s'agrandit d'une case en
profondeur (et d'une case sur chaque côté de la nouvelle ligne) à chaque élévation. Au niveau 1,
l'unité de vision vaut 1.

La commande `Look` renvoie le contenu de chaque case visible, numérotée selon un schéma en
éventail centré sur le joueur (case `0`), les cases étant séparées par une virgule.

### Les broadcasts

Le `Broadcast` envoie un message à **tous** les joueurs de la carte, sans révéler l'identité de
l'émetteur. Chaque récepteur perçoit uniquement la **direction** d'origine du son (numérotée de
1 à 8 autour de lui, 0 si le son vient de sa propre case), le **trajet le plus court** étant
toujours retenu sur ce monde torique.

### Reproduction et œufs

La commande `Fork` pond un œuf et libère un nouveau slot de connexion pour l'équipe (visible via
`Connect_nbr`). Quand un client se connecte sur un slot libre, un œuf de l'équipe est choisi
**aléatoirement**, éclot, et le joueur apparaît avec une **direction aléatoire**.

---

## Protocole réseau

### Connexion d'un client

```
<-- WELCOME\n
--> TEAM-NAME\n
<-- CLIENT-NUM\n
<-- X Y\n
```

- `CLIENT-NUM` : nombre de slots restants pour l'équipe (≥ 1 requis pour se connecter)
- `X Y` : dimensions du monde

### Liste des commandes

| Action | Commande | Temps | Réponse |
|---|---|---|---|
| Avancer d'une case | `Forward` | 7/f | `ok` |
| Tourner à droite | `Right` | 7/f | `ok` |
| Tourner à gauche | `Left` | 7/f | `ok` |
| Regarder autour de soi | `Look` | 7/f | `[tile1, tile2, ...]` |
| Voir son inventaire | `Inventory` | 1/f | `[linemate n, sibur n, ...]` |
| Émettre un son | `Broadcast <text>` | 7/f | `ok` |
| Slots libres de l'équipe | `Connect_nbr` | - | valeur |
| Pondre un œuf | `Fork` | 42/f | `ok` |
| Éjecter les autres joueurs | `Eject` | 7/f | `ok` / `ko` |
| Mort du joueur | - | - | `dead` |
| Ramasser un objet | `Take <objet>` | 7/f | `ok` / `ko` |
| Déposer un objet | `Set <objet>` | 7/f | `ok` / `ko` |
| Lancer une incantation | `Incantation` | 300/f | `Elevation underway / Current level: k` / `ko` |

> Toute commande inconnue ou mal formée reçoit `ko`. Chaque ligne envoyée se termine par `\n`.

**Temps réel d'une action** = `action / f` secondes (`f = 100` par défaut, donc `Forward` ≈ 0.07 s).

---

## Performances

Avec notre implémentation, une équipe parvient à faire atteindre le **niveau 8 (niveau maximum)** à ses joueurs en environ **100 secondes** (avec `f=100`), grâce à une IA qui optimise en continu l'exploration, la collecte des ressources et la synchronisation des incantations entre joueurs de même niveau.

---

## Auteurs

Clovis Nedelec \
Eros Delianne-le-boucher \
Nathan Cheynet\
Yohan Dupret \
Alexandre Delain

