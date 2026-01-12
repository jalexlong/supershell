mod quest;
mod state;
mod ui;

use clap::Parser;
use directories::ProjectDirs;
use quest::Curriculum;
use state::GameState;
use std::fs;
use std::path::PathBuf;
use ui::play_cutscene;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    check: Option<String>,
    #[arg(long)]
    reset: bool,
}

fn main() {
    let args = Cli::parse();

    // 1. PATH DISCOVERY
    let proj_dirs = ProjectDirs::from("com", "jalexlong", "supershell")
        .expect("Could not determine home directory");
    let data_dir = proj_dirs.data_dir();
    let save_path = data_dir.join("save.json");

    // Look for installed quests.yaml first (Production),
    // fall back to local file (Development).
    let installed_quest_path = data_dir.join("quests.yaml");
    let dev_quest_path = PathBuf::from("quests.yaml");

    let local_quest_path = if installed_quest_path.exists() {
        installed_quest_path
    } else {
        // Fallback for when you run 'cargo run' locally
        dev_quest_path
    };

    // Ensure data directory exists
    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    // 2. RESET LOGIC
    if args.reset {
        if save_path.exists() {
            fs::remove_file(&save_path).expect("Failed to delete save file");
            println!("\r\n>> [SYSTEM] Save state wiped. Resetting to start of YAML.");
        }
        return;
    }

    // 3. LOAD DATA & STATE
    let curriculum = Curriculum::load(&local_quest_path);
    let mut game = GameState::load(save_path.to_str().unwrap());

    let quest_exists = curriculum
        .quests
        .iter()
        .any(|q| q.id == game.current_quest_id);

    if game.current_quest_id.is_empty() || !quest_exists {
        if let Some(first_quest) = curriculum.quests.first() {
            println!(
                ">> [SYSTEM] Auto-selecting first quest: {}",
                first_quest.title
            );
            game.current_quest_id = first_quest.id.clone();
            game.current_chapter_index = 0;
            game.current_task_index = 0;
            game.is_finished = false;
            game.save(save_path.to_str().unwrap());
        } else {
            eprintln!(">> [ERROR] No quests found in quests.yaml!");
            return;
        }
    }

    // 4. MAIN LOOP
    if let Some(user_cmd) = args.check {
        // --- CHECK MODE (Triggered by shell hook) ---

        if game.is_finished {
            return;
        } // Silent exit if game is over

        // Attempt to find the currently active task object
        if let Some((quest, chapter, task)) = curriculum.get_active_content(
            &game.current_quest_id,
            game.current_chapter_index,
            game.current_task_index,
        ) {
            // Verify Conditions
            if task.conditions.iter().all(|c| c.is_met(&user_cmd)) {
                // A. Task Complete
                println!("\n\x1b[32m>> [SUCCESS] {}\x1b[0m", task.success_msg);

                // B. Peek at the future (for "Next Objective" text)
                // We do this BEFORE updating state so we know if we are crossing a chapter boundary
                let next_step_info = curriculum.find_next_step(
                    &game.current_quest_id,
                    game.current_chapter_index,
                    game.current_task_index,
                );

                // C. Update State
                game.advance_task();

                // D. Check Chapter Boundary
                // If the new task index is beyond the bounds of the current chapter
                if game.current_task_index >= chapter.tasks.len() {
                    // 1. Finish Current Chapter (Outro)
                    println!("");
                    println!("\x1b[32m>> [MISSION COMPLETE] Objective Verified.\x1b[0m");
                    println!("\x1b[90m>> Press [ENTER] to begin transmission...\x1b[0m");

                    // Stop until user hits Enter
                    let mut buffer = String::new();
                    let _ = std::io::stdin().read_line(&mut buffer);

                    play_cutscene(&chapter.outro);

                    // 2. Advance State to Next Chapter
                    game.advance_chapter();

                    // 3. Check Quest Boundary
                    if game.current_chapter_index >= quest.chapters.len() {
                        // QUEST COMPLETE
                        game.is_finished = true;
                        println!("\r\n>> [SYSTEM] ALL QUESTS COMPLETE.")
                    } else {
                        // NEW CHAPTER STARTS
                        let next_chapter = &quest.chapters[game.current_chapter_index];
                        play_cutscene(&next_chapter.intro);

                        // Show first objective of next chapter
                        if let Some(info) = next_step_info {
                            println!("\r\n[NEXT OBJECTIVE]");
                            println!("INSTRUCTION: {}", info.instruction);
                            println!("OBJECTIVE:   {}", info.objective);
                        }
                    }
                } else {
                    // E. Normal Task Advance (Same Chapter)
                    if let Some(info) = next_step_info {
                        println!("\r\n[NEXT OBJECTIVE]");
                        println!("INSTRUCTION: {}", info.instruction);
                        println!("OBJECTIVE:   {}", info.objective);
                    }
                }

                game.save(save_path.to_str().unwrap());
            }
        }
    } else {
        // --- MANUAL MODE (Running `supershell` directly) ---

        if game.is_finished {
            println!(">> [SYSTEM] Status: 100% Complete. System Stable.");
            println!(">> Run 'supershell --reset' to restart.");
            return;
        }

        if let Some((_, chapter, task)) = curriculum.get_active_content(
            &game.current_quest_id,
            game.current_chapter_index,
            game.current_task_index,
        ) {
            // If it's the very first task of a chapter, user might need the briefing again
            if game.current_task_index == 0 {
                play_cutscene(&chapter.intro);
            }

            println!("\r\n[CURRENT STATUS: {}]", chapter.title);
            println!("INSTRUCTION: {}", task.instruction);
            println!("OBJECTIVE: {}", task.objective);
        } else {
            println!(">> [ERROR] Save state does not match Quest Database.");
            println!(">> Try running 'supershell --reset'.");
        }
    }
}
