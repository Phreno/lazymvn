//! Adaptive layout management
//!
//! This module handles dynamic layout calculation based on terminal size
//! and focused pane, supporting both narrow (single column) and wide (two column) modes.

use crate::ui::keybindings::Focus;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Create adaptive layout based on terminal size and focused pane
///
/// Returns: (tab_bar, projects, modules, profiles, flags, output, footer)
pub fn create_adaptive_layout(
    area: Rect,
    focused_pane: Option<Focus>,
) -> (Rect, Rect, Rect, Rect, Rect, Rect, Rect) {
    let tab_bar_height = 2;
    let footer_height = 10;

    // Determine layout mode based on output pane comfort
    // In two-column mode, left pane takes min(30%, 40 chars)
    // We want output to have at least 150 chars for readable logs
    const MIN_OUTPUT_WIDTH: u16 = 150;
    const MAX_LEFT_WIDTH: u16 = 40;
    
    // Calculate what the left width would be in two-column mode
    let potential_left_width = ((area.width * 30) / 100).min(MAX_LEFT_WIDTH);
    let potential_output_width = area.width.saturating_sub(potential_left_width);
    
    // Switch to single column if output would be too narrow
    let is_narrow = potential_output_width < MIN_OUTPUT_WIDTH;
    let is_short = area.height < 30; // Short height threshold

    // Split vertically: tab bar, content, footer
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(tab_bar_height),
                Constraint::Min(0),
                Constraint::Length(footer_height),
            ]
            .as_ref(),
        )
        .split(area);

    let tab_bar_area = vertical[0];
    let content_area = vertical[1];
    let footer_area = vertical[2];

    // Adaptive width layout for content
    let (projects_area, modules_area, profiles_area, flags_area, output_area, _footer) =
        if is_narrow {
            // Single column mode - stack everything vertically
            create_single_column_layout(content_area, footer_area, focused_pane, is_short)
        } else {
            // Two column mode - left panes and output
            create_two_column_layout(content_area, footer_area, focused_pane, is_short)
        };

    (
        tab_bar_area,
        projects_area,
        modules_area,
        profiles_area,
        flags_area,
        output_area,
        footer_area,
    )
}

/// Create single column layout for narrow terminals
///
/// In single column mode, all panes are stacked vertically.
/// The focused pane is expanded while others are collapsed.
pub(super) fn create_single_column_layout(
    content_area: Rect,
    footer_area: Rect,
    focused_pane: Option<Focus>,
    is_short: bool,
) -> (Rect, Rect, Rect, Rect, Rect, Rect) {
    // In single column, show focused pane expanded, others collapsed
    let constraints = if is_short {
        // Very restrictive - only show focused pane
        match focused_pane {
            Some(Focus::Projects) => vec![
                Constraint::Min(5),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
            Some(Focus::Modules) => vec![
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
            Some(Focus::Profiles) => vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
            Some(Focus::Flags) => vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Length(1),
            ],
            Some(Focus::Output) | None => vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(5),
            ],
        }
    } else {
        // Normal single column - collapse non-focused panes (like two column mode)
        match focused_pane {
            Some(Focus::Projects) => vec![
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Percentage(30),
            ],
            Some(Focus::Modules) => vec![
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Percentage(30),
            ],
            Some(Focus::Profiles) => vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Percentage(30),
            ],
            Some(Focus::Flags) => vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Percentage(30),
            ],
            Some(Focus::Output) | None => vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(5),
            ],
        }
    };

    let blocks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(content_area);

    (
        blocks[0],
        blocks[1],
        blocks[2],
        blocks[3],
        blocks[4],
        footer_area,
    )
}

/// Create two column layout for normal/wide terminals
///
/// In two column mode, left panes (projects, modules, profiles, flags)
/// are in one column, and output occupies the right column.
/// The left column is limited to a maximum of 40 characters to avoid
/// wasting space on large screens.
///
/// Note: This layout is only used when output width would be >= 150 chars.
/// See create_adaptive_layout for the threshold logic.
pub(super) fn create_two_column_layout(
    content_area: Rect,
    footer_area: Rect,
    focused_pane: Option<Focus>,
    is_short: bool,
) -> (Rect, Rect, Rect, Rect, Rect, Rect) {
    // Calculate left column width: 30% of screen, but max 40 columns
    // This must match the calculation in create_adaptive_layout
    const MAX_LEFT_WIDTH: u16 = 40;
    let left_width = ((content_area.width * 30) / 100).min(MAX_LEFT_WIDTH);
    let right_width = content_area.width.saturating_sub(left_width);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(left_width), Constraint::Length(right_width)].as_ref())
        .split(content_area);

    // Adaptive left pane layout based on height and focus
    let left_constraints = if is_short {
        // Short height - expand focused pane, collapse others
        match focused_pane {
            Some(Focus::Projects) => vec![
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ],
            Some(Focus::Modules) => vec![
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(3),
            ],
            Some(Focus::Profiles) => vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ],
            Some(Focus::Flags) => vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(5),
            ],
            Some(Focus::Output) | None => vec![
                Constraint::Length(3),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ],
        }
    } else {
        // Normal height - standard layout
        vec![
            Constraint::Length(3),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ]
    };

    let left_blocks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(left_constraints)
        .split(content_chunks[0]);

    (
        left_blocks[0],
        left_blocks[1],
        left_blocks[2],
        left_blocks[3],
        content_chunks[1],
        footer_area,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::keybindings::Focus;

    #[test]
    fn test_adaptive_layout_narrow_terminal() {
        let area = Rect::new(0, 0, 70, 40);
        let (tab, proj, _mods, _profs, _flags, out, foot) =
            create_adaptive_layout(area, Some(Focus::Output));

        assert_eq!(tab.height, 2);
        assert_eq!(foot.height, 10);
        // Verify output area got expanded in single column mode
        assert!(out.height > proj.height);
    }

    #[test]
    fn test_adaptive_layout_wide_terminal() {
        let area = Rect::new(0, 0, 120, 40);
        let (tab, _proj, _mods, _profs, _flags, out, foot) =
            create_adaptive_layout(area, Some(Focus::Output));

        assert_eq!(tab.height, 2);
        assert_eq!(foot.height, 10);
        // In two column mode, output should be ~70% width
        assert!(out.width > 80);
    }

    #[test]
    fn test_adaptive_layout_short_height_expands_focused() {
        let area = Rect::new(0, 0, 80, 20); // Short height
        let (_tab, proj, mods, profs, flags, _out, _foot) =
            create_adaptive_layout(area, Some(Focus::Modules));

        // Modules should be expanded, others collapsed
        assert!(mods.height > proj.height);
        assert!(mods.height > profs.height);
        assert!(mods.height > flags.height);
    }

    #[test]
    fn test_adaptive_layout_normal_height_standard_layout() {
        let area = Rect::new(0, 0, 100, 40); // Normal height
        let (_tab, proj, mods, profs, flags, _out, _foot) = create_adaptive_layout(area, None);

        // Standard layout - modules should have reasonable space
        assert!(proj.height >= 3);
        assert!(mods.height >= 3);
        assert!(profs.height >= 3);
        assert!(flags.height >= 3);
    }

    #[test]
    fn test_adaptive_layout_limits_left_column_width() {
        // Very wide terminal (200 columns)
        let area = Rect::new(0, 0, 200, 40);
        let (_tab, proj, _mods, _profs, _flags, out, _foot) = create_adaptive_layout(area, None);

        // Left column should be capped at 40 columns
        assert!(proj.width <= 40, "Left column width should not exceed 40, got {}", proj.width);
        
        // Output should take the rest
        assert!(out.width >= 160, "Output should use remaining space, got {}", out.width);
    }

    #[test]
    fn test_adaptive_layout_normal_width_uses_percentage() {
        // Wide enough terminal (200 columns) to use two-column layout
        let area = Rect::new(0, 0, 200, 40);
        let (_tab, proj, _mods, _profs, _flags, out, _foot) = create_adaptive_layout(area, None);

        // Left column should be capped at 40 (max)
        assert!(proj.width <= 40, "Left column should be capped at 40, got {}", proj.width);
        
        // Output should be ~160 (200 - 40)
        assert!(out.width >= 150, "Output should have at least 150 chars, got {}", out.width);
    }

    #[test]
    fn test_adaptive_layout_switches_to_single_column_for_narrow_output() {
        // Terminal where output would be < 150 chars in two-column mode
        // With 100 cols: left=30, output=70 → too narrow, switches to single column
        let area = Rect::new(0, 0, 100, 40);
        let (_tab, proj, _mods, _profs, _flags, out, _foot) = create_adaptive_layout(area, None);

        // Should use single column layout where all panes have full width
        assert_eq!(proj.width, 100, "Should use single column (full width), got {}", proj.width);
        assert_eq!(out.width, 100, "Output should use full width in single column, got {}", out.width);
        
        // In single column, output should be stacked below other panes
        assert!(out.y > proj.y, "Output should be below projects in single column");
    }
}
