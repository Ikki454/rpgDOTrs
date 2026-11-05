use crate::combat;
use crate::enemie::Enemie;
use crate::item::{Bonus, Emplacement, Item};

pub struct Joueur {
    pub nom: String,
    pub niveau: i32,

    pub vie: i32,
    pub vie_max: i32,

    pub degats: i32,
    pub tx_critique: f64,
    pub dgt_critique: i32,
    pub armure: i32,
    // PV rendus par un sort de soin.
    pub soin: i32,

    pub atk_left: i32,
    pub atk_max: i32,

    pub block_left: i32,
    pub block_max: i32,

    pub soign_left: i32,
    pub soign_max: i32,

    // Un emplacement par case, rangé par `Emplacement::index()`.
    // `Option` : soit `Some(item)`, soit `None` (emplacement vide). Pas de `null` en Rust.
    pub equipement: [Option<Item>; Emplacement::NOMBRE],
}

// `Clone, Copy` : une amélioration se copie comme un nombre, pas besoin de `&`.
#[derive(Clone, Copy)]
pub enum Amelioration {
    Degats,
    Blocage,
    Soin,
    VieMax,
}

// Les critiques ne s'améliorent plus ici : seulement grâce aux objets.
impl Amelioration {
    // Constante associée : toutes les améliorations, dans l'ordre du menu.
    pub const TOUTES: [Amelioration; 4] = [
        Amelioration::Degats,
        Amelioration::Blocage,
        Amelioration::Soin,
        Amelioration::VieMax,
    ];

    pub fn description(&self) -> &'static str {
        match self {
            Amelioration::Degats => "Dégâts +5",
            Amelioration::Blocage => "Blocages +1",
            Amelioration::Soin => "Soin +5 PV",
            Amelioration::VieMax => "PV max +20",
        }
    }
}

// Plus de `println!` ici : les messages vont dans le journal, et c'est l'UI qui les affiche.
impl Joueur {
    pub fn creer_joueur(nom: String) -> Joueur {
        Joueur {
            nom,
            niveau: 1,

            vie: 100,
            vie_max: 100,

            degats: 10,
            tx_critique: 0.2,
            dgt_critique: 20,
            // Pas d'armure de base : elle vient seulement des objets.
            armure: 0,
            soin: 20,

            atk_left: 10,
            atk_max: 10,

            block_left: 5,
            block_max: 5,

            soign_left: 3,
            soign_max: 3,

            // `[None; 8]` ne compile pas : cette syntaxe copie la valeur, et `Option<Item>` n'est pas `Copy`.
            // `[const { None }; 8]` crée chaque case à partir d'une constante, sans copie.
            equipement: [const { None }; Emplacement::NOMBRE],
        }
    }

    // Les champs `degats`, `vie_max`... sont les stats DE BASE (améliorations comprises).
    // Les stats réelles = base + bonus de l'équipement, recalculées à chaque fois :
    // impossible d'oublier de retirer le bonus d'un objet qu'on enlève.
    fn bonus_equipement(&self) -> Bonus {
        self.equipement
            .iter()
            // `flatten` sur des `&Option<Item>` : ignore les `None`, garde les objets.
            .flatten()
            .fold(Bonus::default(), |total, item| total + item.bonus)
    }

    pub fn degats_total(&self) -> i32 {
        self.degats + self.bonus_equipement().degats
    }

    pub fn vie_max_total(&self) -> i32 {
        self.vie_max + self.bonus_equipement().vie_max
    }

    pub fn tx_critique_total(&self) -> f64 {
        self.tx_critique + self.bonus_equipement().tx_critique
    }

    pub fn dgt_critique_total(&self) -> i32 {
        self.dgt_critique + self.bonus_equipement().dgt_critique
    }

    pub fn armure_total(&self) -> i32 {
        self.armure + self.bonus_equipement().armure
    }

    // Même logique que pour l'ennemi : l'armure réduit les dégâts, puis on retire la vie.
    pub fn subir_degats(&mut self, degats: i32) -> i32 {
        let subis = combat::reduire_degats(degats, self.armure_total());
        self.vie = (self.vie - subis).max(0);
        subis
    }

    // `as_ref` : transforme `&Option<Item>` en `Option<&Item>`, pour prêter l'objet sans le déplacer.
    pub fn equipement(&self, emplacement: Emplacement) -> Option<&Item> {
        self.equipement[emplacement.index()].as_ref()
    }

    // `item: Item` (sans `&`) : le joueur PREND possession de l'objet.
    pub fn equiper(&mut self, item: Item, journal: &mut Vec<String>) {
        journal.push(format!("Vous équipez : {}.", item.description()));
        // On retient les PV max AVANT le changement, pour connaître le gain ensuite.
        let vie_max_avant = self.vie_max_total();

        // `&mut Option<Item>` : une référence vers le bon emplacement, qu'on va modifier.
        let place = &mut self.equipement[item.emplacement.index()];
        // `replace` met le nouvel objet et rend l'ancien (s'il y en avait un).
        if let Some(ancien) = place.replace(item) {
            journal.push(format!("{} est abandonné.", ancien.nom));
        }

        // Comme l'amélioration : les PV max gagnés s'ajoutent aux PV actuels (80/100 → 110/130).
        // Si l'objet donne MOINS de PV max, `gain` est négatif et on plafonne seulement.
        let gain = self.vie_max_total() - vie_max_avant;
        if gain > 0 {
            self.vie += gain;
        }
        self.vie = self.vie.min(self.vie_max_total());
    }

    pub fn soigner_completement(&mut self, journal: &mut Vec<String>) {
        self.vie = self.vie_max_total();
        journal.push(format!("PV restaurés : {}/{}.", self.vie, self.vie));
    }

    pub fn reset_skill(&mut self) {
        self.atk_left = self.atk_max;
        self.block_left = self.block_max;
        self.soign_left = self.soign_max;
    }

    // Le choix arrive déjà décidé (par l'UI) : plus besoin de lire le clavier ici.
    pub fn ameliorer(&mut self, choix: Amelioration, journal: &mut Vec<String>) {
        match choix {
            Amelioration::Degats => self.degats += 5,
            // `block_max` et pas `block_left`, sinon reset_skill efface le bonus.
            Amelioration::Blocage => self.block_max += 1,
            // Améliore la stat de soin : chaque sort de soin rendra plus de PV.
            Amelioration::Soin => self.soin += 5,
            // Les PV gagnés s'ajoutent aussi aux PV actuels : 80/100 devient 100/120.
            Amelioration::VieMax => {
                self.vie_max += 20;
                self.vie += 20;
            }
        }

        self.niveau += 1;
        journal.push(format!(
            "Amélioration : {} ! Niveau {}.",
            choix.description(),
            self.niveau
        ));
    }

    pub fn soigner(&mut self, soin: i32, journal: &mut Vec<String>) {
        // `min` plafonne la vie à `vie_max`.
        let vie_avant = self.vie;
        self.vie = (self.vie + soin).min(self.vie_max_total());
        let rendus = self.vie - vie_avant;

        // Le journal montre les PV avant → après, pour ne pas confondre avec la riposte qui suit.
        let mut message = format!(
            "Soin : +{} PV ({} → {}/{}).",
            rendus,
            vie_avant,
            self.vie,
            self.vie_max_total()
        );
        // Si le plafond a coupé le soin, on le dit.
        if rendus < soin {
            message.push_str(" PV max atteints.");
        }
        journal.push(message);
    }

    pub fn attaquer(&self, cible: &mut Enemie, journal: &mut Vec<String>) {
        let mut degats = self.degats_total();
        if rand::random::<f64>() <= self.tx_critique_total() {
            degats += self.dgt_critique_total();
            journal.push(format!("{} réalise un coup critique !", self.nom));
        }

        // On affiche les dégâts APRÈS l'armure de l'ennemi.
        let subis = cible.subir_degats(degats);
        journal.push(format!("Vous infligez {} dégâts à {}.", subis, cible.nom));
    }

    // `&Enemie` : on lit seulement l'ennemi, pas besoin de `&mut`.
    pub fn bloquer(&self, attaquant: &Enemie, journal: &mut Vec<String>) {
        // `if` est une expression : il renvoie directement le bon texte.
        let attaque = if attaquant.prochain_critique {
            "l'attaque puissante"
        } else {
            "l'attaque"
        };
        journal.push(format!(
            "Vous bloquez {} de {} : 0 dégât subi !",
            attaque, attaquant.nom
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l_amelioration_pv_max_augmente_aussi_les_pv() {
        let mut joueur = Joueur::creer_joueur("Test".to_string());
        joueur.vie = 80;
        joueur.ameliorer(Amelioration::VieMax, &mut Vec::new());
        assert_eq!((joueur.vie, joueur.vie_max_total()), (100, 120));
    }

    #[test]
    fn equiper_des_pv_max_augmente_aussi_les_pv() {
        let mut joueur = Joueur::creer_joueur("Test".to_string());
        joueur.vie = 80;
        let torse = Item::creer(Emplacement::Torse, crate::item::Rarete::Commun, 1);
        let gain = torse.bonus.vie_max;
        joueur.equiper(torse, &mut Vec::new());
        assert_eq!(joueur.vie, 80 + gain);
        assert_eq!(joueur.vie_max_total(), 100 + gain);
    }

    #[test]
    fn l_amelioration_soin_augmente_le_montant_soigne() {
        let mut joueur = Joueur::creer_joueur("Test".to_string());
        joueur.ameliorer(Amelioration::Soin, &mut Vec::new());
        joueur.vie = 10;
        let soin = joueur.soin;
        joueur.soigner(soin, &mut Vec::new());
        assert_eq!(joueur.vie, 35);
    }
}
