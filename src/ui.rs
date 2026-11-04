use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Gauge, List, ListItem, ListState, Paragraph},
};

use crate::combat::Action;
use crate::enemie::{Enemie, EnemyRang};
use crate::game::{CHOIX_BUTIN, DERNIER_ETAGE, Ecran, Game};
use crate::item::{Emplacement, Item, Rarete};
use crate::joueur::{Amelioration, Joueur};

// L'UI ne modifie jamais le jeu : elle reçoit `&Game` (lecture seule) et dessine.
// ratatui redessine tout l'écran à chaque appel ("immediate mode").
pub fn dessiner(frame: &mut Frame, game: &Game) {
    // `if let` : on teste UNE variante sans écrire un `match` complet.
    if let Ecran::SaisieNom { nom } = &game.ecran {
        dessiner_saisie_nom(frame, nom);
        return;
    }

    // Découpage de l'écran : `areas` renvoie un tableau qu'on déstructure directement.
    let [haut, arene, zone_equipement, bas] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Min(8),
    ])
    .areas(frame.area());

    dessiner_entete(frame, haut, game);
    dessiner_equipement(frame, zone_equipement, &game.joueur);

    let [zone_joueur, zone_ennemi] =
        Layout::horizontal([Constraint::Percentage(50); 2]).areas(arene);
    dessiner_joueur(frame, zone_joueur, &game.joueur);
    match &game.ecran {
        Ecran::Combat { ennemi, .. } => dessiner_ennemi(frame, zone_ennemi, ennemi),
        // Motif imbriqué : on ne dessine le coffre que s'il contient encore un objet.
        Ecran::Butin {
            item: Some(item), ..
        } => dessiner_butin(frame, zone_ennemi, item, &game.joueur),
        _ => frame.render_widget(
            Paragraph::new("Aucun ennemi en vue.")
                .centered()
                .block(Block::bordered().title(" Ennemi ")),
            zone_ennemi,
        ),
    }

    let [zone_menu, zone_journal] =
        Layout::horizontal([Constraint::Length(34), Constraint::Min(20)]).areas(bas);
    dessiner_menu(frame, zone_menu, game);
    dessiner_journal(frame, zone_journal, &game.journal);
}

fn dessiner_saisie_nom(frame: &mut Frame, nom: &str) {
    let zone = centrer(frame.area(), 40, 7);
    let texte = vec![
        Line::from("Bienvenue dans le donjon !".bold()),
        Line::from(""),
        Line::from("Quel est ton nom ?"),
        Line::from(vec!["> ".yellow(), nom.into(), "_".slow_blink()]),
    ];
    let bloc = Block::bordered().title(" Nouvelle partie ");
    frame.render_widget(Paragraph::new(texte).centered().block(bloc), zone);
}

fn dessiner_entete(frame: &mut Frame, zone: Rect, game: &Game) {
    // `min` : après l'étage 100, on affiche quand même 100/100.
    let etage = game.etage.min(DERNIER_ETAGE);
    let texte = Line::from(vec![
        " Étage ".into(),
        Span::from(format!("{}/{}", etage, DERNIER_ETAGE)).bold(),
        "   ↑↓ choisir · Entrée valider · chiffre = raccourci · Échap quitter".dark_gray(),
    ]);
    let bloc = Block::bordered().title(" Donjon de Rust ".bold());
    frame.render_widget(Paragraph::new(texte).block(bloc), zone);
}

fn dessiner_joueur(frame: &mut Frame, zone: Rect, joueur: &Joueur) {
    let bloc = Block::bordered().title(" Héros ");
    // `inner` : la zone à l'intérieur des bordures, à calculer AVANT de donner `bloc` au rendu.
    let interieur = bloc.inner(zone);
    frame.render_widget(bloc, zone);

    let [ligne_nom, ligne_vie, ligne_stats] =
        Layout::vertical([Constraint::Length(1); 3]).areas(interieur);

    let nom = Line::from(vec![
        joueur.nom.as_str().bold(),
        format!("  niv {}", joueur.niveau).into(),
    ]);
    frame.render_widget(nom, ligne_nom);
    frame.render_widget(jauge_vie(joueur.vie, joueur.vie_max_total()), ligne_vie);

    // Stats totales : base + équipement.
    let stats = format!(
        "Dégâts {}  Crit {:.0}% (+{})",
        joueur.degats_total(),
        joueur.tx_critique_total() * 100.0,
        joueur.dgt_critique_total()
    );
    frame.render_widget(Line::from(stats).dark_gray(), ligne_stats);
}

// Les 8 emplacements, sur 2 colonnes de 4 lignes.
fn dessiner_equipement(frame: &mut Frame, zone: Rect, joueur: &Joueur) {
    let bloc = Block::bordered().title(" Équipement ");
    let interieur = bloc.inner(zone);
    frame.render_widget(bloc, zone);

    let [gauche, droite] = Layout::horizontal([Constraint::Percentage(50); 2]).areas(interieur);
    let lignes_gauche: [Rect; 4] = Layout::vertical([Constraint::Length(1); 4]).areas(gauche);
    let lignes_droite: [Rect; 4] = Layout::vertical([Constraint::Length(1); 4]).areas(droite);

    // `chain` met les deux colonnes bout à bout (8 zones),
    // `zip` les associe une par une aux 8 emplacements.
    for (ligne, emplacement) in lignes_gauche
        .into_iter()
        .chain(lignes_droite)
        .zip(Emplacement::TOUS)
    {
        let texte = ligne_item(emplacement.nom(), joueur.equipement(emplacement));
        frame.render_widget(texte, ligne);
    }
}

// "Arme    : Lame d'acier [Rare] Dégâts +7", ou "Arme    : -" si l'emplacement est vide.
fn ligne_item(etiquette: &str, item: Option<&Item>) -> Line<'static> {
    let debut = Span::from(format!("{:<8}: ", etiquette)).dark_gray();
    match item {
        Some(item) => Line::from(vec![
            debut,
            item.nom.bold(),
            " ".into(),
            texte_rarete(item.rarete),
            format!(" {}", item.bonus.description()).into(),
        ]),
        None => Line::from(vec![debut, "-".dark_gray()]),
    }
}

// Même idée que pour le rang des ennemis : la couleur est décidée par l'UI.
fn texte_rarete(rarete: Rarete) -> Span<'static> {
    let texte = format!("[{}]", rarete.nom());
    match rarete {
        Rarete::Commun => Span::from(texte),
        Rarete::Rare => texte.blue().bold(),
        Rarete::Epique => texte.magenta().bold(),
    }
}

fn dessiner_butin(frame: &mut Frame, zone: Rect, item: &Item, joueur: &Joueur) {
    let bloc = Block::bordered().title(" Coffre ".yellow().bold());
    let interieur = bloc.inner(zone);
    frame.render_widget(bloc, zone);

    let [ligne_nom, ligne_type, ligne_bonus, ligne_actuel] =
        Layout::vertical([Constraint::Length(1); 4]).areas(interieur);

    let nom = Line::from(vec![item.nom.bold(), " ".into(), texte_rarete(item.rarete)]);
    frame.render_widget(nom, ligne_nom);
    frame.render_widget(
        Line::from(format!("Emplacement : {}", item.emplacement.nom())).dark_gray(),
        ligne_type,
    );
    frame.render_widget(Line::from(item.bonus.description()).green(), ligne_bonus);

    // Comparaison avec l'objet déjà porté au même emplacement.
    let actuel = ligne_item("Actuel", joueur.equipement(item.emplacement));
    frame.render_widget(actuel, ligne_actuel);
}

fn dessiner_ennemi(frame: &mut Frame, zone: Rect, ennemi: &Enemie) {
    let bloc = Block::bordered().title(" Ennemi ");
    let interieur = bloc.inner(zone);
    frame.render_widget(bloc, zone);

    let [ligne_nom, ligne_vie, ligne_stats] =
        Layout::vertical([Constraint::Length(1); 3]).areas(interieur);

    // Le texte du rang vient d'enemie.rs, mais la couleur est un choix d'affichage : elle reste ici.
    let rang = ennemi.enemy_rang.nom();
    let rang = match ennemi.enemy_rang {
        EnemyRang::Normal => Span::from(rang),
        EnemyRang::Elite => rang.blue().bold(),
        EnemyRang::Boss => rang.red().bold(),
    };

    let nom = Line::from(vec![
        ennemi.nom.as_str().bold(),
        " ".into(),
        rang,
        format!("  niv {}", ennemi.niveau).into(),
    ]);
    frame.render_widget(nom, ligne_nom);
    frame.render_widget(jauge_vie(ennemi.vie, ennemi.vie_max), ligne_vie);

    let stats = format!(
        "Dégâts {}  Crit {:.0}% (+{})",
        ennemi.degats,
        ennemi.tx_critique * 100.0,
        ennemi.dgt_critique
    );
    frame.render_widget(Line::from(stats).dark_gray(), ligne_stats);
}

// Barre de vie dont la couleur change selon les PV restants.
fn jauge_vie(vie: i32, vie_max: i32) -> Gauge<'static> {
    // `clamp` : le ratio doit rester entre 0 et 1, sinon ratatui panique.
    let ratio = (vie as f64 / vie_max as f64).clamp(0.0, 1.0);
    let couleur = if ratio > 0.5 {
        Color::Green
    } else if ratio > 0.25 {
        Color::Yellow
    } else {
        Color::Red
    };
    Gauge::default()
        .gauge_style(Style::new().fg(couleur))
        .ratio(ratio)
        .label(format!("PV {}/{}", vie, vie_max))
}

fn dessiner_menu(frame: &mut Frame, zone: Rect, game: &Game) {
    // Un `match` qui renvoie un tuple : titre, lignes du menu et ligne sélectionnée.
    let (titre, lignes, selection): (&str, Vec<ListItem>, usize) = match &game.ecran {
        Ecran::Combat { selection, .. } => (
            " Actions ",
            Action::TOUTES
                .iter()
                .enumerate()
                .map(|(i, action)| {
                    ListItem::new(format!(
                        "{}. {}",
                        i + 1,
                        libelle_action(*action, &game.joueur)
                    ))
                })
                .collect(),
            *selection,
        ),
        Ecran::Amelioration { selection } => (
            " Amélioration ",
            Amelioration::TOUTES
                .iter()
                .enumerate()
                .map(|(i, amelioration)| {
                    ListItem::new(format!("{}. {}", i + 1, amelioration.description()))
                })
                .collect(),
            *selection,
        ),
        Ecran::Butin { selection, .. } => (
            " Coffre ",
            CHOIX_BUTIN
                .iter()
                .enumerate()
                .map(|(i, choix)| ListItem::new(format!("{}. {}", i + 1, choix)))
                .collect(),
            *selection,
        ),
        Ecran::Fin { victoire } => {
            dessiner_fin(frame, zone, *victoire);
            return;
        }
        Ecran::SaisieNom { .. } => return,
    };

    let liste = List::new(lignes)
        .block(Block::bordered().title(titre))
        .highlight_symbol("> ")
        .highlight_style(Style::new().yellow().bold());
    // `ListState` dit à ratatui quelle ligne surligner.
    let mut etat = ListState::default().with_selected(Some(selection));
    frame.render_stateful_widget(liste, zone, &mut etat);
}

fn libelle_action(action: Action, joueur: &Joueur) -> String {
    match action {
        Action::Attaquer => format!("Attaquer [{}/{}]", joueur.atk_left, joueur.atk_max),
        Action::Bloquer => format!("Bloquer  [{}/{}]", joueur.block_left, joueur.block_max),
        Action::Soigner => format!("Soigner  [{}/{}]", joueur.soign_left, joueur.soign_max),
        Action::Fuir => "Fuir".to_string(),
    }
}

fn dessiner_fin(frame: &mut Frame, zone: Rect, victoire: bool) {
    let titre = if victoire {
        "VICTOIRE !".green().bold()
    } else {
        "GAME OVER".red().bold()
    };
    let texte = vec![
        Line::from(titre),
        Line::from(""),
        Line::from("[R] Rejouer"),
        Line::from("[Q] Quitter"),
    ];
    frame.render_widget(
        Paragraph::new(texte).centered().block(Block::bordered()),
        zone,
    );
}

fn dessiner_journal(frame: &mut Frame, zone: Rect, journal: &[String]) {
    // On n'affiche que les dernières lignes qui tiennent dans le cadre (moins les 2 bordures).
    let hauteur = zone.height.saturating_sub(2) as usize;
    let debut = journal.len().saturating_sub(hauteur);
    let lignes: Vec<Line> = journal[debut..]
        .iter()
        .map(|message| Line::from(message.as_str()))
        .collect();

    frame.render_widget(
        Paragraph::new(lignes).block(Block::bordered().title(" Journal ")),
        zone,
    );
}

// Renvoie un rectangle de `largeur` x `hauteur` centré dans `zone`.
fn centrer(zone: Rect, largeur: u16, hauteur: u16) -> Rect {
    let [_, milieu, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(hauteur),
        Constraint::Fill(1),
    ])
    .areas(zone);
    let [_, centre, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(largeur),
        Constraint::Fill(1),
    ])
    .areas(milieu);
    centre
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyCode;
    use ratatui::{Terminal, backend::TestBackend};

    // `TestBackend` : un faux terminal en mémoire, pour vérifier ce que l'UI dessine.
    #[test]
    fn dessine_un_combat() {
        let mut game = Game::default();
        for c in "Frank".chars() {
            game.gerer_touche(KeyCode::Char(c));
        }
        game.gerer_touche(KeyCode::Enter);

        let mut terminal = Terminal::new(TestBackend::new(90, 22)).unwrap();
        terminal.draw(|frame| dessiner(frame, &game)).unwrap();

        let ecran = terminal.backend().to_string();
        println!("{ecran}");
        assert!(ecran.contains("Frank"));
        assert!(ecran.contains("Goblin"));
        assert!(ecran.contains("Attaquer [10/10]"));
    }

    #[test]
    fn dessine_un_coffre() {
        let mut game = Game::default();
        game.gerer_touche(KeyCode::Char('F'));
        game.gerer_touche(KeyCode::Enter);
        // Fuir 4 fois mène à l'étage 5, la salle des coffres.
        for _ in 0..4 {
            game.gerer_touche(KeyCode::Char('4'));
        }

        let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
        terminal.draw(|frame| dessiner(frame, &game)).unwrap();

        let ecran = terminal.backend().to_string();
        println!("{ecran}");
        assert!(ecran.contains("Coffre"));
        assert!(ecran.contains("Équiper"));
        assert!(ecran.contains("Actuel"));
    }
}
