use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

struct Game {
    name: &'static str,
    binary: &'static str,
    description: &'static str,
}

const GAMES: &[Game] = &[Game {
    name: "sandbox",
    binary: "sandbox",
    description: "First-person movement on the grid ground",
}];

fn main() -> io::Result<()> {
    println!("Aether Isles — Game Lobby\n");
    for (index, game) in GAMES.iter().enumerate() {
        println!("  {}. {:<16} {}", index + 1, game.name, game.description);
    }
    println!("  q. Quit");

    loop {
        print!("\nSelect a game: ");
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        let answer = answer.trim();
        if answer.eq_ignore_ascii_case("q") {
            return Ok(());
        }
        let Some(game) = answer
            .parse::<usize>()
            .ok()
            .and_then(|number| GAMES.get(number.saturating_sub(1)))
        else {
            println!("Please enter a listed number or q.");
            continue;
        };
        launch(game)?;
    }
}

fn launch(game: &Game) -> io::Result<()> {
    let mut path = std::env::current_exe()?;
    path.pop();
    path.push(executable_name(game.binary));
    println!("Launching {}…", game.name);
    Command::new(path).spawn()?.wait()?;
    Ok(())
}

fn executable_name(name: &str) -> PathBuf {
    let mut path = PathBuf::from(name);
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}
