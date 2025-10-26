//! Loading screen with progress indicator and animated ASCII art logo

use ratatui::{
    Frame, Terminal,
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};
use std::io;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

// ASCII art logo frames for animation
const LOGO_FRAMES: &[&str] = &[
    // Frame 1 - Normal
    r#"
    ██╗      █████╗ ███████╗██╗   ██╗███╗   ███╗██╗   ██╗███╗   ██╗
    ██║     ██╔══██╗╚══███╔╝╚██╗ ██╔╝████╗ ████║██║   ██║████╗  ██║
    ██║     ███████║  ███╔╝  ╚████╔╝ ██╔████╔██║██║   ██║██╔██╗ ██║
    ██║     ██╔══██║ ███╔╝    ╚██╔╝  ██║╚██╔╝██║╚██╗ ██╔╝██║╚██╗██║
    ███████╗██║  ██║███████╗   ██║   ██║ ╚═╝ ██║ ╚████╔╝ ██║ ╚████║
    ╚══════╝╚═╝  ╚═╝╚══════╝   ╚═╝   ╚═╝     ╚═╝  ╚═══╝  ╚═╝  ╚═══╝
"#,
    // Frame 2 - Bright
    r#"
    ██╗      █████╗ ███████╗██╗   ██╗███╗   ███╗██╗   ██╗███╗   ██╗
    ██║     ██╔══██╗╚══███╔╝╚██╗ ██╔╝████╗ ████║██║   ██║████╗  ██║
    ██║     ███████║  ███╔╝  ╚████╔╝ ██╔████╔██║██║   ██║██╔██╗ ██║
    ██║     ██╔══██║ ███╔╝    ╚██╔╝  ██║╚██╔╝██║╚██╗ ██╔╝██║╚██╗██║
    ███████╗██║  ██║███████╗   ██║   ██║ ╚═╝ ██║ ╚████╔╝ ██║ ╚████║
    ╚══════╝╚═╝  ╚═╝╚══════╝   ╚═╝   ╚═╝     ╚═╝  ╚═══╝  ╚═╝  ╚═══╝
"#,
];

pub struct LoadingProgress {
    current_step: usize,
    total_steps: usize,
    current_message: String,
    spinner_frame: usize,
    logo_frame: usize,
}

impl LoadingProgress {
    pub fn new(total_steps: usize) -> Self {
        Self {
            current_step: 0,
            total_steps,
            current_message: String::from("Initializing..."),
            spinner_frame: 0,
            logo_frame: 0,
        }
    }

    pub fn set_step(&mut self, step: usize, message: String) {
        self.current_step = step;
        self.current_message = message;
        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
        self.logo_frame = (self.logo_frame + 1) % LOGO_FRAMES.len();
    }

    pub fn progress(&self) -> f64 {
        if self.total_steps == 0 {
            0.0
        } else {
            (self.current_step as f64 / self.total_steps as f64) * 100.0
        }
    }

    pub fn render<B: Backend>(&self, terminal: &mut Terminal<B>) -> io::Result<()> {
        terminal.draw(|f| {
            let area = f.area();

            // Center the loading screen
            let vertical = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Length(20),
                    Constraint::Percentage(20),
                ])
                .split(area);

            let horizontal = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(5),
                    Constraint::Percentage(90),
                    Constraint::Percentage(5),
                ])
                .split(vertical[1]);

            let loading_area = horizontal[1];

            self.render_loading_box(f, loading_area);
        })?;
        Ok(())
    }

    fn render_loading_box(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(7), // ASCII art logo
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Subtitle
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Progress bar
                Constraint::Length(1), // Spacer
                Constraint::Length(2), // Message with spinner
            ])
            .split(area);

        // Block border
        let block = Block::default().borders(Borders::ALL).border_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

        f.render_widget(block, area);

        // Animated ASCII art logo
        let logo_color = if self.logo_frame == 0 {
            Color::Cyan
        } else {
            Color::LightCyan
        };

        let logo_text = LOGO_FRAMES[self.logo_frame];
        let logo_lines: Vec<Line> = logo_text
            .lines()
            .skip(1) // Skip first empty line
            .map(|line| {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(logo_color).add_modifier(Modifier::BOLD),
                ))
            })
            .collect();

        let logo = Paragraph::new(logo_lines).alignment(Alignment::Center);
        f.render_widget(logo, chunks[0]);

        // Subtitle
        let subtitle = vec![Line::from(vec![Span::styled(
            "⚡ Maven Terminal UI ⚡",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )])];
        let subtitle_widget = Paragraph::new(subtitle).alignment(Alignment::Center);
        f.render_widget(subtitle_widget, chunks[2]);

        // Progress bar
        let progress = self.progress();
        let gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(
                Style::default()
                    .fg(Color::Cyan)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .percent(progress as u16)
            .label(format!("{}%", progress as u16));
        f.render_widget(gauge, chunks[4]);

        // Current step message with spinner
        let spinner = if self.current_step < self.total_steps {
            SPINNER_FRAMES[self.spinner_frame]
        } else {
            "✓"
        };

        let message = vec![Line::from(vec![
            Span::styled(
                format!("{} ", spinner),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&self.current_message, Style::default().fg(Color::White)),
            Span::styled(
                format!(" [{}/{}]", self.current_step, self.total_steps),
                Style::default().fg(Color::DarkGray),
            ),
        ])];
        let message_widget = Paragraph::new(message).alignment(Alignment::Center);
        f.render_widget(message_widget, chunks[6]);
    }
}

/// Helper macro to update loading progress and render
#[macro_export]
macro_rules! loading_step {
    ($progress:expr, $terminal:expr, $step:expr, $total:expr, $msg:expr) => {{
        $progress.set_step($step, $msg.to_string());
        $progress.render($terminal)?;
    }};
}
