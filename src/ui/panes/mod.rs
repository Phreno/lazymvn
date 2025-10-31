mod basic_panes;
mod layout;
mod popups;
mod tab_footer;

pub use basic_panes::*;
pub use layout::create_adaptive_layout;
pub use popups::*;
pub use tab_footer::{render_footer, render_tab_bar};
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::keybindings::Focus;
    use ratatui::layout::Rect;

    #[test]
    fn test_adaptive_layout_narrow_terminal() {
        // Narrow terminal (< 80 cols) should use single column layout
        let area = Rect {
            x: 0,
            y: 0,
            width: 60,
            height: 40,
        };

        let (_, _, modules_area, _, _, output_area, _) =
            create_adaptive_layout(area, Some(Focus::Modules));

        // In single column, all panes should have the same x position (stacked vertically)
        assert_eq!(modules_area.x, output_area.x);
        // Modules should be below projects
        assert!(modules_area.y > 0);
        // Output should be below modules
        assert!(output_area.y > modules_area.y);
    }

    #[test]
    fn test_adaptive_layout_wide_terminal() {
        // Wide terminal (>= 190 cols) should use two column layout
        // With 190 cols: left=40 (max), output=150 (meets minimum)
        let area = Rect {
            x: 0,
            y: 0,
            width: 190,
            height: 40,
        };

        let (_, _, modules_area, _, _, output_area, _) =
            create_adaptive_layout(area, Some(Focus::Modules));

        // In two column, output should be to the right of modules
        assert!(output_area.x > modules_area.x);
        // Output should have at least 150 chars
        assert!(output_area.width >= 150);
    }

    #[test]
    fn test_adaptive_layout_short_height_expands_focused() {
        // Short terminal (< 30 rows) should collapse non-focused panes
        // Use 190 cols to ensure two-column mode (output gets 150 chars)
        let area = Rect {
            x: 0,
            y: 0,
            width: 190,
            height: 20,
        };

        // Focus on modules
        let (_, projects_area, modules_area, profiles_area, _, _, _) =
            create_adaptive_layout(area, Some(Focus::Modules));

        // Modules (focused) should have more height than others
        assert!(modules_area.height > projects_area.height);
        assert!(modules_area.height > profiles_area.height);
    }

    #[test]
    fn test_adaptive_layout_normal_height_standard_layout() {
        // Normal height terminal should use standard layout
        // Use 190 cols to ensure two-column mode (output gets 150 chars)
        let area = Rect {
            x: 0,
            y: 0,
            width: 190,
            height: 40,
        };

        let (_, projects_area, modules_area, profiles_area, flags_area, _, _) =
            create_adaptive_layout(area, Some(Focus::Modules));

        // Projects should be small (length 3)
        assert_eq!(projects_area.height, 3);
        // Other panes should have reasonable sizes
        assert!(modules_area.height > 5);
        assert!(profiles_area.height > 5);
        assert!(flags_area.height > 5);
    }
}
