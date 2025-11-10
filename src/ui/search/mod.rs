mod highlighter;
mod matcher;
mod status;
mod types;

pub use highlighter::search_line_style;
pub use matcher::collect_search_matches;
pub use status::search_status_line;
pub use types::{SearchMatch, SearchState};
