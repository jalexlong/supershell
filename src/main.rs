mod quest;
mod state;
mod ui;

use clap::Parser;
use directories::ProjectDirs;
use quest::{Chapter, load_chapters};
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
    // Using env! to find quests.yaml relative to your project root
    let local_quest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("quests.yaml");

    let proj_dirs = ProjectDirs::from("com", "jalexlong", "supershell")
        .expect("Could not determine home directory");
    let save_path = proj_dirs.data_dir().join("save.json");

    // Ensure data directory exists
    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    // 2. LOAD CHAPTER DATABASE
    let yaml_content = fs::read_to_string(&local_quest_path).unwrap_or_default();
    let chapters_list: Vec<Chapter> = serde_yml::from_str(&yaml_content).unwrap_or_default();
    let start_id = chapters_list
        .first()
        .map(|c| c.id.clone())
        .unwrap_or_else(|| "START".into());

    // 3. RESET LOGIC
    if args.reset {
        if save_path.exists() {
            fs::remove_file(&save_path).expect("Failed to delete save file");
            println!("\r\n>> [SYSTEM] Save state wiped. Resetting to start of YAML.");
        }
        return;
    }

    // 4. LOAD GAME STATE
    let chapter_db = load_chapters(local_quest_path.to_str().unwrap());
    let mut game = GameState::load(save_path.to_str().unwrap(), start_id);

    // 5. EXECUTION LOGIC
    if let Some(user_cmd) = args.check {
        if game.is_finished {
            return;
        } // End-of-game silence

        if let Some(chapter) = chapter_db.get(&game.current_chapter_id) {
            if let Some(checkpoint) = chapter.checkpoints.get(game.current_checkpoint_index) {
                if checkpoint.conditions.iter().all(|c| c.is_met(&user_cmd)) {
                    // Success message
                    play_cutscene(&checkpoint.success);

                    // Advance to next Checkpoint
                    if (game.current_checkpoint_index + 1) < chapter.checkpoints.len() {
                        game.advance_checkpoint();

                        if let Some(next_cp) =
                            chapter.checkpoints.get(game.current_checkpoint_index)
                        {
                            println!("\r\n[NEXT OBJECTIVE]");
                            println!("INSTRUCTION: {}", next_cp.instruction);
                            println!("OBJECTIVE:   {}", next_cp.objective);
                        }
                    } else {
                        // Chapter Complete
                        play_cutscene(&chapter.debriefing);

                        if let Some(next_id) = &chapter.next_chapter_id {
                            game.move_to_chapter(next_id.clone());
                            if let Some(next_ch) = chapter_db.get(next_id) {
                                play_cutscene(&next_ch.briefing);

                                // Show first objective of new chapter
                                if let Some(first_cp) = next_ch.checkpoints.get(0) {
                                    println!("\r\n[NEXT OBJECTIVE]");
                                    println!("INSTRUCTION: {}", first_cp.instruction);
                                    println!("OBJECTIVE:   {}", first_cp.objective);
                                }
                            }
                        } else {
                            game.is_finished = true;
                            println!("\r\n>> [SYSTEM] All diagnostic protocols complete.");
                        }
                    }
                    game.save(save_path.to_str().unwrap());
                }
            }
        }
    } else {
        // MANUAL MODE
        if game.is_finished {
            println!(">> [SYSTEM] Status: 100% Complete. System Stable.");
            println!(">> Run 'supershell --reset' to restart.");
        } else if let Some(chapter) = chapter_db.get(&game.current_chapter_id) {
            if let Some(checkpoint) = chapter.checkpoints.get(game.current_checkpoint_index) {
                if game.current_checkpoint_index == 0 {
                    play_cutscene(&chapter.briefing);
                }
                println!("\r\n[CURRENT STATUS: {}]", chapter.title);
                println!("OBJECTIVE: {}", checkpoint.objective);
            }
        }
    }
}
