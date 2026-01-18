use std::collections::HashSet;

use katha::config::ClaudePaths;
use katha::data::HistoryReader;
use katha::tui::App;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "--count-sessions" {
        count_sessions();
    } else {
        run_tui();
    }
}

fn run_tui() {
    match App::new() {
        Ok(mut app) => {
            if let Err(e) = app.load_sessions() {
                eprintln!("Error loading sessions: {}", e);
                std::process::exit(1);
            }

            if let Err(e) = app.run() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error initializing TUI: {}", e);
            std::process::exit(1);
        }
    }
}

fn count_sessions() {
    let paths = match ClaudePaths::new() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if !paths.history_exists() {
        eprintln!("Error: history.jsonl not found");
        std::process::exit(1);
    }

    match HistoryReader::read_all(&paths.history_file) {
        Ok(entries) => {
            let session_count = entries
                .iter()
                .map(|e| &e.session_id)
                .collect::<HashSet<_>>()
                .len();

            println!("Total entries: {}", entries.len());
            println!("Unique sessions: {}", session_count);

            let projects = HistoryReader::group_by_project(&paths.history_file).unwrap();
            println!("\nProjects: {}", projects.len());

            for (project, entries) in projects.iter().take(5) {
                let name = project.rsplit('/').next().unwrap_or(project);
                println!("  {} ({} entries)", name, entries.len());
            }
            if projects.len() > 5 {
                println!("  ... and {} more", projects.len() - 5);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
