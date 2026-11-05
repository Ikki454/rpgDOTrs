use std::ops::Add;

// `Rng` est un trait : il faut l'importer pour pouvoir appeler `gen_range` sur le générateur.
use rand::Rng;

// Où l'objet se porte. Le joueur a une seule place par emplacement.
#[derive(Clone, Copy)]
pub enum Emplacement {
    Arme,
    Casque,
    Torse,
    Mains,
    Pantalon,
    Bottes,
    Ceinture,
    Anneau,
}

impl Emplacement {
    pub const NOMBRE: usize = 8;

    // L'ordre de ce tableau = l'ordre d'affichage dans le panneau Équipement.
    pub const TOUS: [Emplacement; Emplacement::NOMBRE] = [
        Emplacement::Arme,
        Emplacement::Casque,
        Emplacement::Torse,
        Emplacement::Mains,
        Emplacement::Pantalon,
        Emplacement::Bottes,
        Emplacement::Ceinture,
        Emplacement::Anneau,
    ];

    // Un enum sans données se convertit en nombre avec `as` : Arme = 0, Casque = 1, etc.
    // Sert d'index dans le tableau d'équipement du joueur.
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn nom(&self) -> &'static str {
        match self {
            Emplacement::Arme => "Arme",
            Emplacement::Casque => "Casque",
            Emplacement::Torse => "Torse",
            Emplacement::Mains => "Mains",
            Emplacement::Pantalon => "Pantalon",
            Emplacement::Bottes => "Bottes",
            Emplacement::Ceinture => "Ceinture",
            Emplacement::Anneau => "Anneau",
        }
    }
}

#[derive(Clone, Copy)]
pub enum Rarete {
    Commun,
    PeuCommun,
    Rare,
    Epique,
    Legendaire,
    Mythique,
}

impl Rarete {
    // Plus on descend, plus les raretés hautes sont fréquentes.
    // Chances à l'étage 1 → étage 100 :
    //   Mythique 0.2 % → 3 %, Légendaire 1 % → 8 %, Épique 4 % → 20 %,
    //   Rare 12 % → 25 %, Peu commun 25 %, Commun : le reste (58 % → 19 %).
    fn aleatoire(etage: u32) -> Rarete {
        // `t` va de 0.0 (étage 1) à 1.0 (étage 100).
        let t = (etage.min(100) as f64) / 100.0;

        // Un tableau de (rareté, chance), de la plus rare à la plus courante.
        let chances = [
            (Rarete::Mythique, 0.002 + 0.03 * t),
            (Rarete::Legendaire, 0.01 + 0.07 * t),
            (Rarete::Epique, 0.04 + 0.16 * t),
            (Rarete::Rare, 0.12 + 0.13 * t),
            (Rarete::PeuCommun, 0.25),
        ];

        // On tire un nombre entre 0 et 1, puis on retire chaque chance tour à tour :
        // la première rareté qui fait passer le tirage sous sa chance est choisie.
        let mut tirage: f64 = rand::random();
        for (rarete, chance) in chances {
            if tirage < chance {
                return rarete;
            }
            tirage -= chance;
        }
        // Aucune chance n'a été atteinte : objet commun.
        Rarete::Commun
    }

    // Toutes les raretés, de la plus faible à la plus forte.
    // `cfg(test)` : seuls les tests s'en servent pour l'instant, le jeu n'en a pas besoin.
    #[cfg(test)]
    const TOUTES: [Rarete; 6] = [
        Rarete::Commun,
        Rarete::PeuCommun,
        Rarete::Rare,
        Rarete::Epique,
        Rarete::Legendaire,
        Rarete::Mythique,
    ];

    // Palier de la rareté : Commun = 1, Peu commun = 2, ... Mythique = 6.
    // `self as usize` donne la position dans l'enum (Commun = 0), comme pour `Emplacement::index`.
    fn palier(self) -> f64 {
        (self as usize + 1) as f64
    }

    pub fn nom(&self) -> &'static str {
        match self {
            Rarete::Commun => "Commun",
            Rarete::PeuCommun => "Peu Commun",
            Rarete::Rare => "Rare",
            Rarete::Epique => "Épique",
            Rarete::Legendaire => "Légendaire",
            Rarete::Mythique => "Mythique",
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
    pub armure: i32,
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
            armure: self.armure + autre.armure,
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
        if self.armure != 0 {
            morceaux.push(format!("Armure +{}", self.armure));
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
    // Tire un emplacement et une rareté au hasard, puis fabrique l'objet correspondant.
    pub fn aleatoire(etage: u32) -> Item {
        let index = rand::thread_rng().gen_range(0..Emplacement::NOMBRE);
        Item::creer(Emplacement::TOUS[index], Rarete::aleatoire(etage), etage)
    }

    pub fn creer(emplacement: Emplacement, rarete: Rarete, etage: u32) -> Item {
        let palier = rarete.palier();
        // `t` va de 0.0 (étage 1) à 1.0 (étage 100).
        let t = etage.min(100) as f64 / 100.0;

        // Dégâts, PV et armure : `base × (palier + t)`.
        // Chaque rareté a sa propre tranche, et l'étage fait avancer DANS la tranche :
        //   Commun    : base × 1.0 (étage 1) → base × 2.0 (étage 100)
        //   Peu commun: base × 2.0             → base × 3.0
        //   ...
        //   Mythique  : base × 6.0             → base × 7.0
        // Un Commun de l'étage 100 ne dépasse donc jamais un Peu commun de l'étage 1.
        let stat = |base: f64| (base * (palier + t)).round() as i32;
        // Critiques : dépendent SEULEMENT de la rareté, pas de l'étage.
        let tx_crit = |base: f64| base * palier;
        let dgt_crit = |base: f64| (base * palier).round() as i32;

        // `..Default::default()` : les champs non écrits prennent leur valeur par défaut (0).
        let bonus = match emplacement {
            // `base` = valeur d'un objet Commun à l'étage 1.
            Emplacement::Arme => Bonus {
                degats: stat(4.0),
                dgt_critique: dgt_crit(4.0),
                ..Default::default()
            },
            // Les pièces d'armure (casque, torse, pantalon, bottes) donnent de l'armure.
            // Le torse en donne le plus, les bottes le moins.
            Emplacement::Casque => Bonus {
                vie_max: stat(10.0),
                armure: stat(2.0),
                ..Default::default()
            },
            Emplacement::Torse => Bonus {
                vie_max: stat(18.0),
                armure: stat(4.0),
                ..Default::default()
            },
            Emplacement::Mains => Bonus {
                degats: stat(2.0),
                tx_critique: tx_crit(0.04),
                ..Default::default()
            },
            Emplacement::Pantalon => Bonus {
                vie_max: stat(12.0),
                armure: stat(3.0),
                ..Default::default()
            },
            Emplacement::Bottes => Bonus {
                vie_max: stat(6.0),
                degats: stat(1.0),
                armure: stat(2.0),
                ..Default::default()
            },
            Emplacement::Ceinture => Bonus {
                vie_max: stat(8.0),
                dgt_critique: dgt_crit(5.0),
                ..Default::default()
            },
            Emplacement::Anneau => Bonus {
                tx_critique: tx_crit(0.03),
                dgt_critique: dgt_crit(8.0),
                ..Default::default()
            },
        };

        Item {
            nom: Item::nom_pour(emplacement, rarete),
            emplacement,
            rarete,
            bonus,
        }
    }

    // `match` sur un TUPLE : un nom pour chaque combinaison emplacement × rareté.
    // Le compilateur vérifie qu'aucune des 48 combinaisons (8 × 6) n'est oubliée.
    fn nom_pour(emplacement: Emplacement, rarete: Rarete) -> &'static str {
        use Emplacement::*;
        use Rarete::*;

        match (emplacement, rarete) {
            (Arme, Commun) => "Épée rouillée",
            (Arme, PeuCommun) => "Épée en bois",
            (Arme, Rare) => "Lame d'acier",
            (Arme, Epique) => "Lame runique",
            (Arme, Legendaire) => "Lame légendaire",
            (Arme, Mythique) => "Lame mythique",

            (Casque, Commun) => "Casque en cuir",
            (Casque, PeuCommun) => "Casque en bois",
            (Casque, Rare) => "Heaume de fer",
            (Casque, Epique) => "Couronne du roi déchu",
            (Casque, Legendaire) => "Casque légendaire",
            (Casque, Mythique) => "Casque mythique",

            (Torse, Commun) => "Tunique",
            (Torse, PeuCommun) => "Tunique en cuir",
            (Torse, Rare) => "Cuirasse",
            (Torse, Epique) => "Plastron du titan",
            (Torse, Legendaire) => "Torse légendaire",
            (Torse, Mythique) => "Torse mythique",

            (Mains, Commun) => "Gants en cuir",
            (Mains, PeuCommun) => "Gants en tissu",
            (Mains, Rare) => "Gantelets de fer",
            (Mains, Epique) => "Poignes du voleur",
            (Mains, Legendaire) => "Gants légendaires",
            (Mains, Mythique) => "Gants mythiques",

            (Pantalon, Commun) => "Pantalon de toile",
            (Pantalon, PeuCommun) => "Pantalon en tissu",
            (Pantalon, Rare) => "Jambières de mailles",
            (Pantalon, Epique) => "Jambières draconiques",
            (Pantalon, Legendaire) => "Jambières légendaires",
            (Pantalon, Mythique) => "Jambières mythiques",

            (Bottes, Commun) => "Bottes usées",
            (Bottes, PeuCommun) => "Bottes en tissu",
            (Bottes, Rare) => "Bottes renforcées",
            (Bottes, Epique) => "Bottes des vents",
            (Bottes, Legendaire) => "Bottes légendaires",
            (Bottes, Mythique) => "Bottes mythiques",

            (Ceinture, Commun) => "Ceinture de corde",
            (Ceinture, PeuCommun) => "Ceinture en tissu",
            (Ceinture, Rare) => "Ceinture cloutée",
            (Ceinture, Epique) => "Ceinture du géant",
            (Ceinture, Legendaire) => "Ceinture légendaire",
            (Ceinture, Mythique) => "Ceinture mythique",

            (Anneau, Commun) => "Anneau de cuivre",
            (Anneau, PeuCommun) => "Anneau de bronze",
            (Anneau, Rare) => "Anneau d'argent",
            (Anneau, Epique) => "Anneau des ombres",
            (Anneau, Legendaire) => "Anneau légendaire",
            (Anneau, Mythique) => "Anneau mythique",
        }
    }

    // Exemple : "Lame d'acier | [Rare] | Dégâts +7, Dgt crit +6".
    pub fn description(&self) -> String {
        format!(
            "{} | [{}] | {}",
            self.nom,
            self.rarete.nom(),
            self.bonus.description()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_critiques_ne_dependent_pas_de_l_etage() {
        let bas = Item::creer(Emplacement::Anneau, Rarete::Rare, 1);
        let profond = Item::creer(Emplacement::Anneau, Rarete::Rare, 90);
        assert_eq!(bas.bonus.tx_critique, profond.bonus.tx_critique);
        assert_eq!(bas.bonus.dgt_critique, profond.bonus.dgt_critique);
    }

    // Pour chaque emplacement et chaque paire de raretés qui se suivent :
    // le pire objet de la rareté au-dessus (étage 1) vaut au moins le meilleur d'en dessous (étage 100).
    #[test]
    fn les_raretes_ne_se_chevauchent_pas() {
        // Toutes les stats d'un bonus dans un tableau, pour les comparer d'un coup.
        let stats = |b: Bonus| {
            [
                b.degats as f64,
                b.vie_max as f64,
                b.tx_critique,
                b.dgt_critique as f64,
                b.armure as f64,
            ]
        };

        for emplacement in Emplacement::TOUS {
            // `windows(2)` : parcourt les paires voisines (Commun/PeuCommun, PeuCommun/Rare...).
            for paire in Rarete::TOUTES.windows(2) {
                let meilleur_bas = stats(Item::creer(emplacement, paire[0], 100).bonus);
                let pire_haut = stats(Item::creer(emplacement, paire[1], 1).bonus);
                for (bas, haut) in meilleur_bas.iter().zip(pire_haut) {
                    assert!(
                        *bas <= haut,
                        "{} : {} dépasse {}",
                        emplacement.nom(),
                        paire[0].nom(),
                        paire[1].nom()
                    );
                }
            }
        }
    }

    #[test]
    fn les_pv_dependent_de_l_etage() {
        let bas = Item::creer(Emplacement::Torse, Rarete::Commun, 1);
        let profond = Item::creer(Emplacement::Torse, Rarete::Commun, 90);
        assert!(profond.bonus.vie_max > bas.bonus.vie_max);
    }

    // Test statistique : sur 20 000 tirages, l'écart dû au hasard est négligeable.
    #[test]
    fn les_raretes_hautes_sont_plus_frequentes_en_profondeur() {
        // `filter` + `count` : combien de tirages donnent une rareté Mythique.
        let mythiques = |etage| {
            (0..20_000)
                .filter(|_| matches!(Rarete::aleatoire(etage), Rarete::Mythique))
                .count()
        };
        // Environ 40 attendus à l'étage 1, environ 640 à l'étage 100.
        assert!(mythiques(100) > mythiques(1) * 3);
    }
}
