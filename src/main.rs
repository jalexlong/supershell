pub mod actions;
mod quest;
mod state;
mod ui;
mod world;

use clap::Parser;
use directories::ProjectDirs;
use quest::{Course, Library, NextStepInfo};
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
}

fn main() {
    let args = Cli::parse();

    // 1. PATH DISCOVERY
    let proj_dirs = ProjectDirs::from("com", "jalexlong", "supershell")
        .expect("Could not determine home directory");

    let data_dir = proj_dirs.data_dir();
    let save_path = data_dir.join("save.json");
    let library_path = data_dir.join("library");

    // Ensure directories exists
    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    // 2. LOAD STATE
    let mut game = GameState::load(save_path.to_str().unwrap());

    // Handle Reset
    if args.reset {
        if save_path.exists() {
            fs::remove_file(&save_path).expect("Failed to delete save file");
            println!("\r\n>> [SYSTEM] Save state wiped. Resetting to start of YAML.");
        }
        game = GameState::new();
    }

    // 3. MENU SELECTION LOGIC
    // We show the menu if:
    // A. The user passed --menu
    // B. The user has no active course selected
    // C. The active course file is missing

    let mut active_course_path = library_path.join(&game.current_course);

    if args.menu || game.current_course.is_empty() || !active_course_path.exists() {
        println!("\n╔══════════════════════════════════════╗");
        println!("║           S U P E R S H E L L        ║");
        println!("╠══════════════════════════════════════╣");

        let lib = Library::new(library_path.clone());
        let courses = lib.list_available_courses();

        if courses.is_empty() {
            println!("║  [ERROR] No courses found in:        ║");
            println!("║  {:?}  ║", library_path);
            println!("║                                      ║");
            println!("║  Please run the installer again.     ║");
            println!("╚══════════════════════════════════════╝");
            return;
        }

        for (i, (_, name)) in courses.iter().enumerate() {
            println!("║  [{}] {:<30} ║", i + 1, name);
        }
        println!("╚══════════════════════════════════════╝");
        print!(">> SELECT MODULE_ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim().parse::<usize>().unwrap_or(0);

        if choice > 0 && choice <= courses.len() {
            let (path, name) = &courses[choice - 1];
            println!(">> Loading Module: {}", name);

            game.current_course = path.file_name().unwrap().to_string_lossy().to_string();
            game.current_quest_id = String::new();
            game.current_chapter_index = 0;
            game.current_task_index = 0;
            game.is_finished = false;
            game.save(save_path.to_str().unwrap());

            active_course_path = path.clone();
        } else {
            println!(">> [ERROR] Invalid Selection.");
            return;
        }
    }

    // 4. LOAD THE COURSE
    let course = Course::load(&active_course_path);
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

    // Check for "New Game" Setup Actions
    if game.current_chapter_index == 0 && game.current_task_index == 0 {
        if let Some(quest) = course.quests.iter().find(|q| q.id == game.current_quest_id) {
            if let Some(chapter) = quest.chapters.first() {
                if args.check.is_none() && !chapter.setup_actions.is_empty() {
                    println!(">> [SYSTEM] Initializing Construct...");
                    world.build_scenario(&chapter.setup_actions);
                }
            }
        }
    }

    // 5. MAIN LOOP
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
                game.advance_task();

                let mut next_step_info: Option<NextStepInfo> = None;

                if game.current_task_index >= chapter.tasks.len() {
                    // Chapter Complete
                    play_cutscene(&chapter.outro);
                    game.advance_chapter();

                    if game.current_chapter_index >= quest.chapters.len() {
                        println!(">> [QUEST COMPLETE] {}", quest.title);
                        game.is_finished = true;
                    } else {
                        // New Chapter
                        let next_chapter = &quest.chapters[game.current_chapter_index];
                        if !next_chapter.setup_actions.is_empty() {
                            println!(">> [SYSTEM] Reconfiguring Construct...");
                            world.build_scenario(&next_chapter.setup_actions);
                        }
                        play_cutscene(&next_chapter.intro);

                        if let Some(first_task) = next_chapter.tasks.first() {
                            next_step_info = Some(NextStepInfo {
                                instruction: first_task.instruction.clone(),
                                objective: first_task.objective.clone(),
                            });
                        }
                    }
                } else {
                    if let Some(next_task) = chapter.tasks.get(game.current_task_index) {
                        next_step_info = Some(NextStepInfo {
                            instruction: next_task.instruction.clone(),
                            objective: next_task.objective.clone(),
                        });
                    }
                }

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

        if let Some((_, chapter, task)) = course.get_active_content(
            &game.current_quest_id,
            game.current_chapter_index,
            game.current_task_index,
        ) {
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
