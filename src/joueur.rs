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
            Amelioration::Soin => "Soigner 20 PV",
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

    // `as_ref` : transforme `&Option<Item>` en `Option<&Item>`, pour prêter l'objet sans le déplacer.
    pub fn equipement(&self, emplacement: Emplacement) -> Option<&Item> {
        self.equipement[emplacement.index()].as_ref()
    }

    // `item: Item` (sans `&`) : le joueur PREND possession de l'objet.
    pub fn equiper(&mut self, item: Item, journal: &mut Vec<String>) {
        journal.push(format!("Vous équipez : {}.", item.description()));

        // `&mut Option<Item>` : une référence vers le bon emplacement, qu'on va modifier.
        let place = &mut self.equipement[item.emplacement.index()];
        // `replace` met le nouvel objet et rend l'ancien (s'il y en avait un).
        if let Some(ancien) = place.replace(item) {
            journal.push(format!("{} est abandonné.", ancien.nom));
        }

        // Si la nouvelle armure donne moins de PV max, la vie ne doit pas dépasser le maximum.
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
            Amelioration::Soin => self.soigner(20, journal),
            Amelioration::VieMax => self.vie_max += 20,
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
        journal.push(format!("Vous vous soignez de {} PV.", self.vie - vie_avant));
    }

    pub fn attaquer(&self, cible: &mut Enemie, journal: &mut Vec<String>) {
        let mut degats = self.degats_total();
        if rand::random::<f64>() <= self.tx_critique_total() {
            degats += self.dgt_critique_total();
            journal.push(format!("{} réalise un coup critique !", self.nom));
        }

        cible.subir_degats(degats);
        journal.push(format!("Vous infligez {} dégâts à {}.", degats, cible.nom));
    }

    // `&Enemie` : on lit seulement l'ennemi, pas besoin de `&mut`.
    pub fn bloquer(&self, attaquant: &Enemie, journal: &mut Vec<String>) {
        journal.push(format!(
            "Vous bloquez l'attaque de {} : 0 dégât subi !",
            attaquant.nom
        ));
    }
}
