use crate::enemie::Enemie;
use crate::joueur::Joueur;

// Action choisie par le joueur dans le menu de l'UI.
#[derive(Clone, Copy)]
pub enum Action {
    Attaquer,
    Bloquer,
    Soigner,
    Fuir,
}

impl Action {
    // L'ordre de ce tableau = l'ordre du menu à l'écran.
    pub const TOUTES: [Action; 4] = [
        Action::Attaquer,
        Action::Bloquer,
        Action::Soigner,
        Action::Fuir,
    ];
}

// Ce que le jeu doit faire après un tour de combat.
pub enum Issue {
    EnCours,
    Victoire,
    Defaite,
    Fuite,
}

// Résultat d'une action du joueur, utilisé seulement dans ce module.
enum Tour {
    Impossible, // ressource vide : on redemande, l'ennemi ne joue pas
    Joue,       // action faite : l'ennemi riposte
    Bloque,     // action faite : l'ennemi ne riposte pas
}

// Avant : une boucle qui lisait le clavier jusqu'à la fin du combat.
// Maintenant : UN seul tour. C'est la boucle de l'UI (main.rs) qui rappelle cette fonction
// à chaque touche, et l'`Issue` lui dit où en est le combat.
pub fn jouer_tour(
    joueur: &mut Joueur,
    ennemie: &mut Enemie,
    action: Action,
    journal: &mut Vec<String>,
) -> Issue {
    let tour = match action {
        Action::Attaquer => attaquer(joueur, ennemie, journal),
        Action::Bloquer => bloquer(joueur, ennemie, journal),
        Action::Soigner => soigner(joueur, journal),
        Action::Fuir => {
            journal.push("Vous fuyez le combat.".to_string());
            return Issue::Fuite;
        }
    };

    match tour {
        // Rien ne s'est passé : même tour, l'attaque annoncée reste la même.
        Tour::Impossible => return Issue::EnCours,
        // L'attaque de l'ennemi (puissante ou non) est arrêtée par le blocage.
        Tour::Bloque => {}
        Tour::Joue => {
            if ennemie.vie < 1 {
                journal.push(format!("Vous avez vaincu {} !", ennemie.nom));
                return Issue::Victoire;
            }

            ennemie.attaquer(joueur, journal);

            if joueur.vie < 1 {
                journal.push(format!("Vous avez été vaincu par {} !", ennemie.nom));
                return Issue::Defaite;
            }
        }
    }

    // Début du tour suivant : l'ennemi choisit (et annonce) sa prochaine attaque.
    ennemie.preparer_attaque(journal);
    Issue::EnCours
}

// Multiplicateur des dégâts subis : 100 / (100 + armure).
// 0 armure = ×1.0, 100 armure = ×0.5, 300 armure = ×0.25 : l'armure aide toujours,
// mais ne rend jamais invincible.
pub fn multiplicateur_armure(armure: i32) -> f64 {
    // `max(0)` : une armure négative ne doit pas augmenter les dégâts (ni diviser par 0).
    100.0 / (100.0 + armure.max(0) as f64)
}

// Dégâts réellement subis après l'armure. Toujours au moins 1 : un coup fait toujours mal.
pub fn reduire_degats(degats: i32, armure: i32) -> i32 {
    ((degats as f64 * multiplicateur_armure(armure)).round() as i32).max(1)
}

fn attaquer(joueur: &mut Joueur, ennemie: &mut Enemie, journal: &mut Vec<String>) -> Tour {
    if joueur.atk_left < 1 {
        journal.push("Plus d'attaques disponibles !".to_string());
        return Tour::Impossible;
    }
    joueur.attaquer(ennemie, journal);
    joueur.atk_left -= 1;
    Tour::Joue
}

fn bloquer(joueur: &mut Joueur, ennemie: &Enemie, journal: &mut Vec<String>) -> Tour {
    if joueur.block_left < 1 {
        journal.push("Plus de blocages disponibles !".to_string());
        return Tour::Impossible;
    }
    joueur.bloquer(ennemie, journal);
    joueur.block_left -= 1;
    Tour::Bloque
}

fn soigner(joueur: &mut Joueur, journal: &mut Vec<String>) -> Tour {
    if joueur.soign_left < 1 {
        journal.push("Plus de soins disponibles !".to_string());
        return Tour::Impossible;
    }
    // Le montant vient de la stat du joueur (améliorable), plus d'un 20 écrit en dur.
    joueur.soigner(joueur.soin, journal);
    joueur.soign_left -= 1;
    Tour::Joue
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l_armure_reduit_les_degats() {
        assert_eq!(reduire_degats(100, 0), 100);
        assert_eq!(reduire_degats(100, 100), 50);
        assert_eq!(reduire_degats(100, 300), 25);
        // Jamais moins de 1 dégât, même avec une armure énorme.
        assert_eq!(reduire_degats(1, 10_000), 1);
    }

    #[test]
    fn l_attaque_puissante_annoncee_frappe_en_critique() {
        let mut joueur = Joueur::creer_joueur("Test".to_string());
        let mut ennemi = Enemie::creer_ennemi(1);
        // On force l'annonce au lieu de compter sur le hasard.
        ennemi.prochain_critique = true;

        // Soigner : l'ennemi riposte avec l'attaque annoncée.
        jouer_tour(&mut joueur, &mut ennemi, Action::Soigner, &mut Vec::new());
        let attendu = reduire_degats(ennemi.degats + ennemi.dgt_critique, 0);
        assert_eq!(joueur.vie, 100 - attendu);
    }

    #[test]
    fn bloquer_arrete_l_attaque_puissante() {
        let mut joueur = Joueur::creer_joueur("Test".to_string());
        let mut ennemi = Enemie::creer_ennemi(1);
        ennemi.prochain_critique = true;

        jouer_tour(&mut joueur, &mut ennemi, Action::Bloquer, &mut Vec::new());
        assert_eq!(joueur.vie, 100);
    }
}
