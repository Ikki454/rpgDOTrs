mod combat;
mod enemie;
mod game;
mod item;
mod joueur;
mod ui;

use std::io;

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};

use game::Game;

// `io::Result<()>` : main peut renvoyer une erreur, et `?` la fait remonter.
fn main() -> io::Result<()> {
    // `init` passe le terminal en mode "plein écran" ; `restore` le remet dans son état normal.
    let mut terminal = ratatui::init();
    let resultat = boucle_de_jeu(&mut terminal);
    // On restaure AVANT de renvoyer le résultat, même en cas d'erreur.
    ratatui::restore();
    resultat
}

// Boucle d'événements : dessiner, attendre une touche, mettre à jour le jeu, recommencer.
fn boucle_de_jeu(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut game = Game::default();

    while !game.quitter {
        // Closure `|frame| ...` : une fonction anonyme que ratatui appelle pour dessiner.
        terminal.draw(|frame| ui::dessiner(frame, &game))?;

        // `event::read` bloque jusqu'au prochain événement.
        // Sur Windows, chaque touche envoie "Press" PUIS "Release" : on ne garde que Press.
        if let Event::Key(touche) = event::read()?
            && touche.kind == KeyEventKind::Press
        {
            game.gerer_touche(touche.code);
        }
    }

    Ok(())
}
