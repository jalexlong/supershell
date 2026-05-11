pub mod actions;
mod engine;
mod paths;
mod quest;
mod shell;
mod state;
mod ui;
mod world; // <--- The new module

use anyhow::{Context, Result};
use clap::Parser;
use engine::{
    Progression, advance_progress, apply_rewards, is_command_relevant, validate_task_logic,
};
use include_dir::{Dir, include_dir};
use paths::build_app_context;
use quest::{Course, Library};
use state::GameState;
use std::path::Path;
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

    // 1. SETUP PATHS
    let ctx = build_app_context();

    // 2. SYSTEM OPERATIONS
    if let Some(path_str) = args.validate {
        perform_validation(&path_str);
        return Ok(());
    }

    // ALWAYS force extraction to ensure the library is up to date with your source code.
    // We create the directory if it's missing, then overwrite the files.
    std::fs::create_dir_all(&ctx.library_path).context("Failed to create library dir")?;

    DEFAULT_LIBRARY
        .extract(&ctx.library_path)
        .context("Failed to extract default library")?;

    // 3. LOAD GAME STATE
    let mut game = if args.reset {
        reset_game(&ctx.save_path)
    } else {
        GameState::load(ctx.save_path.to_str().unwrap())
    };

    // 4. RESOLVE COURSE
    let lib = Library::new(ctx.library_path.clone());
    let mut active_course_path = resolve_course_path(&game, &lib);

    if args.menu {
        active_course_path = show_menu(&lib);
        if let Some(ref path) = active_course_path {
            game.current_course = path.file_name().unwrap().to_string_lossy().to_string();
            game.current_quest_id = String::new();
            game.current_chapter_index = 0;
            game.current_task_index = 0;
            game.is_finished = false;
            game.save(ctx.save_path.to_str().unwrap())
                .expect("Failed to save game state");
        }
    }

    // 5. VALIDATE & LOAD COURSE
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

    let course = Course::load(&course_path);

    // Version Sync
    if game.course_version != course.version {
        game.course_version = course.version.clone();
        game.save(ctx.save_path.to_str().unwrap())
            .expect("Failed to save game state");
    }

    let world = WorldEngine::new();
    world.initialize();

    if game.current_quest_id.is_empty() {
        if let Some(first_quest) = course.quests.first() {
            game.current_quest_id = first_quest.id.clone();
            game.save(ctx.save_path.to_str().unwrap())
                .expect("Failed to save game state");
        }
    }

    // 6. RUN GAME LOOP
    if let Some(cmd) = args.check {
        let exit_code = handle_check_command(cmd, &mut game, &course, &ctx.save_path, &world);
        std::process::exit(exit_code);
    } else if args.status {
        handle_status_display(&game, &course);
    } else if args.refresh {
        handle_refresh_sequence(&game, &course);
    } else {
        // DEFAULT: Launch the Infection
        shell::launch_infected_session();
    }
    Ok(())
}

// --- HELPER FUNCTIONS ---

fn resolve_course_path(game: &GameState, lib: &Library) -> Option<std::path::PathBuf> {
    if !game.current_course.is_empty() {
        let path = lib.root_dir.join(&game.current_course);
        if path.exists() {
            return Some(path);
        }
    }
    // Default to intro if exists
    let intro = lib.root_dir.join("intro.yaml");
    if intro.exists() {
        return Some(intro);
    }
    None
}

fn show_menu(lib: &Library) -> Option<std::path::PathBuf> {
    let courses = lib.list_available_courses();
    if courses.is_empty() {
        println!(">> No modules found in {:?}", lib.root_dir);
        return None;
    }

    println!("\n>> AVAILABLE MODULES:");
    for (i, (_, name)) in courses.iter().enumerate() {
        println!("   [{}] {}", i + 1, name);
    }

    // Simple selection logic for now
    println!("\n>> Selecting module 1 by default (Menu UI WIP)");
    courses.first().map(|(path, _)| path.clone())
}

fn reset_game(save_path: &Path) -> GameState {
    if save_path.exists() {
        std::fs::remove_file(save_path).expect("Failed to delete save file");
    }
    println!(">> [SYSTEM] Save Data Wiped.");
    GameState::new()
}

// --- GAMEPLAY HANDLERS ---
fn perform_validation(path_str: &str) {
    let path = std::path::Path::new(path_str);
    if !path.exists() {
        println!(">> [ERROR] File not found: {}", path_str);
        return;
    }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            println!(">> [ERROR] Could not read file: {}", e);
            return;
        }
    };

    match serde_yml::from_str::<Course>(&content) {
        Ok(course) => {
            println!(">> [SUCCESS] YAML Syntax is valid.");
            println!(">> Title:   {}", course.title);
            println!(">> Author:  {}", course.author);
            println!(">> Version: {}", course.version);
        }
        Err(e) => {
            println!(">> [FAIL] YAML Parsing Error:");
            println!("{}", e);
        }
    }
}

fn handle_refresh_sequence(game: &GameState, course: &Course) {
    // 1. PLAY INTRO (If new chapter)
    if game.current_task_index == 0 {
        if let Some((_, chapter, _)) = course.get_active_content(
            &game.current_quest_id,
            game.current_chapter_index,
            game.current_task_index,
        ) {
            ui::play_cutscene(&chapter.intro);

            // 2. CLEAR SCREEN
            // Only clear if we played an intro, so the Status Card pops fresh
            print!("\x1b[2J\x1b[H");
        }
    } else {
        // Newline if no clear screen needed
        println!();
    }

    // 3. SHOW STATUS
    handle_status_display(game, course);
}

fn handle_status_display(game: &GameState, course: &Course) {
    if game.is_finished {
        println!(">> [SYSTEM] Quest Complete. Run 'supershell --menu' for more.");
        return;
    }

    if let Some((quest, chapter, task)) = course.get_active_content(
        &game.current_quest_id,
        game.current_chapter_index,
        game.current_task_index,
    ) {
        ui::draw_status_card(
            &quest.title,
            &chapter.title,
            &task.instruction,
            &task.objective,
            game.current_task_index + 1,
            chapter.tasks.len(),
        );
    }
}

fn handle_check_command(
    user_cmd: String,
    game: &mut GameState,
    course: &Course,
    save_path: &Path,
    world: &WorldEngine,
) -> i32 {
    // 1. Setup Logic (Lazy Init)
    if game.current_task_index == 0 {
        if let Some(quest) = course.quests.iter().find(|q| q.id == game.current_quest_id) {
            if let Some(chapter) = quest.chapters.get(game.current_chapter_index) {
                if !chapter.setup_actions.is_empty() {
                    world.build_scenario(&chapter.setup_actions);
                }
            }
        }
    }

    if let Some((quest, chapter, task)) = course.get_active_content(
        &game.current_quest_id,
        game.current_chapter_index,
        game.current_task_index,
    ) {
        // --- PASS 1: RELEVANCE (Permissive) ---
        // If it doesn't match the regex, allow it to run silently (Exit 0)
        if !is_command_relevant(&user_cmd, task, game) {
            return 0;
        }

        // --- PASS 2: LOGIC (Strict) ---
        // It matches regex. If logic fails (wrong permissions), BLOCK it (Exit 1).
        if let Err(msg) = validate_task_logic(&user_cmd, task, game) {
            ui::print_fail(&msg, "Review system requirements.");
            return 1;
        }

        // --- SUCCESS ---
        println!("\r\n");
        ui::print_success(&task.success_msg);

        // Rewards
        apply_rewards(game, &task.rewards);

        let progression = advance_progress(game, chapter.tasks.len(), quest.chapters.len());

        // Handle Transitions
        match progression {
            Progression::NextTask => {
                // If we didn't play a cutscene, we should pause so the user
                // sees the "Success" message before the screen clears.
                println!("\n\x1b[0;90m[ PRESS ENTER TO CONTINUE ]\x1b[0m");
                let mut s = String::new();
                std::io::stdin().read_line(&mut s).unwrap();
            }
            Progression::NextChapter => {
                ui::play_cutscene(&chapter.outro);

                // World Building for NEXT chapter
                let next_chapter = &quest.chapters[game.current_chapter_index];
                if !next_chapter.setup_actions.is_empty() {
                    println!(">> [SYSTEM] Reconfiguring Construct...");
                    world.build_scenario(&next_chapter.setup_actions);
                }
            }
            Progression::ModuleComplete => {
                ui::play_cutscene(&chapter.outro);
                println!("\n\x1b[1;32m>> [SYSTEM] ALL MODULES COMPLETE. DISCONNECTING...\x1b[0m");

                game.save(save_path.to_str().unwrap())
                    .expect("Failed to save game state");

                return 0;
            }
        }

        game.save(save_path.to_str().unwrap())
            .expect("Failed to save game state");
        // Return Exit Code 2 to tell Bash to refresh the UI
        return 2;
    }

    // Default: No change, Exit 0
    0
}
