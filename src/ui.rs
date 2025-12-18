use crossterm::{
    cursor::{Hide, MoveTo, MoveToColumn, MoveToNextLine, Show},
    event::{Event, KeyCode, KeyEventKind, poll, read},
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size},
};
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
    let chars: Vec<char> = text.chars().collect();
    let mut skipped = false;

    for char in chars {
        if char == '\n' {
            // MANUAL FIX: When we hit a newline...
            // 1. Go down one line (resets to col 0)
            // 2. Move right to col 2 (to maintain indentation)
            execute!(stdout, MoveToNextLine(1), MoveToColumn(1)).unwrap();
        } else {
            // Otherwise, print normally
            print!("{}", char);
            stdout.flush().unwrap();
        }

        // CHECK FOR SKIP (Non-blocking check)
        // We poll for 0 seconds to see if a key is waiting
        if poll(Duration::from_secs(0)).unwrap() {
            if let Event::Key(key) = read().unwrap() {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char(' ') {
                    skipped = true;
                    break; // Stop typing, jump to end
                }
            }
        }

        // The "Rhythm" (25ms)
        if !skipped {
            thread::sleep(Duration::from_millis(25));
        }
    }

    // SKIP HANDLER (Instant Print)
    if skipped {
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0)).unwrap();
        // We manually inject the alignment for the fast-print block
        // \r = Return to 0, \n = Down, "  " = Indent to 2
        let aligned_block = wrapped_text.replace("\n", "\r\n ");

        print!("{}", aligned_block);
        stdout.flush().unwrap();
    }

    // PROMPT
    execute!(
        stdout,
        Print("\r\n\r\n"),
        SetForegroundColor(Color::DarkGrey),
        Print(">> PRESS [SPACE] TO CONTINUE_")
    )
    .unwrap();

    // Loop until Space is pressed
    loop {
        if let Event::Key(key) = read().unwrap() {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char(' ') {
                break;
            }
        }
    }
}
