pub mod actions;
mod quest;
mod state;
mod ui;
mod world;

use clap::Parser;
use directories::ProjectDirs;
use include_dir::{Dir, include_dir};
use quest::{Course, Library};
use state::GameState;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use ui::play_cutscene;
use world::WorldEngine;

// --- CONSTANTS & EMBEDDED ASSETS ---

const VERSION: &str = env!("CARGO_PKG_VERSION");
static DEFAULT_LIBRARY: Dir = include_dir!("$CARGO_MANIFEST_DIR/library");

const SHELL_HOOK_SOURCE: &str = r#"
_supershell_hook() {
    local RETVAL=$?
    if [[ -n "$SUPERSHELL_BIN" && -x "$SUPERSHELL_BIN" ]]; then
        local LAST_CMD=""
        if [ -n "$ZSH_VERSION" ]; then
            LAST_CMD=$(fc -ln -1 | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
        elif [ -n "$BASH_VERSION" ]; then
            LAST_CMD=$(fc -ln -1 | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
        fi

        # Only check if command is not empty
        if [[ -n "$LAST_CMD" ]]; then
             "$SUPERSHELL_BIN" --check "$LAST_CMD"
        fi
    fi
    return $RETVAL
}

# INSTALLATION
# The Rust binary injects the specific path below
SUPERSHELL_BIN="__BINARY_PATH__"

if [ -n "$ZSH_VERSION" ]; then
    autoload -Uz add-zsh-hook
    if [[ "${precmd_functions[@]}" != *"_supershell_hook"* ]]; then
        add-zsh-hook precmd _supershell_hook
    fi
elif [ -n "$BASH_VERSION" ]; then
    if [[ ! "$PROMPT_COMMAND" =~ "_supershell_hook" ]]; then
        PROMPT_COMMAND="_supershell_hook;$PROMPT_COMMAND"
    fi
fi
"#;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    check: Option<String>,
    #[arg(long)]
    reset: bool,
    #[arg(long)]
    menu: bool,
    #[arg(long)]
    hint: bool,
    #[arg(long)]
    validate: Option<String>,
}

struct AppContext {
    data_dir: PathBuf,
    library_path: PathBuf,
    save_path: PathBuf,
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
    // (Validate, Update Library, Install Hook)
    if let Some(path_str) = args.validate {
        perform_validation(&path_str);
        return;
    }

    initialize_infrastructure(&ctx);

    // 3. LOAD GAME STATE
    let mut game = if args.reset {
        reset_game(&ctx.save_path)
    } else {
        GameState::load(ctx.save_path.to_str().unwrap())
    };

    // 4. RESOLVE COURSE
    // Determine which course we are playing (Active, Auto-load, or Menu Selection)
    let lib = Library::new(ctx.library_path.clone());
    let mut active_course_path = resolve_course_path(&game, &lib);

    if args.menu {
        active_course_path = show_menu(&lib);
        if let Some(ref path) = active_course_path {
            // Update state with new selection
            game.current_course = path.file_name().unwrap().to_string_lossy().to_string();
            game.current_quest_id = String::new(); // Reset progress on switch
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

    // Sync Version Metadata
    if game.course_version != course.version {
        if !game.course_version.is_empty() {
            println!(
                ">> [SYSTEM] Course updated: v{} -> v{}",
                game.course_version, course.version
            );
        }
        game.course_version = course.version.clone();
        game.save(ctx.save_path.to_str().unwrap());
    }

    // Initialize World Engine
    let world = WorldEngine::new();
    world.initialize();

    // Ensure state validity (if first load)
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
    } else {
        handle_status_dashboard(&game, &course);
    }
}

// --- INFRASTRUCTURE HELPERS ---

fn initialize_infrastructure(ctx: &AppContext) {
    // A. Ensure Data Dir
    if !ctx.data_dir.exists() {
        fs::create_dir_all(&ctx.data_dir).ok();
    }

    // B. Library Updates
    let version_path = ctx.library_path.join("version.txt");
    let lib_version = fs::read_to_string(&version_path).unwrap_or_else(|_| "0.0.0".to_string());

    if !ctx.library_path.exists() || lib_version.trim() != VERSION {
        if !ctx.library_path.exists() {
            println!(">> [SYSTEM] Initializing Quest Library (v{})...", VERSION);
            fs::create_dir_all(&ctx.library_path).expect("Failed to create library directory");
        } else {
            println!(
                ">> [SYSTEM] Engine updated (v{}). Synchronizing assets...",
                VERSION
            );
        }

        DEFAULT_LIBRARY
            .extract(&ctx.library_path)
            .expect("Failed to extract default library");
        fs::write(&version_path, VERSION).ok();
        println!(">> [SYSTEM] Library synchronized.");
    }

    // C. Shell Hook Self-Repair
    let hook_path = ctx.data_dir.join("init.sh");
    if let Ok(current_exe) = std::env::current_exe() {
        let exe_path = current_exe.to_string_lossy();
        let script_content = SHELL_HOOK_SOURCE.replace("__BINARY_PATH__", &exe_path);
        if let Err(e) = fs::write(&hook_path, script_content) {
            eprintln!(">> [ERROR] Failed to install shell hook: {}", e);
        }
    }
}

fn reset_game(save_path: &Path) -> GameState {
    if save_path.exists() {
        fs::remove_file(save_path).expect("Failed to delete save file");
        println!("\r\n>> [SYSTEM] Save state wiped.");
    }
    GameState::new()
}

// --- NAVIGATION HELPERS ---

fn resolve_course_path(game: &GameState, lib: &Library) -> Option<PathBuf> {
    if !game.current_course.is_empty() {
        let p = lib.root_dir.join(&game.current_course);
        if p.exists() {
            return Some(p);
        }
    }

    // Fallback to first available
    lib.list_available_courses().first().map(|(p, _)| p.clone())
}

fn show_menu(lib: &Library) -> Option<PathBuf> {
    println!("\n╔══════════════════════════════════════╗");
    println!("║            S U P E R S H E L L       ║");
    println!("╠══════════════════════════════════════╣");

    let courses = lib.list_available_courses();
    if courses.is_empty() {
        println!("║  [ERROR] No courses found            ║");
        println!("╚══════════════════════════════════════╝");
        return None;
    }

    for (i, (_, name)) in courses.iter().enumerate() {
        println!("║  [{:2}] {:<30} ║", i + 1, name);
    }
    println!("╚══════════════════════════════════════╝");
    print!(">> SELECT MODULE_ ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    if let Ok(choice) = input.trim().parse::<usize>() {
        if choice > 0 && choice <= courses.len() {
            let (path, name) = &courses[choice - 1];
            println!(">> Loading Module: {}", name);
            return Some(path.clone());
        }
    }
    println!(">> [ERROR] Invalid Selection.");
    None
}

// --- GAMEPLAY HANDLERS ---

fn handle_hint(game: &GameState, course: &Course) {
    if let Some((_, _, task)) = course.get_active_content(
        &game.current_quest_id,
        game.current_chapter_index,
        game.current_task_index,
    ) {
        println!("\r\n>> [SYSTEM] DECRYPTING HINT...");
        if task.hint.is_empty() {
            println!(">> [RESULT] No hint available for this task.");
        } else {
            println!(">> [HINT] {}", task.hint);
        }
    } else {
        println!(">> [ERROR] Unable to retrieve task context.");
    }
}

fn handle_status_dashboard(game: &GameState, course: &Course) {
    if game.is_finished {
        println!(">> [SYSTEM] Quest Complete. Run 'supershell --menu' for more.");
        return;
    }

    if let Some((quest, chapter, task)) = course.get_active_content(
        &game.current_quest_id,
        game.current_chapter_index,
        game.current_task_index,
    ) {
        // Construct Location Check
        if quest.construct {
            let current_dir = std::env::current_dir().unwrap_or_default();
            let user_home = directories::UserDirs::new()
                .unwrap()
                .home_dir()
                .to_path_buf();
            let construct_path = user_home.join("Construct");

            if !current_dir.starts_with(&construct_path) {
                println!("\r\n[WARNING: SIGNAL UNSTABLE]");
                println!("You are currently outside the Construct.");
                println!(">> REQUIRED: cd ~/Construct");
                return;
            }
        }

        // Auto-play Intro if new chapter
        if game.current_task_index == 0 {
            play_cutscene(&chapter.intro);
        }

        // Draw Dashboard
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
) {
    // 1. Setup Logic (Lazy Init)
    // Runs scenario setup if we are at the very start of a chapter
    if game.current_task_index == 0 {
        if let Some(quest) = course.quests.iter().find(|q| q.id == game.current_quest_id) {
            if let Some(chapter) = quest.chapters.get(game.current_chapter_index) {
                if !chapter.setup_actions.is_empty() {
                    world.build_scenario(&chapter.setup_actions);
                }
            }
        }
    }

    // 2. Validation Logic
    if let Some((quest, chapter, task)) = course.get_active_content(
        &game.current_quest_id,
        game.current_chapter_index,
        game.current_task_index,
    ) {
        let all_met = task.conditions.iter().all(|c| c.is_met(&user_cmd));

        if all_met {
            println!("\r\n"); // Spacer
            println!(">> [SUCCESS] {}", task.success_msg);

            // Look ahead for next step info
            let next_step_info = course.find_next_step(
                &game.current_quest_id,
                game.current_chapter_index,
                game.current_task_index,
            );

            game.advance_task();

            // Handle Transitions
            if game.current_task_index >= chapter.tasks.len() {
                play_cutscene(&chapter.outro);
                game.advance_chapter();

                if game.current_chapter_index >= quest.chapters.len() {
                    println!(">> [QUEST COMPLETE] {}", quest.title);
                    game.is_finished = true;
                } else {
                    // Pre-load next chapter setup
                    let next_chapter = &quest.chapters[game.current_chapter_index];
                    if !next_chapter.setup_actions.is_empty() {
                        println!(">> [SYSTEM] Reconfiguring Construct...");
                        world.build_scenario(&next_chapter.setup_actions);
                    }
                    play_cutscene(&next_chapter.intro);
                }
            } else if let Some(info) = next_step_info {
                // Peek at next task
                println!("\r\n[NEXT OBJECTIVE]");
                println!("INSTRUCTION: {}", info.instruction);
                println!("OBJECTIVE:    {}", info.objective);
            }

            game.save(save_path.to_str().unwrap());
        }
    }
}

// --- VALIDATION TOOL ---

fn perform_validation(file_path: &str) {
    let path = PathBuf::from(file_path);
    println!(">> [SYSTEM] Validating module: {:?}", path);

    if !path.exists() {
        println!(">> [ERROR] File not found.");
        return;
    }

    let content = match fs::read_to_string(&path) {
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

            let mut errors: Vec<String> = Vec::new();

            if course.title == "Untitled Course" {
                errors.push("Missing top-level field: 'title'".to_string());
            }
            if course.quests.is_empty() {
                errors.push("Course contains 0 quests.".to_string());
            }

            for (i, quest) in course.quests.iter().enumerate() {
                if quest.id.trim().is_empty() {
                    errors.push(format!("Quest #{} has missing ID.", i + 1));
                }
                if quest.chapters.is_empty() {
                    errors.push(format!("Quest '{}' has no chapters.", quest.title));
                }
            }

            if errors.is_empty() {
                println!(">> [RESULT] Module is VALID.");
            } else {
                println!(">> [RESULT] Logic Errors Found:");
                for e in errors {
                    println!("   - {}", e);
                }
            }
        }
        Err(e) => {
            println!(">> [FAIL] YAML Parsing Error:");
            println!("{}", e);
        }
    }
}
