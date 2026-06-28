# IA Zappy (`zappy_ai`, Python)

Le client IA pilote un joueur de façon autonome. Il se connecte au serveur, maintient une vue locale du jeu, et décide à chaque tour des commandes à envoyer.

---

## 1. Architecture (Python / asyncio)

Le client est construit autour d'une boucle `asyncio` :

- `ZappyConnection` gère le handshake (`WELCOME` / team name / nb de slots / dimensions) puis une tâche `asyncio` de lecture (`_reader_loop`) qui distingue les réponses **sollicitées** (résultat de la dernière commande envoyée) des messages **non sollicités** (`message`, `eject:`, `Current level:`, `dead`), redirigés vers un callback ;
- `GameState` garde l'inventaire, la vision, le niveau et les besoins en ressources pour l'élévation courante ;
- `HeuristicStrategy` est le cerveau du joueur : à chaque `tick()`, elle lit l'état du jeu et envoie 0 à N commandes (voir [section 2](#2-stratégie-de-lia)).

> Le code définit une interface `Strategy` abstraite (`strategy/base.py`) : l'architecture est pensée pour accueillir d'autres stratégies à l'avenir, mais **seule la stratégie heuristique est branchée et utilisée aujourd'hui** (`main.py` n'instancie que `HeuristicStrategy`).

---

## 2. Stratégie de l'IA

La stratégie ne repose pas sur de l'apprentissage automatique mais sur une **heuristique à rôles**, où chaque joueur bascule entre trois comportements selon son niveau et les messages reçus :

| Rôle | Quand | Comportement |
|-|-|-|
| `SOLO` | Niveau 1, niveau 2 sans aide trouvée, ou niveau ≥3 sans leader élu | Explore, se nourrit, cherche les ressources de son niveau, monte de niveau seul ou via une coordination ponctuelle |
| `LEADER` | Le premier joueur niveau 3 à s'auto-élire | Collecte **toutes** les ressources nécessaires pour grimper de 3 à 8 d'un coup, rassemble les followers, lance les incantations en chaîne |
| `FOLLOWER` | Tout joueur niveau ≥3 qui a reconnu un leader | Converge vers le leader, se positionne sur sa case, attend et participe à chaque incantation |

### 2.1 Boucle de décision (`tick`)

À chaque tour : annonce sa présence si nécessaire (`Broadcast PLAYER_<level>_<team_name>_<pid>`) → rafraîchit son inventaire et sa vision (`Inventory` + `Look`) → ramasse au passage ce qui lui manque sur sa case → exécute la logique propre à son rôle actuel.

### 2.2 Gestion de la survie (nourriture)

Deux seuils pilotent tout le comportement :

- **seuil critique (`20`)** : la progression est suspendue, l'IA fonce sur la nourriture la plus proche en priorité absolue (`_grind_low_food`) ;
- **seuil bas (`30`)** : sert de garde-fou avant de s'engager dans une incantation ou un ralliement, pour ne jamais lancer un rituel avec un stock de food trop juste.

### 2.3 Exploration et collecte

Quand aucune coordination n'est en cours, l'IA explore en avançant tout en tournant périodiquement (motif gauche/droite) et choisit, dans son champ de vision, la **meilleure case candidate** pour une ressource donnée : un score combine la quantité de ressources sur la case, la distance (pénalité par ligne et par colonne) et un bonus pour les cases dans l'axe central avec une pondération renforcée pour la nourriture quand le stock est critique.

### 2.4 Montée de niveau 1 → 2 : en solo

Le passage du niveau 1 au niveau 2 ne demande qu'1 joueur et 1 linemate : dès que les conditions sont réunies, le joueur lance seul son `Incantation`, sans coordination.

### 2.5 Montée de niveau 2 → 3 : coordination par paire

Le palier 2→3 demande 2 joueurs de même niveau. Un joueur prêt lance un appel général (`HELP_`), attend qu'un pair se manifeste, puis bascule en dialogue **adressé** avec ce pair précis (`HERE_FOR_HELP_` / `HELP2_`) pour converger sur la même case, lancer l'incantation à deux, puis prévenir tout le monde du succès (`SUCCESS_HELP_`). Voir le détail des messages en [section 3](#3-les-broadcasts-de-lia).

### 2.6 Montée de niveau 3 → 8 : le "convoi" du leader

C'est le cœur de la stratégie, et l'explication de la rapidité observée en jeu :

1. **Élection** : le premier joueur niveau 3 qui ne voit aucun leader annoncé, et qui a le plus grand identifiant parmi les joueurs niveau 3 connus, s'auto-proclame `LEADER` (`Broadcast I_AM_LEADER_<level>_<team_name>_<pid>`). Ce critère évite que deux joueurs s'élisent leader en même temps.
2. **Ravitaillement massif** : au lieu de ne récolter que les ressources du palier en cours, le leader accumule **la somme cumulée des besoins des paliers 3→4 jusqu'à 7→8** (9 linemate, 8 deraumere, 10 sibur, 5 mendiane, 6 phiras, 1 thystame). Une fois ce stock complet, il n'aura plus jamais besoin de ressortir chercher des pierres.
3. **Ralliement** : le leader diffuse en boucle `Broadcast INCANT_<level>_<pid>`. Chaque `FOLLOWER` niveau ≥3 calcule, grâce à la direction sonore renvoyée par le serveur, le chemin le plus court vers le leader et avance jusqu'à atteindre sa case (direction `0`).
4. **Incantations en chaîne** : dès que 6 joueurs sont réunis sur la case du leader, celui-ci pose lui-même les ressources nécessaires (`Set <ressource>`, prélevées sur son propre inventaire, les followers n'ont donc pas besoin de porter de pierres) et lance `Incantation`. Le groupe entier monte d'un niveau (3→4 ici) **en une seule case**, sans jamais se disperser, puis recommence immédiatement le palier suivant (4→5, 5→6, 6→7, 7→8) avec le même groupe et le même stock de ressources.

Cette mise en commun des ressources par un seul "porteur" est ce qui permet à toute une équipe de passer du niveau 3 au niveau 8 en une seule réunion, sans aller-retours répétés, d'où la performance observée d'environ **100 secondes pour atteindre le niveau 8** (`f=100`).

### 2.7 Partage de nourriture pendant le rassemblement

Pendant qu'ils attendent sur la case du leader (potentiellement longtemps, le temps de réaliser 5 incantations successives), les followers font tourner un petit "pot commun" de nourriture : un joueur qui a plus de 30 food en dépose une unité sur la case (`Set food`), un joueur qui tombe sous le seuil critique en reprend une (`Take food`). Cela évite qu'un membre du groupe ne meure de faim alors que le convoi grimpe les niveaux.

---

## 3. Les broadcasts de l'IA

Tous les messages texte envoyés par `Broadcast` suivent une convention `MOT-CLÉ_<arguments>`, interprétée par chaque IA réceptrice. Le destinataire connaît uniquement la **direction** sonore (0 à 8, fournie par le serveur) et le texte, jamais l'identité réelle de l'émetteur ; c'est donc le contenu du message qui sert de "carte de visite".

| Broadcast | Émetteur / déclencheur | Rôle |
|-|-|-|
| `PLAYER_<level>_<team_name>_<pid>` | Tout joueur, dès la connexion puis chaque fois qu'il croise un identifiant inconnu ou vient de monter de niveau | Annonce générale de présence + niveau ; le `team_name` filtre les émetteurs adverses, alimente la table interne `others` (qui sait qui est à quel niveau) |
| `HELP_<level>_<pid>` | Joueur niveau 2 prêt à monter mais seul | Appel **général** à l'aide pour un palier nécessitant 2 joueurs |
| `HERE_FOR_HELP_<recPID>_<level>_<pid>` | Joueur qui répond à un `HELP_` et vient d'arriver sur la case de l'appelant (direction = 0) | Message **adressé** ("je suis arrivé") qui permet à l'appelant d'identifier précisément son aidant |
| `HELP2_<recPID>_<level>_<pid>` | L'appelant initial, une fois l'aidant identifié | Bascule la conversation en dialogue **adressé** entre les deux joueurs pour finaliser le ralliement |
| `SUCCESS_HELP_<level>_<pid>` | Le joueur qui vient de réussir une incantation à deux | Signale la fin du rituel pour que l'aidant relâche son rôle et reprenne une activité normale |
| `I_AM_LEADER_<level>_<team_name>_<pid>` | Le joueur niveau 3 qui s'auto-élit (plus grand identifiant parmi les niveaux 3 connus, aucun leader déjà annoncé) | Élection du leader ; tous les joueurs niveau ≥3 de la même équipe qui le reçoivent passent en `FOLLOWER` |
| `I_AM_DEAD_<pid>` | Tout joueur dont la réserve de nourriture tombe au plus bas (mort imminente de faim) | Signale sa disparition pour que les autres le retirent de leur table `others` (et qu'il ne soit plus attendu pour une incantation) |
| `INCANT_<level>_<pid>` | Le `LEADER`, en boucle, pendant toute la phase de ralliement et entre chaque incantation | Guide les followers vers sa position (via la direction sonore) et indique le palier en cours |

### Pourquoi les broadcasts sont indispensables

Le protocole Zappy ne fournit **aucun canal direct entre joueurs** : tout passe par le son. Les broadcasts permettent donc, sans aucune connaissance préalable des autres joueurs :

- de construire une vue partagée de "qui est à quel niveau" (`PLAYER_`) ;
- d'organiser un ralliement à deux pour les paliers à faible effectif (`HELP_` / `HERE_FOR_HELP_` / `HELP2_` / `SUCCESS_HELP_`) ;
- d'élire un unique leader sans coordination centrale et de rallier tout un groupe autour de lui (`I_AM_LEADER_` / `INCANT_`) ;
- de retirer proprement un joueur mort de la planification (`I_AM_DEAD_`) ;
- de naviguer à l'aveugle vers un coéquipier en ne connaissant que la direction du son, grâce à des messages répétés qui permettent de recalculer le chemin à chaque tour.

---

## 4. Commandes émises par l'IA

Au-delà des broadcasts inter-IA, l'heuristique envoie au serveur les commandes du protocole IA (coûts et réponses détaillés dans la [fiche serveur](server.md#4-jeu-de-commandes-ia-couche-commandrs)) :

| Commande | Usage dans la stratégie |
|-|-|
| `Inventory` | rafraîchit l'inventaire à chaque `tick` (et sert à mesurer la fréquence `f` au démarrage) |
| `Look` | rafraîchit la vision, base de la sélection de case candidate |
| `Forward` `Left` `Right` | déplacement et exploration (motif gauche/droite), convergence vers une case ou une direction sonore |
| `Take <obj>` | ramasse food et pierres (sur la case courante ou la meilleure case visée) |
| `Set <obj>` | le leader pose les pierres avant incantation ; partage de food dans le "pot commun" |
| `Broadcast <texte>` | tous les messages de coordination (§3) |
| `Incantation` | lance le rituel d'élévation, puis attend l'événement `Current level:` |
| `Fork` | le candidat niveau 2 complète l'effectif de l'équipe avant un palier (via `Connect_nbr`) |
| `Connect_nbr` | compte les slots libres pour décider combien de `Fork` lancer |
| `Eject` | dégage une case surchargée (plus de joueurs que nécessaire pour le palier) |
