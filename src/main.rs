pub mod actions;
mod quest;
mod shell;
mod state;
mod ui;
mod world; // <--- The new module

use clap::Parser;
use directories::ProjectDirs;
use include_dir::{Dir, include_dir};
use quest::{ConditionType, Course, Library, Reward, ValidationResult};
use state::GameState;
use std::path::Path;
use ui::play_cutscene;
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
    hint: bool,

    #[arg(long)]
    reset: bool,

    #[arg(long)]
    validate: Option<String>,

    #[arg(long)]
    menu: bool,
}

// --- APP CONTEXT ---

struct AppContext {
    data_dir: std::path::PathBuf,
    library_path: std::path::PathBuf,
    save_path: std::path::PathBuf,
}

// --- MAIN ENTRY POINT ---

fn main() {
    let args = Cli::parse();

    // 1. SETUP PATHS
    let proj_dirs = ProjectDirs::from("com", "jalexlong", "supershell")
        .expect("Could not determine home directory");

    let ctx = AppContext {
        data_dir: proj_dirs.data_dir().to_path_buf(),
        library_path: proj_dirs.data_dir().join("library"),
        save_path: proj_dirs.data_dir().join("save.json"),
    };

    // 2. SYSTEM OPERATIONS
    if let Some(path_str) = args.validate {
        perform_validation(&path_str);
        return;
    }

    // Ensure Library Exists
    if !ctx.library_path.exists() {
        std::fs::create_dir_all(&ctx.library_path).expect("Failed to create library dir");
        DEFAULT_LIBRARY
            .extract(&ctx.library_path)
            .expect("Failed to extract default library");
    }

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
            game.save(ctx.save_path.to_str().unwrap());
        }
    }

    // 5. VALIDATE & LOAD COURSE
    let course_path = match active_course_path {
        Some(p) => p,
        None => {
            eprintln!(">> [ERROR] No module selected. Run 'supershell --menu'.");
            return;
        }
    };

    if !course_path.exists() {
        eprintln!(">> [ERROR] Module file missing: {:?}", course_path);
        return;
    }

    let course = Course::load(&course_path);

    // Version Sync
    if game.course_version != course.version {
        game.course_version = course.version.clone();
        game.save(ctx.save_path.to_str().unwrap());
    }

    let world = WorldEngine::new();
    world.initialize();

    if game.current_quest_id.is_empty() {
        if let Some(first_quest) = course.quests.first() {
            game.current_quest_id = first_quest.id.clone();
            game.save(ctx.save_path.to_str().unwrap());
        }
    }

    // 6. RUN GAME LOOP
    if args.hint {
        handle_hint(&game, &course);
    } else if let Some(cmd) = args.check {
        handle_check_command(cmd, &mut game, &course, &ctx.save_path, &world);
    } else if args.menu {
        // Already handled above
    } else {
        // DEFAULT: Launch the Infection
        shell::launch_infected_session();
    }
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

fn handle_hint(game: &GameState, course: &Course) {
    if let Some((_, _, task)) = course.get_active_content(
        &game.current_quest_id,
        game.current_chapter_index,
        game.current_task_index,
    ) {
        ui::print_hint(&task.hint);
    } else {
        println!(">> [SYSTEM] No active task.");
    }
}

fn handle_check_command(
    user_cmd: String,
    game: &mut GameState,
    course: &Course,
    save_path: &Path,
    world: &WorldEngine,
) {
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
        let is_relevant = task
            .conditions
            .iter()
            .filter(|c| matches!(c.condition_type, ConditionType::CommandMatches { .. }))
            .any(|c| matches!(c.check(&user_cmd, game), ValidationResult::Valid));

        if !is_relevant {
            std::process::exit(0);
        }

        // --- PASS 2: LOGIC (Strict) ---
        // It matches regex. If logic fails (wrong permissions), BLOCK it (Exit 1).
        for condition in &task.conditions {
            if matches!(
                condition.condition_type,
                ConditionType::CommandMatches { .. }
            ) {
                continue;
            }

            match condition.check(&user_cmd, game) {
                ValidationResult::Valid => continue,
                ValidationResult::LogicError(msg) => {
                    ui::print_fail(&msg, "Review system requirements.");
                    std::process::exit(1);
                }
                _ => {}
            }
        }

        // --- SUCCESS ---
        println!("\r\n");
        ui::print_success(&task.success_msg);

        // Rewards
        for reward in &task.rewards {
            match reward {
                Reward::SetFlag { key, value } => game.set_flag(key, *value),
                Reward::SetVar { key, value } => game.set_var(key, *value),
                Reward::AddVar { key, amount } => game.mod_var(key, *amount),
            }
        }

        game.advance_task();

        // Handle Transitions
        if game.current_task_index >= chapter.tasks.len() {
            ui::play_cutscene(&chapter.outro);
            game.advance_chapter();

            if game.current_chapter_index >= quest.chapters.len() {
                println!(">> [QUEST COMPLETE] {}", quest.title);
                game.is_finished = true;
            } else {
                let next_chapter = &quest.chapters[game.current_chapter_index];
                if !next_chapter.setup_actions.is_empty() {
                    println!(">> [SYSTEM] Reconfiguring Construct...");
                    world.build_scenario(&next_chapter.setup_actions);
                }
                ui::play_chapter_intro(&next_chapter.title, &next_chapter.intro);
                if let Some(first_task) = next_chapter.tasks.first() {
                    ui::draw_status_card(
                        "NEW MODULE",
                        &next_chapter.title,
                        &first_task.instruction,
                        &first_task.objective,
                        1,
                        next_chapter.tasks.len(),
                    );
                }
            }
        } else {
            if let Some((_, _, next_task)) = course.get_active_content(
                &game.current_quest_id,
                game.current_chapter_index,
                game.current_task_index,
            ) {
                ui::draw_status_card(
                    "MISSION UPDATE",
                    &chapter.title,
                    &next_task.instruction,
                    &next_task.objective,
                    game.current_task_index + 1,
                    chapter.tasks.len(),
                );
            }
        }

        game.save(save_path.to_str().unwrap());
        std::process::exit(0);
    }

    // Fallback
    std::process::exit(0);
}

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

    match serde_yaml::from_str::<Course>(&content) {
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
