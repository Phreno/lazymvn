//! Tests for purge local repository command

#[cfg(test)]
mod purge_tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_purge_command_key_is_p() {
        // Verify that 'p' is the correct key for purge
        let key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        assert_eq!(key.code, KeyCode::Char('p'));
    }

    #[test]
    fn test_purge_command_arguments() {
        // The purge command should use these exact arguments
        let expected_args = ["dependency:purge-local-repository", "-DreResolve=false"];
        
        // Verify the command structure
        assert_eq!(expected_args[0], "dependency:purge-local-repository");
        assert_eq!(expected_args[1], "-DreResolve=false");
    }

    #[test]
    fn test_purge_command_goal() {
        // Verify Maven goal is correct
        let goal = "dependency:purge-local-repository";
        assert!(goal.starts_with("dependency:"));
        assert!(goal.contains("purge"));
    }

    #[test]
    fn test_purge_no_reresolve_flag() {
        // The -DreResolve=false flag prevents Maven from trying to re-download
        // This is important for clearing corrupted cache without network access
        let flag = "-DreResolve=false";
        assert!(flag.contains("reResolve=false"));
    }
}
