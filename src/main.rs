// main.rs

pub mod actions;
mod quest;
mod shell;
mod state;
mod ui;
mod world;

use clap::Parser;
use directories::ProjectDirs;
use include_dir::{Dir, include_dir};
use log::{LevelFilter, debug, error, info, warn};
use quest::{Library, ValidationResult};
use simplelog::{Config, SimpleLogger, WriteLogger};
use state::GameState;
use std::fs::OpenOptions;
use world::WorldEngine;

// Embed library files into binary for portability
const VERSION: &str = env!("CARGO_PKG_VERSION");
static DEFAULT_LIBRARY: Dir = include_dir!("$CARGO_MANIFEST_DIR/library");

#[derive(Parser)]
#[command(name = "Supershell")]
struct Cli {
    #[arg(short, long)]
    check: Option<String>,
    #[arg(long)]
    reset: bool,
    #[arg(long)]
    status: bool,
    #[arg(long)]
    debug: bool,
}

struct AppContext {
    library_path: std::path::PathBuf,
    save_path: std::path::PathBuf,
    log_path: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();

    // 1. Setup Data Directories
    let proj_dirs = ProjectDirs::from("com", "jalexlong", "supershell").unwrap();
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir).unwrap();

    // 2. Logging Setup
    let log_path = data_dir.join("supershell.log");

    // Determine log strictness: Debug shows internals, Info shows flow.
    let log_level = if args.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    // Initialize File Logger
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path);

    if let Ok(target) = file {
        let _ = WriteLogger::init(log_level, Config::default(), target);
    } else {
        let _ = SimpleLogger::init(log_level, Config::default());
        error!(
            "Failed to create log file at {:?}. Falling back to stdout.",
            log_path
        );
    }

    debug!(">> DEBUG MODE: ENABLED. Log file: {:?}", log_path);
    info!(
        ">> SYSTEM STARTUP. Version: {}. Session Log: {:?}",
        VERSION, log_path
    );

    // 3. Extract Embedded Library
    let lib_path = data_dir.join("library");
    // Force update in debug mode for development velocity
    if args.debug && lib_path.exists() {
        debug!("Dev Mode: Cleaning library cache at {:?}", lib_path);
        std::fs::remove_dir_all(&lib_path).unwrap_or_default();
    }

    if !lib_path.exists() {
        std::fs::create_dir_all(&lib_path).unwrap();
        DEFAULT_LIBRARY.extract(&lib_path).unwrap();
        info!("Library extracted to {:?}", lib_path);
    }

    let ctx = AppContext {
        library_path: lib_path,
        save_path: data_dir.join("save.json"),
        log_path,
    };

    let world = WorldEngine::new();
    world.initialize();

    // 4. Mode Selection
    if args.reset {
        warn!(">> SYSTEM RESET INITIATED BY USER.");
        if ctx.save_path.exists() {
            std::fs::remove_file(&ctx.save_path).unwrap();
        }
        println!(">> System Reset.");
        return;
    }

    if let Some(cmd) = args.check {
        // Run verification loop
        debug!("Logic Check Initiated. Input: '{}'", cmd);
        handle_check_command(&cmd, &ctx, &world);
    } else if args.status {
        // Just show HUD
        debug!("Status Check Initiated.");
        handle_check_command("", &ctx, &world);
    } else {
        // Default: Start the Game Shell
        info!("Spawning Shell Environment...");
        shell::start_shell();
    }
}

/// The Main Logic Loop
fn handle_check_command(cmd: &str, app: &AppContext, world: &WorldEngine) {
    // --- Load State ---
    let mut game = GameState::load(app.save_path.to_str().unwrap());
    let library = Library::new(app.library_path.clone());

    // Auto-select first course if playing a new game
    if game.current_course.is_empty() {
        let courses = library.list_available_courses();
        if let Some((path, _)) = courses.first() {
            let id = path.file_stem().unwrap().to_string_lossy().to_string();
            info!("New Game Detected. Auto-loading course: {}", id);
            game.current_course = id;
        }
    }

    let course = library
        .get_course(&game.current_course)
        .expect("Course data corrupt");

    // Safety check for bounds
    if game.current_chapter_index >= course.chapters.len() {
        info!("Campaign Complete. No further checks required.");
        return;
    }

    let chapter = &course.chapters[game.current_chapter_index];
    let task = &chapter.tasks[game.current_task_index];

    // --- Check Conditions ---
    let mut is_task_complete = false;

    for condition in &task.conditions {
        if let ValidationResult::Valid = condition.check(cmd, &game) {
            is_task_complete = true;
            debug!("Condition Met: Task '{}' validated.", task.objective);
            break;
        }
    }

    // --- Render UI ---
    ui::render_mission_hud(
        &chapter.title,
        task,
        game.current_task_index + 1,
        chapter.tasks.len(),
    );

    // --- Update State on Success ---
    if is_task_complete {
        info!("Task Completion Verified: [{}]", task.objective);
        println!("\n>> \x1b[1;32mSUCCESS: {}\x1b[0m", task.success_msg);

        game.advance_task();

        // Check if Chapter is Done
        if game.current_task_index >= chapter.tasks.len() {
            ui::play_cutscene(&chapter.outro);
            game.advance_chapter();

            // Run Setup for Next Chapter
            if game.current_chapter_index < course.chapters.len() {
                let next_chapter = &course.chapters[game.current_chapter_index];
                if !next_chapter.setup_actions.is_empty() {
                    info!(
                        "Executing World Setup for Chapter {}",
                        game.current_chapter_index + 1
                    );
                    println!(">> [SYSTEM] Reconfiguring Construct...");
                    world.build_scenario(&next_chapter.setup_actions);
                }
            } else {
                info!("Course Completion Verified.");
                println!("\n\x1b[1;32m>> [SYSTEM] ALL MODULES COMPLETE.\x1b[0m");
                game.is_finished = true;
            }
        } else {
            // Pause so user sees success message
            println!("\n\x1b[0;90m[ PRESS ENTER TO CONTINUE ]\x1b[0m");
            let mut s = String::new();
            std::io::stdin().read_line(&mut s).unwrap();
        }

        game.save(app.save_path.to_str().unwrap());
    } else if !cmd.is_empty() {
        debug!("Validation Failed for input: '{}'", cmd);
    }
}
