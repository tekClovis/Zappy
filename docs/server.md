# Serveur Zappy (`zappy_server`, Rust)

Le serveur est la seule autorité de la partie : il simule le monde, applique les règles et distribue les ressources. Les deux clients (IA, GUI) ne communiquent jamais entre eux — tout passe par lui, en TCP. Sa contrainte fondatrice (sujet p.10) : **aucune attente active**, le multiplexage des sockets repose sur `poll`, qui ne se débloque que lorsqu'un socket a de l'activité ou qu'un événement planifié arrive à échéance.

---

## 1. Architecture en couches

Quatre couches, chacune avec une seule responsabilité, pour éviter un objet `Server` qui ferait à la fois les sockets, le parsing **et** les règles du jeu :

| Couche | Possède | Ne connaît rien de |
|-|-|-|
| **Reactor** (`net/reactor.rs`) | l'ensemble `poll`, la table `fd → Connection`, la boucle principale | les règles du jeu |
| **Connection** (`net/connection.rs`) | un socket, ses buffers in/out, le découpage en lignes, son état | le monde |
| **World** (`game/*`) | carte, tuiles, joueurs, équipes, œufs, règles | les sockets |
| **Scheduler** (`scheduler.rs`) | les événements futurs datés (tas-min), la prochaine échéance | les sockets |

Disposition des modules :

```
server/src/
  main.rs                // Config::parse → Server::new/start/run ; exit 84 en cas d'erreur
  server/
    mod.rs               // re-exports Config, Server
    config.rs            // parsing CLI à la main → Result, sans panic
    server.rs            // bind le listener, construit le World, passe la main au Reactor
    scheduler.rs         // tas-min d'événements, next_timeout()
    net/
      connection.rs      // un socket : buffers in/out, framing ligne, ConnState
      reactor.rs         // boucle poll, table fd → Connection, routage des commandes
    game/
      world.rs           // équipes + joueurs ; carte/tuiles/œufs + règles de spawn
      player.rs          // drone : position, orientation, niveau, food, file d'actions
      team.rs            // slots (= œufs), ids des joueurs connectés
      command.rs         // Command : parse + cost + execute (c'est LE protocole IA)
      gui.rs             // encodeur GUI (msz/bct/pnw/…)
```

Le format requête/réponse de l'IA vit dans `game/command.rs` ; l'encodeur GUI est son voisin `game/gui.rs`. Il n'y a pas de couche `proto/` séparée.

### Boucle du reactor

Chaque itération : `poll` (avec un timeout = prochaine échéance du scheduler), puis `handle_io` (accepter les nouveaux sockets + lire/écrire ceux qui sont prêts), `handle_due_events` (déclencher les événements arrivés à échéance), `deliver_outbox` (vider les buffers d'écriture) et `reap_closed` (fermer les connexions terminées). Tout nouveau socket reçoit `WELCOME\n` ; la première ligne complète décide du type de connexion (`GRAPHIC` → GUI, sinon nom d'équipe → IA).

---

## 2. Modèle de temps

- `f` (fréquence) = inverse de l'unité de temps. Défaut `f = 100`.
- Une action de coût `c` unités se termine après `c / f` **secondes**
  (ex. `Forward = 7/f`, `Incantation = 300/f`, `Fork = 42/f`).
- Horloge monotone (`std::time::Instant`) : chaque action est planifiée à `now + c/f`.
- La boucle calcule `timeout = prochaine_échéance - Instant::now()` et le passe à `poll` ; quand le monde est au repos, le processus dort dans `poll`, jamais dans un `sleep` fixe, jamais à `0`.

Événements récurrents / longs, eux aussi dans le scheduler (variantes `Event` de `net/scheduler.rs`) :

- **`RespawnResources`** : repeuplement des ressources toutes les **20 unités** de temps.
- **`IncantationDone`** : fin d'incantation, `300/f` après son lancement.
- **`Starve`** : la famine. Une unité de nourriture = **126 unités** de temps de vie. Un drone éclôt avec 10 food (soit `1260/f` secondes). Modélisé par un événement périodique tous les `126/f` qui décrémente la food ; à zéro le joueur meurt — on répond `dead\n` à son IA et on émet `pdi #n` à la GUI.
- **`ActionDone`** : résolution d'une action de joueur ordinaire, `c/f` après son acceptation.

Seul le joueur concerné est gelé pendant son action ou son incantation ; le serveur entier ne l'est jamais.

---

## 3. Machine à états des connexions

Un socket démarre non identifié. Le serveur envoie `WELCOME\n` ; la première ligne complète lue est le nom d'équipe (ou `GRAPHIC`).

```
            nouveau socket
                │  envoie "WELCOME\n"
                ▼
           ┌─────────┐  première ligne == "GRAPHIC"  ┌──────┐
           │ Pending │ ─────────────────────────────▶│ Gui  │
           └─────────┘                               └──────┘
                │  première ligne == <équipe> avec un slot libre
                ▼
        assigne un œuf/slot, répond "<CLIENT-NUM>\n" puis "X Y\n"
                │
                ▼
           ┌──────┐
           │  Ai  │  (lié à un PlayerId)
           └──────┘
```

Octets exacts du handshake (serveur `<--`, client `-->`) :

```
<-- WELCOME\n
--> <team-name>\n        (ou GRAPHIC pour la GUI)
<-- <CLIENT-NUM>\n        slots libres pour l'équipe ; >=1 = connectable
<-- <X> <Y>\n             dimensions du monde
```

Une IA peut avoir jusqu'à **10 commandes en file** ; au-delà elles sont silencieusement ignorées. Les requêtes sont exécutées **dans l'ordre reçu**, et le temps d'exécution d'une commande ne bloque que ce joueur.

**Slots, œufs et `Fork`.** Une équipe démarre avec `c` slots, chacun montré à la GUI comme un œuf. Une connexion consomme un slot et fait éclore un œuf en joueur d'orientation aléatoire. `Fork` (coût 42) pond un nouvel œuf → ajoute un slot → autorise un client de plus ; `Connect_nbr` renvoie le nombre de slots libres. Une éjection depuis une tuile détruit aussi les œufs qui s'y trouvent.

---

## 4. Jeu de commandes IA (couche `command.rs`)

Chaque action est une variante qui connaît son **coût** et sait muter le monde. Le parsing d'une ligne produit un `Command` ; on l'enfile (≤10 par joueur), on planifie `ActionDone` à `now + cost/f`, et la mutation + la réponse n'arrivent qu'au déclenchement. La seule réponse synchrone est `ko` pour une ligne non parsable.

| Commande | Coût (unités) | Réponse (succès) | Notes |
|-|-|-|-|
| `Forward` `Right` `Left` | 7 | `ok` | déplacement torique / rotation 90° |
| `Look` | 7 | `[t0, t1, ...]` | tuiles dans l'ordre de vision (§6.2) |
| `Inventory` | 1 | `[food n, linemate n, ...]` | les 7 ressources + food restante |
| `Take <obj>` `Set <obj>` | 7 | `ok` / `ko` | `ko` si l'objet n'est pas là / pas porté |
| `Broadcast <texte>` | 7 | `ok` | chaque joueur reçoit `message K, <texte>\n` (§6.4) |
| `Eject` | 7 | `ok` / `ko` | `ko` si seul ; pousse les autres, tue les œufs |
| `Fork` | 42 | `ok` | pond un œuf, libère un slot |
| `Connect_nbr` | 0 | `<n>` | slots libres de l'équipe ; immédiat |
| `Incantation` | 300 | `Elevation underway\nCurrent level: <k>\n` / `ko` | double vérification (§6.5) |
| *(famine)* | — | `dead\n` | pas une commande ; la connexion se ferme ensuite |

Ligne inconnue/malformée → `ko` à l'IA. Pour une ligne GUI, on répond plutôt `suc` (commande inconnue) ou `sbp` (paramètre invalide). Le joueur reste **gelé** jusqu'à ce que son `ActionDone` se résolve, puis on tire la commande suivante de la file. `Incantation` est spéciale : elle planifie un `IncantationDone` et revérifie les prérequis de la tuile (§6.5) au **début et à la fin** ; échouer à l'un ou l'autre répond `ko`. Tous les joueurs de même niveau sur la tuile montent ensemble et restent gelés pendant la durée.

---

## 5. La GUI comme observateur

La GUI s'authentifie avec `GRAPHIC` et n'est ensuite qu'un **observateur** : le serveur lui pousse les mutations du monde. La nomenclature complète figure dans la [fiche GUI](gui.md) ; côté serveur, il suffit d'un puits unique (`notify_gui`) qui ajoute le message au buffer de sortie de chaque connexion `Gui`, plus un petit parseur pour les requêtes que la GUI peut envoyer (`msz`, `bct`, `mct`, `tna`, `ppo`, `plv`, `pin`, `sgt`, `sst`). Règle : ne pousser que la **tuile changée** (`bct`) sur les changements non-joueur (ex. respawn), et les événements d'action précis (`pgt`/`pdr`/`ppo`/…) sinon — ne jamais renvoyer toute la carte (`mct`) sauf sur requête explicite.

---

## 6. Règles du monde (couche `World`)

Tables numériques que le `World` doit implémenter ; les couches réseau ne les voient jamais. Source : `G-YEP-400_zappy.pdf`.

### 6.1 Ressources & respawn

Sept types de ressources. Quantité cible sur toute la carte : `floor(largeur * hauteur * densité)`.

| ressource | densité |
|-|-|
| food | 0.5 |
| linemate | 0.3 |
| deraumere | 0.15 |
| sibur | 0.1 |
| mendiane | 0.1 |
| phiras | 0.08 |
| thystame | 0.05 |

(ex. 10×10 → 50 food, 5 thystame.) Spawn une fois au démarrage puis repeuplement toutes les **20 unités** vers ces cibles ; au moins une de chaque doit exister sur le sol. Chaque repeuplement qui change une tuile émet un `bct` à la GUI.

### 6.2 Vision

Un drone voit un cône qui s'élargit vers l'avant. Au niveau `L` il voit les rangées `0..=L` ; la rangée `r` compte `2r + 1` tuiles, soit `(L+1)²` tuiles au total. La tuile `0` est celle du drone ; la numérotation balaye ensuite chaque rangée de gauche à droite, relativement à l'orientation du drone. `Look` renvoie `[tuile0, tuile1, ...]`, tuiles séparées par une virgule, contenus séparés par des espaces, ex. `[player,,,thystame,,food,...]`.

### 6.3 Géométrie du monde

Trantor est un tore : sortir d'un bord renvoie au bord opposé (`x mod W`, `y mod H`). `Forward` avance d'une tuile selon l'orientation ; `Right`/`Left` tournent l'orientation de 90°.

### 6.4 Direction sonore (broadcast / éjection)

Un `Broadcast` atteint *tous* les joueurs sous la forme `message K, <texte>\n` ; `K` (1–8) est la tuile entrante qui pointe vers l'émetteur, numérotée dans le sens antihoraire avec `1` droit devant le récepteur, relativement à son orientation. `K = 0` quand émetteur et récepteur partagent la tuile. Comme le monde s'enroule, on calcule le vecteur torique **le plus court** du récepteur vers l'émetteur, puis on le mappe sur `K`. `Eject` utilise la même convention dans `eject: K\n`.

### 6.5 Rituel d'élévation

`Incantation` exige, sur une seule tuile, assez de joueurs de même niveau (toutes équipes) et les pierres exactes. Les prérequis sont vérifiés au début **et** à la fin (`300/f` plus tard) ; échouer à l'un ou l'autre → `ko`. En cas de succès chaque participant monte d'un niveau et les pierres sont consommées.

| niveau | joueurs | linemate | deraumere | sibur | mendiane | phiras | thystame |
|-|-|-|-|-|-|-|-|
| 1→2 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| 2→3 | 2 | 1 | 1 | 1 | 0 | 0 | 0 |
| 3→4 | 2 | 2 | 0 | 1 | 0 | 2 | 0 |
| 4→5 | 4 | 1 | 1 | 2 | 0 | 1 | 0 |
| 5→6 | 4 | 1 | 2 | 1 | 3 | 0 | 0 |
| 6→7 | 6 | 1 | 2 | 3 | 0 | 1 | 0 |
| 7→8 | 6 | 2 | 2 | 2 | 2 | 2 | 1 |

**Condition de victoire :** la première équipe avec **au moins 6 joueurs au niveau 8** gagne ; on l'annonce à la GUI avec `seg N`.

---

## 7. Conventions de robustesse

- **Aucun panic dans le chemin du binaire ; code de sortie 84 en cas d'erreur.** `Config::parse` renvoie `Result<_, String>` ; `main` mappe chaque échec sur `ExitCode::from(84)` (convention Epitech) et écrit sur stderr. Pas d'`unwrap`/`expect`/`panic!` dans les chemins de requête ou de démarrage.
- **`libc` est la seule dépendance hors std**, utilisée uniquement pour l'appel `poll`.
