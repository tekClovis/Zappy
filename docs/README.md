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

## 2. Documentation par composant

Chaque composant a sa propre fiche technique :

- [**Serveur** (`server.md`)](server.md) — architecture event-driven (reactor `poll`), modèle de temps, machine à états des connexions, jeu de commandes IA et leurs coûts, règles du monde (ressources, vision, géométrie, son, incantation).
- [**IA** (`ai.md`)](ai.md) — boucle `asyncio`, stratégie heuristique à rôles, gestion de la survie, montées de niveau 1→8, broadcasts inter-IA et commandes émises.
- [**GUI** (`gui.md`)](gui.md) — client spectateur raylib, rendus 2D/3D, effets visuels, HUD, contrôles et protocole graphique implémenté.

Pour la **compilation**, le **lancement** des binaires (flags CLI) et un rappel des **règles du jeu**, voir le [`README.md`](../README.md) à la racine.

---

## 3. Conclusion

Le projet articule trois composants spécialisés (serveur Rust event-driven, IA Python à rôles, GUI C++ raylib 2D/3D) qui communiquent uniquement via le protocole réseau du sujet. La force de l'implémentation tient surtout à la stratégie d'élévation 3→8 : en faisant porter l'intégralité des ressources nécessaires par un seul leader et en gardant le même groupe de joueurs assemblé d'un palier à l'autre, l'équipe évite tout aller-retour inutile et atteint le niveau maximum en une fraction du temps qu'une approche naïve (récolte palier par palier, dispersion entre chaque incantation) aurait demandé.
