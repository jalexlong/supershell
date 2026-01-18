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
use std::path::PathBuf;
use ui::play_cutscene;
use world::WorldEngine;

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
}

// EMBED THE LIBRARY FOLDER
static DEFAULT_LIBRARY: Dir = include_dir!("$CARGO_MANIFEST_DIR/library");

fn main() {
    let args = Cli::parse();

    // 1. PATH DISCOVERY
    let proj_dirs = ProjectDirs::from("com", "jalexlong", "supershell")
        .expect("Could not determine home directory");

    let data_dir = proj_dirs.data_dir();
    let save_path = data_dir.join("save.json");
    let library_path = data_dir.join("library");

    // Ensure save_path directories exists
    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    // Auto-extract assets
    if !library_path.exists() {
        println!(">> [SYSTEM] Initializing Construct environment...");

        // Create the library directory
        fs::create_dir_all(&library_path).expect("Failed to create library directory");

        // Extract the embedded files to the real path
        DEFAULT_LIBRARY
            .extract(&library_path)
            .expect("Failed to extract default library");

        println!(">> [SYSTEM] Core modules installed to: {:?}", library_path);
    }

    // 2. LOAD STATE
    let mut game = GameState::load(save_path.to_str().unwrap());

    // Handle Reset
    if args.reset {
        if save_path.exists() {
            fs::remove_file(&save_path).expect("Failed to delete save file");
            println!("\r\n>> [SYSTEM] Save state wiped.");
        }
        game = GameState::new();
    }

    // 3. COURSE RESOLUTION
    let lib = Library::new(library_path.clone());
    let courses = lib.list_available_courses();
    let mut active_course_path: Option<PathBuf> = None;

    if args.menu {
        println!("\n╔══════════════════════════════════════╗");
        println!("║           S U P E R S H E L L        ║");
        println!("╠══════════════════════════════════════╣");

        let lib = Library::new(library_path.clone());
        let courses = lib.list_available_courses();

        if courses.is_empty() {
            println!("║  [ERROR] No courses found            ║");
            println!("╚══════════════════════════════════════╝");
            return;
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

                game.current_course = path.file_name().unwrap().to_string_lossy().to_string();
                game.current_quest_id = String::new();
                game.current_chapter_index = 0;
                game.current_task_index = 0;
                game.is_finished = false;
                game.save(save_path.to_str().unwrap());

                active_course_path = Some(path.clone());
            } else {
                println!(">> [ERROR] Invalid Selection.");
                return;
            }
        } else {
            return;
        }
    }
    // B) No Menu Requested
    else {
        // Do we have a curent course?
        if !game.current_course.is_empty() {
            active_course_path = Some(library_path.join(&game.current_course));
        } else if let Some((path, _)) = courses.first() {
            // println!(">> [SYSTEM] Auto-loading Module: {}", name);
            game.current_course = path.file_name().unwrap().to_string_lossy().to_string();
            game.save(save_path.to_str().unwrap());
            active_course_path = Some(path.clone());
        }
    }

    // 4. VALIDATE & LOAD
    let course_path = match active_course_path {
        Some(p) => p,
        None => {
            if courses.is_empty() {
                println!(">> [ERROR] No quests found in library.");
                println!(">> Please reinstall or run 'supershell --reset'.");
            } else {
                println!(">> [ERROR] Could not load module. Run 'supershell --menu'.");
            }
            return;
        }
    };

    if !course_path.exists() {
        println!(">> [ERROR] Module file missing: {:?}", course_path);
        return;
    }

    // 5. RUN ENGINE
    let course = Course::load(&course_path);
    let world = WorldEngine::new();
    world.initialize();

    // Auto-select first quest if none active
    let quest_exists = course.quests.iter().any(|q| q.id == game.current_quest_id);
    if game.current_quest_id.is_empty() || !quest_exists {
        if let Some(first_quest) = course.quests.first() {
            game.current_quest_id = first_quest.id.clone();
            game.current_chapter_index = 0;
            game.current_task_index = 0;
            game.is_finished = false;
            game.save(save_path.to_str().unwrap());
        }
    }

    // New Game Setup Actions
    if game.current_chapter_index == 0 && game.current_task_index == 0 {
        if let Some(quest) = course.quests.iter().find(|q| q.id == game.current_quest_id) {
            if let Some(chapter) = quest.chapters.first() {
                if args.check.is_none() && !chapter.setup_actions.is_empty() {
                    // Only run setup if we haven't already (simple heuristic: are we at the very start?)
                    // Ideally, we'd have a 'chapter_initialized' flag, but this works for now.
                    // To prevent re-running on every 'status' check, we trust the idempotency of actions,
                    // OR we only run it if this is strictly the manual 'supershell' command, not '--check'.
                    println!(">> [SYSTEM] Initializing Construct...");
                    world.build_scenario(&chapter.setup_actions);
                }
            }
        }
    }

    if args.hint {
        if let Some((_, _, task)) = course.get_active_content(
            &game.current_quest_id,
            game.current_chapter_index,
            game.current_task_index,
        ) {
            println!("\r\n>> [SYSTEM HELP]");
            match &task.hint {
                Some(h) => println!("   {}", h),
                None => println!("   No additional data available. Reread the instruction."),
            }
        } else {
            println!(">> [ERROR] Cannot retrieve task data.");
        }
        return;
    }

    // 6. MAIN LOOP
    if let Some(user_cmd) = args.check {
        // --- CHECK MODE (Triggered by shell hook) ---
        if let Some((quest, chapter, task)) = course.get_active_content(
            &game.current_quest_id,
            game.current_chapter_index,
            game.current_task_index,
        ) {
            // Verify Conditions
            let mut all_met = true;
            for condition in &task.conditions {
                if !condition.is_met(&user_cmd) {
                    all_met = false;
                    break;
                }
            }

            if all_met {
                println!("\r\n>> [SUCCESS] {}", task.success_msg);

                // 1. Look ahead before advancing
                let next_step_info = course.find_next_step(
                    &game.current_quest_id,
                    game.current_chapter_index,
                    game.current_task_index,
                );

                // 2. Update the state (move the indices)
                game.advance_task();

                // 3. Handle transitions and cutscenes
                if game.current_task_index >= chapter.tasks.len() {
                    // Chapter Complete
                    play_cutscene(&chapter.outro);
                    game.advance_chapter();

                    if game.current_chapter_index >= quest.chapters.len() {
                        println!(">> [QUEST COMPLETE] {}", quest.title);
                        game.is_finished = true;
                    } else {
                        // New Chapter Intro
                        let next_chapter = &quest.chapters[game.current_chapter_index];
                        if !next_chapter.setup_actions.is_empty() {
                            println!(">> [SYSTEM] Reconfiguring Construct...");
                            world.build_scenario(&next_chapter.setup_actions);
                        }
                        play_cutscene(&next_chapter.intro);
                    }
                }

                // 4. Print the data (if found)
                if let Some(info) = next_step_info {
                    println!("\r\n[NEXT OBJECTIVE]");
                    println!("INSTRUCTION: {}", info.instruction);
                    println!("OBJECTIVE:   {}", info.objective);
                }

                game.save(save_path.to_str().unwrap());
            }
        }
    } else {
        // --- MANUAL MODE (Running `supershell` directly) ---
        if game.is_finished {
            println!(">> [SYSTEM] Quest Complete. Run 'supershell --menu' for more.");
            return;
        }

        if let Some((quest, chapter, task)) = course.get_active_content(
            &game.current_quest_id,
            game.current_chapter_index,
            game.current_task_index,
        ) {
            // --- CHECK LOCATION ---
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
                    println!("Return to base to receive instructions.");
                    println!(">> REQUIRED: cd ~/Construct");
                    return;
                }
            }
            // Optional: Replay intro if at the very start of a chapter
            if game.current_task_index == 0 {
                play_cutscene(&chapter.intro);
            }

            println!("\r\n[CURRENT STATUS: {}]", chapter.title);
            println!("INSTRUCTION: {}", task.instruction);
            println!("OBJECTIVE: {}", task.objective);
        }
    }
}
