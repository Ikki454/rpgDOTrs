use ratatui::crossterm::event::KeyCode;

use crate::combat::{self, Action, Issue};
use crate::enemie::Enemie;
use crate::item::Item;
use crate::joueur::{Amelioration, Joueur};

// Les deux choix possibles devant un coffre, dans l'ordre du menu.
pub const CHOIX_BUTIN: [&str; 2] = ["Équiper", "Laisser"];

pub const DERNIER_ETAGE: u32 = 100;

// Machine à états : le jeu est toujours dans UN de ces écrans.
// Chaque variante transporte les données dont elle a besoin (l'ennemi n'existe que pendant un combat).
pub enum Ecran {
    SaisieNom {
        nom: String,
    },
    Combat {
        ennemi: Enemie,
        selection: usize,
    },
    Amelioration {
        selection: usize,
    },
    // `Option<Item>` : l'objet est dans le coffre (`Some`) jusqu'à ce qu'on le sorte avec `take()`.
    Butin {
        item: Option<Item>,
        selection: usize,
    },
    Fin {
        victoire: bool,
    },
}

pub struct Game {
    pub joueur: Joueur,
    pub etage: u32,
    pub ecran: Ecran,
    pub journal: Vec<String>,
    pub quitter: bool,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            joueur: Joueur::creer_joueur(String::new()),
            etage: 1,
            ecran: Ecran::SaisieNom { nom: String::new() },
            journal: Vec::new(),
            quitter: false,
        }
    }
}

impl Game {
    // Appelée par main.rs à chaque touche pressée.
    pub fn gerer_touche(&mut self, touche: KeyCode) {
        if touche == KeyCode::Esc {
            self.quitter = true;
            return;
        }

        // `&mut self.ecran` : on emprunte l'écran pour modifier `nom` ou `selection` sur place.
        match &mut self.ecran {
            Ecran::SaisieNom { nom } => match touche {
                KeyCode::Char(c) if nom.chars().count() < 20 => nom.push(c),
                KeyCode::Backspace => {
                    nom.pop();
                }
                KeyCode::Enter if !nom.trim().is_empty() => {
                    self.joueur.nom = nom.trim().to_string();
                    // Ici `nom` n'est plus utilisé : l'emprunt de `self.ecran` est terminé,
                    // donc on peut de nouveau utiliser `self` en entier.
                    self.journal
                        .push(format!("Bienvenue dans le donjon, {} !", self.joueur.nom));
                    self.entrer_etage();
                }
                _ => {}
            },

            Ecran::Combat { ennemi, selection } => {
                if let Some(i) = choisir(touche, selection, Action::TOUTES.len()) {
                    let issue = combat::jouer_tour(
                        &mut self.joueur,
                        ennemi,
                        Action::TOUTES[i],
                        &mut self.journal,
                    );
                    self.apres_combat(issue);
                }
            }

            Ecran::Amelioration { selection } => {
                if let Some(i) = choisir(touche, selection, Amelioration::TOUTES.len()) {
                    self.joueur
                        .ameliorer(Amelioration::TOUTES[i], &mut self.journal);
                    self.etage_suivant();
                }
            }

            Ecran::Butin { item, selection } => {
                if let Some(i) = choisir(touche, selection, CHOIX_BUTIN.len()) {
                    // `take()` sort l'objet du coffre et laisse `None` à sa place.
                    // On ne peut pas simplement "prendre" `item` : il est emprunté depuis `self.ecran`.
                    if i == 0
                        && let Some(item) = item.take()
                    {
                        self.joueur.equiper(item, &mut self.journal);
                    } else {
                        self.journal.push("Vous laissez l'objet.".to_string());
                    }
                    self.etage_suivant();
                }
            }

            Ecran::Fin { .. } => match touche {
                KeyCode::Char('r') => *self = Game::default(),
                KeyCode::Char('q') | KeyCode::Enter => self.quitter = true,
                _ => {}
            },
        }
    }

    fn entrer_etage(&mut self) {
        if self.etage > DERNIER_ETAGE {
            self.journal
                .push("Vous avez terminé le donjon !".to_string());
            self.ecran = Ecran::Fin { victoire: true };
            return;
        }

        self.journal.push(format!("── Étage {} ──", self.etage));

        if self.etage.is_multiple_of(10) {
            self.lancer_combat();
        } else if self.etage.is_multiple_of(5) {
            self.journal.push("Salle des coffres !".to_string());
            self.ouvrir_coffre();
        } else {
            self.lancer_combat();
        }
    }

    fn lancer_combat(&mut self) {
        let ennemi = Enemie::creer_ennemi(self.etage);
        if self.etage.is_multiple_of(10) {
            self.journal
                .push(format!("Un BOSS apparaît : {} !", ennemi.nom));
        } else {
            self.journal.push(format!("Un {} apparaît !", ennemi.nom));
        }

        // Chaque combat commence avec toutes les attaques, blocages et soins.
        self.joueur.reset_skill();
        // `ennemi` est déplacé (move) dans l'écran : c'est maintenant l'écran qui le possède.
        self.ecran = Ecran::Combat {
            ennemi,
            selection: 0,
        };
    }

    fn apres_combat(&mut self, issue: Issue) {
        match issue {
            Issue::EnCours => {}
            // Après un boss : coffre (soin + objet). Après un ennemi normal : amélioration.
            Issue::Victoire if self.etage.is_multiple_of(10) => {
                self.journal.push("Le boss laisse un coffre !".to_string());
                self.ouvrir_coffre();
            }
            Issue::Victoire => {
                self.journal
                    .push("Choisissez une amélioration.".to_string());
                self.ecran = Ecran::Amelioration { selection: 0 };
            }
            Issue::Fuite => self.etage_suivant(),
            Issue::Defaite => self.ecran = Ecran::Fin { victoire: false },
        }
    }

    // Coffre : soigne entièrement le joueur et propose un objet aléatoire.
    fn ouvrir_coffre(&mut self) {
        self.joueur.soigner_completement(&mut self.journal);

        let item = Item::aleatoire(self.etage);
        self.journal
            .push(format!("Le coffre contient : {}", item.description()));
        self.ecran = Ecran::Butin {
            item: Some(item),
            selection: 0,
        };
    }

    fn etage_suivant(&mut self) {
        self.etage += 1;
        self.entrer_etage();
    }
}

// Navigation commune aux menus : flèches pour déplacer, Entrée ou chiffre pour valider.
// Renvoie `Some(index)` quand un choix est validé, `None` sinon.
fn choisir(touche: KeyCode, selection: &mut usize, taille: usize) -> Option<usize> {
    match touche {
        // `+ taille - 1` puis `%` : remonter depuis le haut revient en bas (sans passer sous 0).
        KeyCode::Up => *selection = (*selection + taille - 1) % taille,
        KeyCode::Down => *selection = (*selection + 1) % taille,
        KeyCode::Enter => return Some(*selection),
        // `to_digit` : '3' devient Some(3). On vérifie que le chiffre est dans le menu.
        KeyCode::Char(c) => {
            if let Some(chiffre) = c.to_digit(10)
                && (1..=taille).contains(&(chiffre as usize))
            {
                return Some(chiffre as usize - 1);
            }
        }
        _ => {}
    }
    None
}

// `cfg(test)` : ce module n'est compilé que par `cargo test`, jamais dans le jeu.
#[cfg(test)]
mod tests {
    use super::*;

    // Petite aide : crée une partie et valide le nom "Test".
    fn partie_commencee() -> Game {
        let mut game = Game::default();
        for c in "Test".chars() {
            game.gerer_touche(KeyCode::Char(c));
        }
        game.gerer_touche(KeyCode::Enter);
        game
    }

    #[test]
    fn le_nom_lance_le_premier_combat() {
        let game = partie_commencee();
        assert_eq!(game.joueur.nom, "Test");
        assert_eq!(game.etage, 1);
        // `matches!` : vrai si la valeur correspond au motif.
        assert!(matches!(game.ecran, Ecran::Combat { .. }));
    }

    #[test]
    fn l_etage_5_est_une_salle_des_coffres() {
        let mut game = partie_commencee();
        game.joueur.vie = 10;
        for _ in 0..4 {
            game.gerer_touche(KeyCode::Char('4'));
        }
        assert_eq!(game.etage, 5);
        // `Some(_)` dans le motif : on vérifie aussi qu'il y a bien un objet dans le coffre.
        assert!(matches!(game.ecran, Ecran::Butin { item: Some(_), .. }));
        assert_eq!(game.joueur.vie, game.joueur.vie_max_total());
    }

    #[test]
    fn equiper_l_objet_du_coffre() {
        let mut game = partie_commencee();
        for _ in 0..4 {
            game.gerer_touche(KeyCode::Char('4'));
        }
        // Touche '1' = "Équiper" : l'objet quitte le coffre et va dans un emplacement du joueur.
        game.gerer_touche(KeyCode::Char('1'));
        // `Option::is_some` passé directement comme fonction, sans écrire de closure.
        assert!(game.joueur.equipement.iter().any(Option::is_some));
        assert_eq!(game.etage, 6);
    }
}
