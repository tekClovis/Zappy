# Documentation du projet Zappy

---

## 1. Présentation générale

Zappy est un jeu réseau où plusieurs équipes de joueurs autonomes explorent un monde torique, se nourrissent, collectent des pierres et réalisent des rituels d'incantation pour grimper jusqu'au niveau 8. Le projet est découpé en trois composants qui ne communiquent **que** via le serveur, en TCP :

| Composant | Rôle | Langage / techno |
|-|-|-|
| `zappy_server` | Simule le monde, applique les règles, distribue les ressources | Rust |
| `zappy_ai` | Pilote un joueur de façon autonome | Python |
| `zappy_gui` | Observe la partie en temps réel | C++ |

---

## 2. Architecture technique

### 2.1 Serveur (Rust)

Le serveur repose sur un **reactor** event-driven autour d'un seul appel `libc::poll` :

- un seul thread, **aucune attente active** : `poll()` se débloque uniquement quand un socket a des données, ou qu'un événement planifié (`Scheduler`) arrive à échéance ;
- chaque action de joueur (`Forward`, `Take`, `Incantation`, ...) est traduite en un événement programmé dans le futur (`Event::ActionDone`, `Event::IncantationDone`), exécuté après le délai `action / f` ; seul le joueur concerné est bloqué pendant ce temps, jamais le serveur entier ;
- la famine est gérée par un événement périodique `Event::Starve` (toutes les 126 unités de temps, soit la durée d'une unité de nourriture) ;
- le respawn des ressources est un événement périodique `Event::RespawnResources` (toutes les 20 unités de temps) ;
- les nouvelles connexions, les lignes reçues, les écritures en attente et les sockets fermés sont tous traités dans la même itération de boucle (`handle_io`, `handle_due_events`, `reap_closed`, `deliver_outbox`).

### 2.2 IA (Python)

Le client IA est construit autour d'une boucle `asyncio` :

- `ZappyConnection` gère le handshake (`WELCOME` / team name / nb de slots / dimensions) puis un thread de lecture asynchrone (`_reader_loop`) qui distingue les réponses **sollicitées** (résultat de la dernière commande envoyée) des messages **non sollicités** (`message`, `eject:`, `Current level:`, `dead`), redirigés vers un callback ;
- `GameState` garde l'inventaire, la vision, le niveau et les besoins en ressources pour l'élévation courante ;
- `HeuristicStrategy` est le cerveau du joueur : à chaque `tick()`, elle lit l'état du jeu et envoie 0 à N commandes (voir [section 3](#3-stratégie-de-lia)).

> Le code prévoit une interface `Strategy` abstraite et un dossier `models/` + des dépendances `torch` / `numpy` dans `requirements.txt` : l'architecture est pensée pour accueillir une future stratégie par apprentissage (ex. `ml_ppo`), mais **seule la stratégie heuristique est branchée et utilisée aujourd'hui** (`main.py` n'instancie que `HeuristicStrategy`).

### 2.3 GUI (C++ / raylib)

La GUI se connecte en tant qu'équipe spéciale `GRAPHIC`, puis demande l'état initial du monde (`msz`, `mct`, `tna`, `sgt`) avant de traiter en continu les notifications poussées par le serveur. Le rendu est assuré par **raylib**, avec deux moteurs de rendu interchangeables (`Renderer2D` / `Renderer3D`) implémentant la même interface `IRenderer`.

---

## 3. Stratégie de l'IA

La stratégie ne repose pas sur de l'apprentissage automatique mais sur une **heuristique à rôles**, où chaque joueur bascule entre trois comportements selon son niveau et les messages reçus :

| Rôle | Quand | Comportement |
|-|-|-|
| `SOLO` | Niveau 1, niveau 2 sans aide trouvée, ou niveau ≥3 sans leader élu | Explore, se nourrit, cherche les ressources de son niveau, monte de niveau seul ou via une coordination ponctuelle |
| `LEADER` | Le premier joueur niveau 3 à s'auto-élire | Collecte **toutes** les ressources nécessaires pour grimper de 3 à 8 d'un coup, rassemble les followers, lance les incantations en chaîne |
| `FOLLOWER` | Tout joueur niveau ≥3 qui a reconnu un leader | Converge vers le leader, se positionne sur sa case, attend et participe à chaque incantation |

### 3.1 Boucle de décision (`tick`)

À chaque tour : annonce sa présence si nécessaire (`Broadcast PLAYER_<level>_<pid>`) → rafraîchit son inventaire et sa vision (`Inventory` + `Look`) → ramasse au passage ce qui lui manque sur sa case → exécute la logique propre à son rôle actuel.

### 3.2 Gestion de la survie (nourriture)

Deux seuils pilotent tout le comportement :

- **seuil critique (`20`)** : la progression est suspendue, l'IA fonce sur la nourriture la plus proche en priorité absolue (`_grind_low_food`) ;
- **seuil bas (`40`)** : sert de garde-fou avant de s'engager dans une incantation ou un ralliement, pour ne jamais lancer un rituel avec un stock de food trop juste.

### 3.3 Exploration et collecte

Quand aucune coordination n'est en cours, l'IA explore en avançant tout en tournant périodiquement (motif gauche/droite) et choisit, dans son champ de vision, la **meilleure case candidate** pour une ressource donnée : un score combine la quantité de ressources sur la case, la distance (pénalité par ligne et par colonne) et un bonus pour les cases dans l'axe central — avec une pondération renforcée pour la nourriture quand le stock est critique.

### 3.4 Montée de niveau 1 → 2 : en solo

Le passage du niveau 1 au niveau 2 ne demande qu'1 joueur et 1 linemate : dès que les conditions sont réunies, le joueur lance seul son `Incantation`, sans coordination.

### 3.5 Montée de niveau 2 → 3 : coordination par paire

Le palier 2→3 demande 2 joueurs de même niveau. Un joueur prêt lance un appel général (`HELP_`), attend qu'un pair se manifeste, puis bascule en dialogue **adressé** avec ce pair précis (`HERE_FOR_HELP_` / `HELP2_`) pour converger sur la même case, lancer l'incantation à deux, puis prévenir tout le monde du succès (`SUCCESS_HELP_`). Voir le détail des messages en [section 4](#4-les-broadcasts-de-lia).

### 3.6 Montée de niveau 3 → 8 : le "convoi" du leader

C'est le cœur de la stratégie, et l'explication de la rapidité observée en jeu :

1. **Élection** : le premier joueur niveau 3 qui ne voit aucun leader annoncé, et qui a le plus grand identifiant parmi les joueurs niveau 3 connus, s'auto-proclame `LEADER` (`Broadcast I_AM_LEADER_<level>_<pid>`). Ce critère évite que deux joueurs s'élisent leader en même temps.
2. **Ravitaillement massif** : au lieu de ne récolter que les ressources du palier en cours, le leader accumule **la somme cumulée des besoins des paliers 3→4 jusqu'à 7→8** (9 linemate, 8 deraumere, 10 sibur, 5 mendiane, 6 phiras, 1 thystame). Une fois ce stock complet, il n'aura plus jamais besoin de ressortir chercher des pierres.
3. **Ralliement** : le leader diffuse en boucle `Broadcast INCANT_<level>_<pid>`. Chaque `FOLLOWER` niveau ≥3 calcule, grâce à la direction sonore renvoyée par le serveur, le chemin le plus court vers le leader et avance jusqu'à atteindre sa case (direction `0`).
4. **Incantations en chaîne** : dès que 6 joueurs sont réunis sur la case du leader, celui-ci pose lui-même les ressources nécessaires (`Set <ressource>`, prélevées sur son propre inventaire — les followers n'ont donc pas besoin de porter de pierres) et lance `Incantation`. Le groupe entier monte d'un niveau (1→2... non, 3→4 ici) **en une seule case**, sans jamais se disperser, puis recommence immédiatement le palier suivant (4→5, 5→6, 6→7, 7→8) avec le même groupe et le même stock de ressources.

Cette mise en commun des ressources par un seul "porteur" est ce qui permet à toute une équipe de passer du niveau 3 au niveau 8 en une seule réunion, sans aller-retours répétés — d'où la performance observée d'environ **100 secondes pour atteindre le niveau 8** (`f=100`).

### 3.7 Partage de nourriture pendant le rassemblement

Pendant qu'ils attendent sur la case du leader (potentiellement longtemps, le temps de réaliser 5 incantations successives), les followers font tourner un petit "pot commun" de nourriture : un joueur qui a plus de 40 food en dépose une unité sur la case (`Set food`), un joueur qui tombe sous le seuil critique en reprend une (`Take food`). Cela évite qu'un membre du groupe ne meure de faim alors que le convoi grimpe les niveaux.

---

## 4. Les broadcasts de l'IA

Tous les messages texte envoyés par `Broadcast` suivent une convention `MOT-CLÉ_<arguments>`, interprétée par chaque IA réceptrice. Le destinataire connaît uniquement la **direction** sonore (0 à 8, fournie par le serveur) et le texte — jamais l'identité réelle de l'émetteur ; c'est donc le contenu du message qui sert de "carte de visite".

| Broadcast | Émetteur / déclencheur | Rôle |
|-|-|-|
| `PLAYER_<level>_<pid>` | Tout joueur, dès la connexion puis chaque fois qu'il croise un identifiant inconnu ou vient de monter de niveau | Annonce générale de présence + niveau ; alimente la table interne `others` (qui sait qui est à quel niveau) |
| `HELP_<level>_<pid>` | Joueur niveau 2 prêt à monter mais seul | Appel **général** à l'aide pour un palier nécessitant 2 joueurs |
| `HERE_FOR_HELP_<recPID>_<level>_<pid>` | Joueur qui répond à un `HELP_` et vient d'arriver sur la case de l'appelant (direction = 0) | Message **adressé** ("je suis arrivé") qui permet à l'appelant d'identifier précisément son aidant |
| `HELP2_<recPID>_<level>_<pid>` | L'appelant initial, une fois l'aidant identifié | Bascule la conversation en dialogue **adressé** entre les deux joueurs pour finaliser le ralliement |
| `SUCCESS_HELP_<level>_<pid>` | Le joueur qui vient de réussir une incantation à deux | Signale la fin du rituel pour que l'aidant relâche son rôle et reprenne une activité normale |
| `I_AM_LEADER_<level>_<pid>` | Le joueur niveau 3 qui s'auto-élit (plus grand identifiant parmi les niveaux 3 connus, aucun leader déjà annoncé) | Élection du leader ; tous les joueurs niveau ≥3 qui le reçoivent passent en `FOLLOWER` |
| `INCANT_<level>_<pid>` | Le `LEADER`, en boucle, pendant toute la phase de ralliement et entre chaque incantation | Guide les followers vers sa position (via la direction sonore) et indique le palier en cours |

### Pourquoi les broadcasts sont indispensables

Le protocole Zappy ne fournit **aucun canal direct entre joueurs** : tout passe par le son. Les broadcasts permettent donc, sans aucune connaissance préalable des autres joueurs :

- de construire une vue partagée de "qui est à quel niveau" (`PLAYER_`) ;
- d'organiser un ralliement à deux pour les paliers à faible effectif (`HELP_` / `HERE_FOR_HELP_` / `HELP2_` / `SUCCESS_HELP_`) ;
- d'élire un unique leader sans coordination centrale et de rallier tout un groupe autour de lui (`I_AM_LEADER_` / `INCANT_`) ;
- de naviguer à l'aveugle vers un coéquipier en ne connaissant que la direction du son, grâce à des messages répétés qui permettent de recalculer le chemin à chaque tour.

---

## 5. Fonctionnalités de la GUI

### 5.1 Connexion et chargement

Au lancement, la GUI s'authentifie avec le nom d'équipe réservé `GRAPHIC`, puis demande successivement la taille du monde (`msz`), le contenu complet de la carte (`mct`), la liste des équipes (`tna`) et l'unité de temps courante (`sgt`). Un écran "Connecting..." puis "Loading map..." s'affiche tant que ces informations n'ont pas été reçues.

### 5.2 Rendu 2D (par défaut)

- carte vue du dessus, une case = un carré vert, avec les ressources représentées par des petits points colorés (jusqu'à leur quantité affichée en chiffre si la case est suffisamment grande) ;
- joueurs représentés par un triangle coloré par équipe, orienté selon leur direction, avec leur niveau affiché au-dessus ;
- déplacement des joueurs **interpolé** entre deux positions reçues, pour un mouvement fluide plutôt que des sauts case par case (avec gestion du cas où le joueur traverse un bord du monde torique : on "snap" plutôt que de le faire glisser à travers tout l'écran) ;
- légende des 7 ressources affichée en permanence en haut à gauche.

### 5.3 Rendu 3D (bascule avec `Tab`)

- caméra libre en orbite autour de la carte : clic droit + glisser pour tourner, molette pour zoomer, clic du milieu (ou flèches du clavier) pour faire un panoramique, `R` pour réinitialiser la caméra ;
- le sol est un damier de cubes plats, les ressources sont de petits cubes colorés, les joueurs sont des cubes orientés (avec une petite excroissance noire indiquant la direction regardée), et les œufs sont des sphères ;
- sélection d'un joueur par lancer de rayon (raycast) depuis la position de la souris.

### 5.4 Effets visuels animés

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

### 5.5 Panneau d'information (HUD)

Un panneau latéral, toujours visible, affiche :

- taille de la carte, nombre de joueurs connectés, nombre d'œufs en attente d'éclosion ;
- unité de temps courante (modifiable au clavier, voir contrôles) ;
- liste des équipes avec un point de couleur et le nombre de joueurs vivants par équipe ;
- au clic gauche sur un joueur : sa fiche détaillée (identifiant, équipe, niveau, position, direction, quantité de nourriture **et** estimation du temps de vie restant en secondes, et son inventaire complet) ;
- un journal défilant des 6 derniers messages serveur (`smg`).

### 5.6 Contrôles clavier / souris

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

### 5.7 Protocole serveur → GUI implémenté

La GUI implémente le protocole graphique de référence du sujet (même nomenclature de tags à
3 lettres) :

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

---

## 6. Conclusion

Le projet articule trois composants spécialisés (serveur Rust event-driven, IA Python à rôles, GUI C++ raylib 2D/3D) qui communiquent uniquement via le protocole réseau du sujet. La force de l'implémentation tient surtout à la stratégie d'élévation 3→8 : en faisant porter l'intégralité des ressources nécessaires par un seul leader et en gardant le même groupe de joueurs assemblé d'un palier à l'autre, l'équipe évite tout aller-retour inutile et atteint le niveau maximum en une fraction du temps qu'une approche naïve (récolte palier par palier, dispersion entre chaque
incantation) aurait demandé.
