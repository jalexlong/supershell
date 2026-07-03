pub mod actions;
mod app;
mod construct;
mod engine;
mod paths;
mod quest;
mod shell;
mod state;
mod ui;
mod world;

use anyhow::{Context, Result};
use clap::Parser;
use include_dir::{Dir, include_dir};
use paths::build_app_context;
use quest::{Course, Library};
use state::GameState;
use std::path::PathBuf;
use world::WorldEngine;

// --- CONSTANTS & EMBEDDED ASSETS ---

const VERSION: &str = env!("CARGO_PKG_VERSION");
static DEFAULT_LIBRARY: Dir = include_dir!("$CARGO_MANIFEST_DIR/library");

// --- CLI DEFINITION ---

#[derive(Parser)]
#[command(name = "Supershell")]
#[command(version = VERSION)]
#[command(about = "A terminal-based learning RPG", long_about = None)]
struct Cli {
    #[arg(short, long)]
    check: Option<String>,
    #[arg(long)]
    cwd: Option<PathBuf>,
    #[arg(long)]
    command_status: Option<i32>,
    #[arg(long)]
    reset: bool,
    #[arg(long)]
    validate: Option<String>,
    #[arg(long)]
    menu: bool,
    #[arg(long)]
    status: bool,
    #[arg(long)]
    refresh: bool,
}

// --- MAIN ENTRY POINT ---

fn main() {
    if let Err(err) = run() {
        eprintln!(">> [ERROR] {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Cli::parse();

    let ctx = build_app_context();

    if let Some(path_str) = args.validate {
        app::perform_validation(&path_str);
        return Ok(());
    }

    std::fs::create_dir_all(&ctx.library_path).context("Failed to create library dir")?;
    DEFAULT_LIBRARY
        .extract(&ctx.library_path)
        .context("Failed to extract default library")?;

    let mut game = if args.reset {
        app::reset_game(&ctx.save_path)
    } else {
        GameState::load(ctx.save_path.to_str().unwrap())
    };

    let lib = Library::new(ctx.library_path.clone());
    let mut active_course_path = app::resolve_course_path(&game, &lib);

    if args.menu {
        active_course_path = ui::show_module_menu(lib.list_available_courses());
        if let Some(ref path) = active_course_path {
            game.current_course = path.file_name().unwrap().to_string_lossy().to_string();
            game.current_quest_id = String::new();
            game.current_chapter_index = 0;
            game.current_task_index = 0;
            game.is_finished = false;
            app::save_game_state(&game, &ctx.save_path);

            if std::env::var("CONSTRUCT_UPLINK").is_ok() {
                println!("\n>> [SYSTEM] Module selection saved.");
                println!(
                    ">> [SYSTEM] Type 'exit' to leave the Construct, then run 'supershell' to play your selection."
                );
            } else {
                println!("\n>> [SYSTEM] Module selected. Run 'supershell' to begin.");
            }
        }
        return Ok(());
    }

    let course_path = match active_course_path {
        Some(p) => p,
        None => {
            eprintln!(">> [ERROR] No module selected. Run 'supershell --menu'.");
            return Ok(());
        }
    };

    if !course_path.exists() {
        eprintln!(">> [ERROR] Module file missing: {:?}", course_path);
        return Ok(());
    }

    let course = Course::load(&course_path).context("Failed to load course")?;

    if game.course_version != course.version {
        game.course_version = course.version.clone();
        app::save_game_state(&game, &ctx.save_path);
    }

    let world = WorldEngine::new()?;
    world.initialize()?;

    if game.current_quest_id.is_empty() {
        if let Some(first_quest) = course.quests.first() {
            game.current_quest_id = first_quest.id.clone();
            app::save_game_state(&game, &ctx.save_path);
        }
    }

    let check_cwd = args.cwd.clone();
    let command_status = args.command_status;
    if let Some(cmd) = args.check {
        let outcome = app::handle_check_command(
            &cmd,
            check_cwd.as_deref(),
            command_status,
            &mut game,
            &course,
            &ctx.save_path,
            &world,
        );
        std::process::exit(outcome.exit_code());
    } else if args.status {
        app::handle_status_display(&game, &course);
    } else if args.refresh {
        app::handle_refresh_sequence(&game, &course);
    } else {
        shell::launch_infected_session()?;
    }

    Ok(())
}
