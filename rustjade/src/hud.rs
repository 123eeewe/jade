//! Консольный рендер HUD — имитация тултипа Jade на экране.
//!
//! В реальном моде этот модуль заменяется на GuiGraphics-рендер
//! (Fabric/Forge). Здесь тултип печатается в терминале с ANSI-цветами,
//! чтобы можно было запустить интерактивную демку без Minecraft.

use crate::core::Tooltip;
use crate::text::{Color, Component};

/// ANSI-код цвета для Color.
fn ansi(c: Color) -> &'static str {
    use Color::*;
    match c {
        Black => "\x1b[30m",
        DarkRed => "\x1b[31m",
        Green | DarkGreen => "\x1b[32m",
        Gold | Yellow => "\x1b[33m",
        DarkBlue | Blue => "\x1b[34m",
        LightPurple | DarkPurple => "\x1b[35m",
        Aqua | DarkAqua => "\x1b[36m",
        Gray | White => "\x1b[37m",
        DarkGray => "\x1b[90m",
        Red => "\x1b[91m",
    }
}

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const ITALIC: &str = "\x1b[3m";

fn render_component(c: &Component, out: &mut String, parent: Option<Color>) {
    let color = c.style.color.or(parent);
    if c.style.bold {
        out.push_str(BOLD);
    }
    if c.style.italic {
        out.push_str(ITALIC);
    }
    if let Some(col) = color {
        out.push_str(ansi(col));
    }
    out.push_str(&c.text);
    for ch in &c.children {
        render_component(ch, out, color);
    }
    out.push_str(RESET);
}

/// Печать тултипа в стиле Minecraft: тёмная рамка, отступы.
pub fn print_tooltip(tip: &Tooltip) {
    // Собираем готовые ANSI-строки и их визуальную ширину.
    let mut rendered: Vec<(String, usize)> = Vec::new();

    let mut title = String::new();
    render_component(&tip.title, &mut title, None);
    rendered.push((title.clone(), vis_len(&title)));

    for l in &tip.lines {
        let mut body = String::new();
        render_component(&l.component, &mut body, None);
        let full = match &l.icon {
            Some(ic) => format!("{ic} {body}"),
            None => body,
        };
        rendered.push((full.clone(), vis_len(&full)));
    }

    let width = rendered.iter().map(|(_, w)| *w).max().unwrap_or(10).max(20);
    let top = "┌".to_string() + &"─".repeat(width + 2) + "┐";
    let bottom = "└".to_string() + &"─".repeat(width + 2) + "┘";
    println!("{top}");
    for (s, w) in &rendered {
        println!("│ {s}{} │", " ".repeat(width.saturating_sub(*w)));
    }
    println!("{bottom}");
}

/// Визуальная длина строки без ANSI-последовательностей.
fn vis_len(s: &str) -> usize {
    strip_ansi(s).chars().count()
}

pub fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\x1b' {
            // пропускаем ESC[ ... m
            if it.peek() == Some(&'[') {
                it.next();
                while let Some(&c2) = it.peek() {
                    it.next();
                    if c2 == 'm' {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}
