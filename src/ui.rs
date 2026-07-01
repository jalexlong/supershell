use crossterm::{
    cursor::{MoveDown, MoveToColumn, MoveUp},
    event::{Event, KeyCode, KeyEventKind, poll, read},
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size},
};
use std::env;
use std::io::{Write, stdout};
use std::thread;
use std::time::Duration;
use textwrap::fill;

// --- VISUAL CONSTANTS ---
const BORDER_COLOR: Color = Color::White;
const TITLE_COLOR: Color = Color::White;
const TEXT_COLOR: Color = Color::White;

const MAX_WIDTH: u16 = 80;

// --- PUBLIC API ---

pub fn play_cutscene(text: &str) {
    render_inline_card("MISSION BRIEF", vec![text.to_string()], true);
}

pub fn draw_status_card(
    title: &str,
    chapter: &str,
    instruction: &str,
    objective: &str,
    current: usize,
    total: usize,
) {
    let content = vec![
        format!("📂 {}", chapter),
        String::from(""),
        format!("🎯 OBJECTIVE: {}", objective),
        String::from(""),
        format!("💡 HINT: {}", instruction),
        String::from(""),
        format!("[Progress: {}/{}]", current, total),
    ];
    render_inline_card(title, content, false);
}

pub fn print_success(msg: &str) {
    let mut stdout = stdout();
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print("\r\n>> [SUCCESS] "),
        SetAttribute(Attribute::Reset),
        SetForegroundColor(Color::White),
        Print(msg),
        Print("\n\n")
    )
    .ok();
}

pub fn print_fail(error: &str, hint: &str) {
    let mut stdout = stdout();

    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print("[+] SYSTEM_INTEGRITY.. OK\n"),
        Print("[+] SYNTAX_VALIDATION. OK\n"),
        SetForegroundColor(Color::Red),
        Print("[-] EXECUTION......... FAIL\n"),
        Print("    └── "),
        Print(error),
        Print("\n\n"),
    )
    .ok();

    if !hint.is_empty() {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print(" >> HINT: "),
            SetAttribute(Attribute::Bold),
            Print(hint),
            SetAttribute(Attribute::Reset),
            Print("\n"),
        )
        .ok();
    }
    execute!(stdout, ResetColor).ok();
}

// --- CORE RENDERING ENGINE ---

fn render_inline_card(title: &str, raw_lines: Vec<String>, use_typewriter: bool) {
    // Bypass for test environments.
    if env::var("SUPERSHELL_TEST_MODE").is_ok() {
        render_plain_card(title, &raw_lines, false);
        return;
    }

    // Enter raw mode; fall back to plain renderer if unavailable.
    if enable_raw_mode().is_err() {
        render_plain_card(title, &raw_lines, use_typewriter);
        return;
    }
    let mut stdout = stdout();

    let (term_cols, term_rows) = size().unwrap_or((80, 24));

    // Fall back to plain renderer for constrained terminals.
    if term_cols < 50 || term_rows < 16 {
        disable_raw_mode().ok();
        render_plain_card(title, &raw_lines, use_typewriter);
        return;
    }

    let width = std::cmp::min(term_cols, MAX_WIDTH);
    let content_width = (width as usize).saturating_sub(4);

    let mut final_lines = Vec::new();
    for raw_line in raw_lines {
        if raw_line.is_empty() {
            final_lines.push("".to_string());
        } else {
            let wrapped = fill(&raw_line, content_width);
            for subline in wrapped.split('\n') {
                final_lines.push(subline.to_string());
            }
        }
    }

    let height = final_lines.len();

    // Draw skeleton.
    print_top_border(width, title);

    execute!(stdout, SetForegroundColor(BORDER_COLOR)).ok();
    for _ in 0..height {
        print!("│");
        print!("{}", " ".repeat(content_width + 2));
        execute!(stdout, MoveToColumn(width - 1)).ok();
        print!("│\r\n");
    }
    print_bottom_border(width);

    // Rewind to top of content area.
    execute!(stdout, MoveUp((height + 1) as u16)).ok();

    // Fill content.
    let mut skipped = false;

    for line in final_lines {
        execute!(stdout, MoveToColumn(2)).ok();
        execute!(stdout, SetForegroundColor(TEXT_COLOR)).ok();

        if use_typewriter {
            for char in line.chars() {
                print!("{}", char);
                stdout.flush().ok();

                if !skipped {
                    if poll(Duration::from_secs(0)).unwrap_or(false) {
                        if let Ok(Event::Key(key)) = read() {
                            if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
                                skipped = true;
                            }
                        }
                    }
                    if !skipped {
                        thread::sleep(Duration::from_millis(15));
                    }
                }
            }
        } else {
            print!("{}", line);
        }

        execute!(stdout, MoveDown(1)).ok();
    }

    // Move past the bottom border.
    execute!(stdout, MoveDown(1)).ok();
    execute!(stdout, MoveToColumn(0), ResetColor).ok();

    if use_typewriter {
        print!("\r\n>> PRESS [ENTER] TO CONTINUE...");
        stdout.flush().ok();

        loop {
            match read() {
                Ok(Event::Key(key))
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter =>
                {
                    break;
                }
                Err(_) => break,
                _ => {}
            }
        }
        print!("\r\n");
    } else {
        print!("\r\n");
    }

    // CRITICAL: always restore the terminal before returning.
    disable_raw_mode().ok();
}

fn render_plain_card(title: &str, raw_lines: &[String], use_pause: bool) {
    println!();
    println!("== {} ==", title);

    for raw_line in raw_lines {
        if raw_line.is_empty() {
            println!();
            continue;
        }
        println!("{}", raw_line);
    }

    if use_pause {
        println!();
        println!(">> PRESS [ENTER] TO CONTINUE...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
    } else {
        println!();
    }
}

// --- BORDER HELPERS ---

fn print_top_border(width: u16, title: &str) {
    let mut stdout = stdout();
    execute!(stdout, SetForegroundColor(BORDER_COLOR)).ok();
    print!("┌──");

    execute!(
        stdout,
        SetForegroundColor(TITLE_COLOR),
        SetAttribute(Attribute::Bold)
    )
    .ok();
    print!(" {} ", title);

    execute!(
        stdout,
        SetForegroundColor(BORDER_COLOR),
        SetAttribute(Attribute::Reset)
    )
    .ok();

    let used_len = 3 + 1 + title.chars().count() + 1;
    let remaining = (width as usize).saturating_sub(used_len + 1);

    print!("{}", "─".repeat(remaining));
    execute!(stdout, MoveToColumn(width - 1)).ok();
    print!("┐\r\n");
}

fn print_bottom_border(width: u16) {
    let mut stdout = stdout();
    execute!(stdout, SetForegroundColor(BORDER_COLOR)).ok();
    print!("└{}", "─".repeat((width as usize).saturating_sub(2)));
    execute!(stdout, MoveToColumn(width - 1)).ok();
    print!("┘\r\n");
    execute!(stdout, ResetColor).ok();
}
