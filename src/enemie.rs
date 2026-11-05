use crate::combat;
use crate::joueur::Joueur;

pub struct Enemie {
    pub enemy_type: EnemyType,
    pub enemy_rang: EnemyRang,

    pub nom: String,
    pub niveau: u32,

    pub vie: i32,
    pub vie_max: i32,
    pub degats: i32,
    pub tx_critique: f64,
    pub dgt_critique: i32,
    pub armure: i32,

    // `true` si la PROCHAINE attaque sera un coup critique (annoncé à l'avance).
    pub prochain_critique: bool,
}

// Réglages de difficulté : stat = (base + étage × par_étage) × multiplicateur du rang.
// Ennemi Normal, avant → maintenant :
//   étage 10 : 70 PV / 12 dégâts / 10 armure  →  50 PV /  9 dégâts /  5 armure
//   étage 50 : 270 PV / 52 dégâts / 50 armure → 170 PV / 33 dégâts / 25 armure
//   étage 100: 520 PV / 102 dégâts / 100 armure → 320 PV / 63 dégâts / 50 armure
const VIE_BASE: f64 = 20.0;
const VIE_PAR_ETAGE: f64 = 3.0; // avant : 5
const DEGATS_BASE: f64 = 3.0; // avant : 2
const DEGATS_PAR_ETAGE: f64 = 0.6; // avant : 1
const ARMURE_PAR_ETAGE: f64 = 0.5; // avant : 1

impl Enemie {
    pub fn creer_ennemi(etage: u32) -> Enemie {
        let enemy_type = EnemyType::from_floor(etage);
        let enemy_rang = EnemyRang::pick_rang(etage);
        let nom = enemy_type.nom().to_string();

        // Un `match` qui renvoie un tuple : tout ce qui dépend du rang, au même endroit.
        // Les critiques sont fixés par le rang : ils n'augmentent plus avec l'étage.
        let (multiplicateur, tx_critique, dgt_critique) = match enemy_rang {
            EnemyRang::Normal => (1.0, 0.10, 10),
            EnemyRang::Elite => (1.5, 0.20, 20),
            EnemyRang::Boss => (2.0, 0.30, 30),
        };

        // `as` convertit explicitement : Rust ne convertit jamais les nombres tout seul.
        // On calcule en f64 PUIS on convertit : `as` passe avant `*`,
        // `x * multiplicateur as i32` tronquerait 1.5 en 1.
        let e = etage as f64;
        let vie = ((VIE_BASE + e * VIE_PAR_ETAGE) * multiplicateur) as i32;
        let degats = ((DEGATS_BASE + e * DEGATS_PAR_ETAGE) * multiplicateur) as i32;
        let armure = (e * ARMURE_PAR_ETAGE * multiplicateur) as i32;

        Enemie {
            enemy_type,
            enemy_rang,

            nom,
            niveau: etage,

            vie,
            // `vie_max` sert à la barre de vie de l'UI.
            vie_max: vie,
            degats,
            tx_critique,
            dgt_critique,
            armure,
            // Décidé par `preparer_attaque`, appelée par le jeu quand le combat commence.
            prochain_critique: false,
        }
    }

    // Le critique est tiré UN TOUR À L'AVANCE et annoncé au joueur, pour qu'il puisse bloquer.
    // Appelée au début du combat et au début de chaque nouveau tour.
    pub fn preparer_attaque(&mut self, journal: &mut Vec<String>) {
        self.prochain_critique = rand::random::<f64>() <= self.tx_critique;
        if self.prochain_critique {
            journal.push(format!("{} prépare une attaque puissante !", self.nom));
        }
    }

    pub fn attaquer(&self, cible: &mut Joueur, journal: &mut Vec<String>) {
        let mut degats = self.degats;
        // Plus de tirage ici : on utilise ce qui a été annoncé au tour précédent.
        if self.prochain_critique {
            degats += self.dgt_critique;
            journal.push(format!("{} déchaîne son attaque puissante !", self.nom));
        }

        // C'est la cible qui applique SON armure : l'attaquant ne la connaît pas.
        let subis = cible.subir_degats(degats);
        // Les PV restants : avec la ligne du soin, on suit le tour en entier.
        journal.push(format!(
            "{} vous inflige {} dégâts ({}/{} PV).",
            self.nom,
            subis,
            cible.vie,
            cible.vie_max_total()
        ));
    }

    // Renvoie les dégâts réellement subis (après armure), pour le journal.
    pub fn subir_degats(&mut self, degats: i32) -> i32 {
        let subis = combat::reduire_degats(degats, self.armure);
        // `max(0)` empêche la vie de passer en négatif.
        self.vie = (self.vie - subis).max(0);
        subis
    }
}

pub enum EnemyType {
    Goblin,
    Minotaure,
    Troll,
    Loup,
    Oursain,
    Pirhana,
    Esprit,
    Golem,
    Gargouille,
    ChauveSouris,
}
impl EnemyType {
    // 10 types sur 100 étages : un nouveau type tous les 10 étages.
    fn from_floor(etage: u32) -> Self {
        // `saturating_sub` évite un panic si etage vaut 0 (0 - 1 sur un u32 = débordement).
        match etage.saturating_sub(1) / 10 {
            0 => Self::Goblin,
            1 => Self::Minotaure,
            2 => Self::Troll,
            3 => Self::Loup,
            4 => Self::Oursain,
            5 => Self::Pirhana,
            6 => Self::Esprit,
            7 => Self::Golem,
            8 => Self::Gargouille,
            _ => Self::ChauveSouris,
        }
    }

    // `&'static str` : le texte est écrit dans le programme, il vit pendant toute l'exécution.
    fn nom(&self) -> &'static str {
        match self {
            Self::Goblin => "Goblin",
            Self::Minotaure => "Minotaure",
            Self::Troll => "Troll",
            Self::Loup => "Loup",
            Self::Oursain => "Oursain",
            Self::Pirhana => "Pirhana",
            Self::Esprit => "Esprit",
            Self::Golem => "Golem",
            Self::Gargouille => "Gargouille",
            Self::ChauveSouris => "Chauve-Souris",
        }
    }
}

pub enum EnemyRang {
    Normal,
    Elite,
    Boss,
}
impl EnemyRang {
    fn pick_rang(etage: u32) -> Self {
        if etage.is_multiple_of(10) {
            return Self::Boss;
        }

        let random_value: f64 = rand::random();
        if random_value < 0.9 {
            Self::Normal
        } else {
            Self::Elite
        }
    }

    pub fn nom(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Elite => "Elite",
            Self::Boss => "Boss",
        }
    }
}
