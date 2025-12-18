mod quest;
mod state;
mod ui;

use clap::Parser;
use directories::ProjectDirs;
use quest::{Condition, load_quests};
use state::GameState;
use std::fs;
use std::path::PathBuf;
use ui::play_cutscene;

#[derive(Parser)]
struct Cli {
    /// The user command to validate
    #[arg(long)]
    check: Option<String>,
}

fn main() {
    let args = Cli::parse();

    // 1. DISCOVER STANDARD PATHS (XDG)
    // qualifier="com", organization="jalexlong", application="supershell"
    // On Linux, this maps to: ~/.local/share/supershell
    let proj_dirs = ProjectDirs::from("com", "jalexlong", "supershell")
        .expect("Could not determine home directory");

    let data_dir = proj_dirs.data_dir();

    // 2. ENSURE DATA DIRECTORY EXISTS
    if !data_dir.exists() {
        fs::create_dir_all(data_dir).expect("Failed to create game data directory");
    }

    // 3. DEFINE FILE PATHS
    // Save file ALWAYS goes to the system data folder
    let save_path = data_dir.join("save.json");

    // Quest file strategy: Check System -> Fallback to Local (Dev)
    let system_quest_path = data_dir.join("quests.yaml");
    let local_quest_path = PathBuf::from("quests.yaml");

    let final_quest_path = if system_quest_path.exists() {
        system_quest_path
    } else if local_quest_path.exists() {
        // Fallback for when you are developing in the source folder
        local_quest_path
    } else {
        // Panic if we can't find the game content anywhere
        eprintln!("CRITICAL ERROR: 'quests.yaml' not found in:");
        eprintln!("  1. System: {:?}", system_quest_path);
        eprintln!("  2. Local:  {:?}", local_quest_path);
        std::process::exit(1);
    };

    // Convert to string for our loader functions
    let save_file = save_path.to_str().unwrap();
    let quest_file = final_quest_path.to_str().unwrap();

    // 4. LOAD ENGINE
    let mut game = GameState::load(save_file);
    let quest_db = load_quests(quest_file);

    // --- LOGIC LOOP (Same as before) ---
    if let Some(user_cmd) = args.check {
        let current_quest = quest_db.get(&game.current_quest_id);

        if let Some(quest) = current_quest {
            let all_met = quest
                .conditions
                .iter()
                .all(|c: &Condition| c.is_met(&user_cmd));

            if all_met {
                play_cutscene(&quest.message);
                game.complete_current_quest(&quest.next_quest_id);
                game.save(save_file);
            }
        }
    } else {
        // DASHBOARD / INTRO
        if game.completed_quests.is_empty() && game.current_quest_id == "00_init" {
            let boot_sequence = "SUPERSHELL DAEMON v1.0.2
INITIALIZING KERNEL MODULES... [OK]
MOUNTING VIRTUAL TUTOR... [OK]

[STATUS]
Standard shell functionality: ACTIVE.
Supershell overlay: ACTIVE.

[PROTOCOL 00: CALIBRATION]
Before complex tasks can be assigned, input/output integrity must be verified.
Demonstrate control of the standard output stream.

[TASK]
Execute the 'echo' command with the string 'hello'.
Command: echo hello";
            play_cutscene(boot_sequence);
        } else {
            println!("\n=== 🛡️  SUPERSHELL DAEMON STATUS 🛡️  ===");
            if let Some(q) = quest_db.get(&game.current_quest_id) {
                println!("CURRENT PROTOCOL: {}", q.name);
                println!("STATUS: Awaiting input matching protocol criteria.");
            } else {
                println!("STATUS: No active protocols. System idle.");
            }
            println!("==========================================\n");
        }
    }
}
