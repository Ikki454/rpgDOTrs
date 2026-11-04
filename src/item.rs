use std::ops::Add;

// `Rng` est un trait : il faut l'importer pour pouvoir appeler `gen_range` sur le générateur.
use rand::Rng;

// Où l'objet se porte. Le joueur a une seule place par emplacement.
#[derive(Clone, Copy)]
pub enum Emplacement {
    Arme,
    Armure,
    Bijou,
}

impl Emplacement {
    pub fn nom(&self) -> &'static str {
        match self {
            Emplacement::Arme => "Arme",
            Emplacement::Armure => "Armure",
            Emplacement::Bijou => "Bijou",
        }
    }
}

#[derive(Clone, Copy)]
pub enum Rarete {
    Commun,
    Rare,
    Epique,
}

impl Rarete {
    // Plus on descend, plus les objets épiques sont fréquents (plafonné à 35 %).
    fn aleatoire(etage: u32) -> Rarete {
        let tirage: f64 = rand::random();
        let chance_epique = (0.05 + etage as f64 * 0.003).min(0.35);
        if tirage < chance_epique {
            Rarete::Epique
        } else if tirage < chance_epique + 0.30 {
            Rarete::Rare
        } else {
            Rarete::Commun
        }
    }

    fn multiplicateur(&self) -> f64 {
        match self {
            Rarete::Commun => 1.0,
            Rarete::Rare => 1.5,
            Rarete::Epique => 2.5,
        }
    }

    pub fn nom(&self) -> &'static str {
        match self {
            Rarete::Commun => "Commun",
            Rarete::Rare => "Rare",
            Rarete::Epique => "Épique",
        }
    }
}

// Les statistiques qu'un objet ajoute au joueur.
// `Default` : tous les champs à 0, pratique pour ne remplir que ceux qui comptent.
#[derive(Clone, Copy, Default)]
pub struct Bonus {
    pub degats: i32,
    pub vie_max: i32,
    pub tx_critique: f64,
    pub dgt_critique: i32,
}

// Surcharge de l'opérateur `+` : on peut écrire `bonus_a + bonus_b`.
impl Add for Bonus {
    type Output = Bonus;

    fn add(self, autre: Bonus) -> Bonus {
        Bonus {
            degats: self.degats + autre.degats,
            vie_max: self.vie_max + autre.vie_max,
            tx_critique: self.tx_critique + autre.tx_critique,
            dgt_critique: self.dgt_critique + autre.dgt_critique,
        }
    }
}

impl Bonus {
    // Exemple : "Dégâts +7, Crit +5%". On n'affiche que les stats non nulles.
    pub fn description(&self) -> String {
        let mut morceaux = Vec::new();
        if self.degats != 0 {
            morceaux.push(format!("Dégâts +{}", self.degats));
        }
        if self.vie_max != 0 {
            morceaux.push(format!("PV max +{}", self.vie_max));
        }
        if self.tx_critique != 0.0 {
            morceaux.push(format!("Crit +{:.0}%", self.tx_critique * 100.0));
        }
        if self.dgt_critique != 0 {
            morceaux.push(format!("Dgt crit +{}", self.dgt_critique));
        }
        morceaux.join(", ")
    }
}

// Pas de `Copy` ici : un objet est unique, il est DÉPLACÉ du coffre vers le joueur.
pub struct Item {
    pub nom: &'static str,
    pub emplacement: Emplacement,
    pub rarete: Rarete,
    pub bonus: Bonus,
}

impl Item {
    // Crée un objet au hasard, plus puissant selon l'étage et la rareté.
    pub fn aleatoire(etage: u32) -> Item {
        let rarete = Rarete::aleatoire(etage);
        let m = rarete.multiplicateur();
        let e = etage as f64;
        // Petite fonction locale : calcule une stat entière à partir d'une base et d'un gain par étage.
        let stat = |base: f64, par_etage: f64| ((base + e * par_etage) * m).round() as i32;

        // `..Default::default()` : les champs non écrits prennent leur valeur par défaut (0).
        let (nom, emplacement, bonus) = match rand::thread_rng().gen_range(0..7) {
            0 => (
                "Épée",
                Emplacement::Arme,
                Bonus {
                    degats: stat(3.0, 0.5),
                    ..Default::default()
                },
            ),
            1 => (
                "Hache",
                Emplacement::Arme,
                Bonus {
                    degats: stat(2.0, 0.3),
                    dgt_critique: stat(5.0, 0.5),
                    ..Default::default()
                },
            ),
            2 => (
                "Dague",
                Emplacement::Arme,
                Bonus {
                    degats: stat(1.0, 0.2),
                    tx_critique: 0.05 * m,
                    ..Default::default()
                },
            ),
            3 => (
                "Armure",
                Emplacement::Armure,
                Bonus {
                    vie_max: stat(15.0, 2.0),
                    ..Default::default()
                },
            ),
            4 => (
                "Bouclier",
                Emplacement::Armure,
                Bonus {
                    vie_max: stat(8.0, 1.0),
                    degats: stat(1.0, 0.2),
                    ..Default::default()
                },
            ),
            5 => (
                "Anneau",
                Emplacement::Bijou,
                Bonus {
                    tx_critique: 0.03 * m,
                    dgt_critique: stat(3.0, 0.3),
                    ..Default::default()
                },
            ),
            _ => (
                "Amulette",
                Emplacement::Bijou,
                Bonus {
                    vie_max: stat(5.0, 1.0),
                    degats: stat(1.0, 0.2),
                    ..Default::default()
                },
            ),
        };

        Item {
            nom,
            emplacement,
            rarete,
            bonus,
        }
    }

    // Exemple : "Épée [Rare] Dégâts +7".
    pub fn description(&self) -> String {
        format!(
            "{} [{}] {}",
            self.nom,
            self.rarete.nom(),
            self.bonus.description()
        )
    }
}
