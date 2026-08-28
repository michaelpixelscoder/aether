use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

use bevy::prelude::*;
use libloading::{Library, Symbol};

struct Game {
    name: &'static str,
    binary: &'static str,
    library: &'static str,
    description: &'static str,
}

const GAMES: &[Game] = &[
    Game {
        name: "Skyway Runner",
        binary: "runner_exec",
        library: "runner_game",
        description: "Third-person lane running and jumping",
    },
    Game {
        name: "Aether Shipwright",
        binary: "shipwright_exec",
        library: "shipwright_game",
        description: "Build a skyship voxel by voxel",
    },
    Game {
        name: "sandbox",
        binary: "sandbox_exec",
        library: "sandbox_game",
        description: "First-person movement on the grid ground",
    },
];

type RegisterGameFn = unsafe extern "C" fn(*mut App);

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
    if try_launch_dynamic(game)? {
        return Ok(());
    }

    let mut path = std::env::current_exe()?;
    path.pop();
    path.push(executable_name(game.binary));
    println!("Launching {}…", game.name);
    Command::new(path).spawn()?.wait()?;
    Ok(())
}

fn try_launch_dynamic(game: &Game) -> io::Result<bool> {
    let mut path = std::env::current_exe()?;
    path.pop();
    path.push(dynamic_library_name(game.library));
    if !path.exists() {
        return Ok(false);
    }

    println!("Loading {} from {}…", game.name, path.display());
    let result = unsafe {
        let library = Library::new(&path).map_err(io::Error::other)?;
        let register: Symbol<'_, RegisterGameFn> = library
            .get(b"aether_register_game")
            .map_err(io::Error::other)?;
        let mut app = App::new();
        register(&mut app as *mut App);
        app.run();
        Ok::<(), io::Error>(())
    };
    result?;
    Ok(true)
}

fn executable_name(name: &str) -> PathBuf {
    let mut path = PathBuf::from(name);
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

fn dynamic_library_name(name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.dll", name)
    } else if cfg!(target_os = "macos") {
        format!("lib{}.dylib", name)
    } else {
        format!("lib{}.so", name)
    }
}
