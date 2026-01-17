use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyCode, KeyEventKind, poll, read},
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size},
};
use std::env;
use std::io::{Write, stdout};
use std::thread;
use std::time::Duration;
use textwrap::fill;

// --- THE SAFETY NET ---
// We create a struct just to handle the terminal state.
// When this variable goes out of scope (at the end of the function),
// the 'drop' function fires automatically.
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> Self {
        enable_raw_mode().expect("Failed to enable raw mode");
        let mut stdout = stdout();
        execute!(stdout, Hide, MoveTo(0, 0)).unwrap(); // Hide cursor
        Self
    }
}

// This runs AUTOMATICALLY when the cutscene ends or crashes
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = stdout();
        disable_raw_mode().unwrap_or(()); // Force raw mode off
        execute!(
            stdout,
            Show, // Bring cursor back
            ResetColor,
            SetAttribute(Attribute::Reset),
            Clear(ClearType::All), // Clean up our mess
            MoveTo(0, 0)           // Reset position
        )
        .unwrap_or(());
    }
}

// --- THE RENDERER ---
pub fn play_cutscene(text: &str) {
    // If we are testing, print plain text and exit IMMEDIATELY
    // This prevents enabling raw mode (which crashes tests) and prevents waiting for Enter.
    if env::var("SUPERSHELL_TEST_MODE").is_ok() {
        println!("\n{}\n", text);
        return;
    }

    // A. Initialize the Guard (Enables Raw Mode immediately)
    let _guard = TerminalGuard::new();
    let mut stdout = stdout();

    // WRAPPING
    let (cols, _) = size().unwrap_or((80, 24));

    // Calculate a safe width (Terminal Width - 4 columns for padding)
    // We stick to a max of 80 chars for readability, even on wide monitors.
    let wrap_width = if cols > 4 {
        (cols - 4) as usize
    } else {
        cols as usize
    };

    // 'fill' intelligently wraps the text at word boundaries
    let wrapped_text = fill(text, wrap_width);

    // Set the Mood (Hacker Green)
    execute!(
        stdout,
        Clear(ClearType::All),
        MoveTo(0, 0),
        SetForegroundColor(Color::Green),
        SetAttribute(Attribute::Bold)
    )
    .unwrap();

    // The Typewriter Loop
    let mut skipped = false;

    for char in wrapped_text.chars() {
        // PRINT LOGIC
        if char == '\n' {
            execute!(stdout, Print("\r\n")).unwrap();
        } else {
            // Otherwise, print normally
            print!("{}", char);
            stdout.flush().unwrap();
        }

        // CHECK FOR SKIP (Non-blocking check)
        if !skipped {
            // Check if key is pressed (Non-blocking)
            if poll(Duration::from_secs(0)).unwrap() {
                if let Event::Key(key) = read().unwrap() {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
                        skipped = true;
                    }
                }
            }
            // THE DELAY
            // We only sleep if the user hasn't skipped yet.
            // If skipped is true, this block is ignored, and the loop
            // spins as fast as the CPU allows (instantly filling the text).
            thread::sleep(Duration::from_millis(25));
        }
    }

    // PROMPT
    execute!(
        stdout,
        Print("\r\n\r\n"),
        SetForegroundColor(Color::DarkGrey),
        Print(">> PRESS [ENTER] TO CONTINUE_")
    )
    .unwrap();

    // Loop until Enter is pressed
    loop {
        if let Event::Key(key) = read().unwrap() {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
                break;
            }
        }
    }
}
