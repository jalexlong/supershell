// main.rs

pub mod actions;
mod content;
mod shell;
mod state;
mod ui;
mod world;

use clap::Parser;
use content::{Library, ValidationResult};
use directories::ProjectDirs;
use include_dir::{Dir, include_dir};
use log::{LevelFilter, debug, error, info, warn};
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

    // Auto-select first module if playing a new game
    if game.current_module.is_empty() {
        let modules = library.list_modules();
        if let Some((path, _)) = modules.first() {
            let id = path.file_stem().unwrap().to_string_lossy().to_string();
            info!("New Game Detected. Auto-loading module: {}", id);
            game.current_module = id;
        }
    }

    let module = library
        .get_module(&game.current_module)
        .expect("module data corrupt");

    // Safety check for bounds
    if game.current_mission_index >= module.missions.len() {
        info!("Module Complete.");
        return;
    }

    let mission = &module.missions[game.current_mission_index];
    let objective = &mission.objectives[game.current_objective_index];

    // --- Check Conditions ---
    let mut is_complete = false;
    for condition in &objective.conditions {
        if let ValidationResult::Valid = condition.check(cmd, &game) {
            is_complete = true;
            debug!("Condition Met: Objective '{}' validated.", objective.title);
            break;
        }
    }

    // --- Render UI ---
    ui::render_mission_hud(
        &mission.title,
        objective,
        game.current_objective_index + 1,
        mission.objectives.len(),
    );

    // --- Update State on Success ---
    if is_complete {
        info!("objective Completion Verified: [{}]", objective.title);
        println!("\n>> \x1b[1;32mSUCCESS: {}\x1b[0m", objective.success_msg);

        game.advance_objective();

        // Check if mission is Done
        if game.current_objective_index >= mission.objectives.len() {
            info!("Mission Complete: [{}]", mission.title);
            ui::play_cutscene(&mission.outro);
            game.advance_mission();

            // Run Setup for Next mission
            if game.current_mission_index < module.missions.len() {
                let next_mission = &module.missions[game.current_mission_index];
                if !next_mission.setup_actions.is_empty() {
                    info!(
                        "Executing World Setup for Mission {}",
                        game.current_mission_index + 1
                    );
                    // println!(">> [SYSTEM] Reconfiguring Construct...");
                    world.build_scenario(&next_mission.setup_actions);
                }
            } else {
                info!("Module Completion Verified.");
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
