// src/discovery/mod.rs

use std::path::{Path, PathBuf};

pub struct DirectoryFinder {
    start: PathBuf,
}

impl DirectoryFinder {
    pub fn new(start: PathBuf) -> Self {
        Self { start }
    }

    pub fn find_components_ui(&self) -> Result<PathBuf, DiscoveryError> {
        // Strategy 1: Look for existing components/
        let target = self.start.join("components");
        if target.exists() {
            return Ok(target);
        }

        // Strategy 2: Look for src/components/
        let src_target = self.start.join("src").join("components");
        if src_target.exists() {
            return Ok(src_target);
        }

        // Strategy 3: Create at components//
        Ok(target)
    }
}

#[derive(Debug)]
pub enum DiscoveryError {
    NoProjectRoot,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn find_components_ui_when_components_exists() {
        let dir = tempdir().unwrap();
        let start = dir.path().to_path_buf();

        // Create components/ directly under start
        let components_path = start.join("components");
        fs::create_dir(&components_path).unwrap();

        let finder = DirectoryFinder::new(start);
        let result = finder.find_components_ui().unwrap();

        assert_eq!(result, components_path);
        assert!(result.exists());
    }

    #[test]
    fn find_components_ui_when_src_components_exists() {
        let dir = tempdir().unwrap();
        let start = dir.path().to_path_buf();

        // Create src/components/
        let src_components = start.join("src").join("components");
        fs::create_dir_all(&src_components).unwrap();

        let finder = DirectoryFinder::new(start.clone());
        let result = finder.find_components_ui().unwrap();

        // Should return src/components, not create components/
        assert_eq!(result, src_components);
        assert!(result.exists());

        // Also ensure that components/ was NOT created
        assert!(!start.join("components").exists());
    }

    #[test]
    fn find_components_ui_when_neither_exists() {
        let dir = tempdir().unwrap();
        let start = dir.path().to_path_buf();

        let finder = DirectoryFinder::new(start.clone());
        let result = finder.find_components_ui().unwrap();

        // Should return start/components (even though it doesn't exist)
        let expected = start.join("components");
        assert_eq!(result, expected);
        assert!(!result.exists()); // not created
    }

    #[test]
    fn find_components_ui_when_start_path_does_not_exist() {
        // Use a path that definitely does not exist
        let start = PathBuf::from("/this/path/does/not/exist/12345");

        let finder = DirectoryFinder::new(start.clone());
        let result = finder.find_components_ui().unwrap();

        // Should return start/components, regardless of existence
        let expected = start.join("components");
        assert_eq!(result, expected);
        // The path itself doesn't exist, but that's fine
    }

    #[test]
    fn find_components_ui_returns_components_over_src_components() {
        let dir = tempdir().unwrap();
        let start = dir.path().to_path_buf();

        // Create both components/ and src/components/
        let components = start.join("components");
        fs::create_dir(&components).unwrap();

        let src_components = start.join("src").join("components");
        fs::create_dir_all(&src_components).unwrap();

        let finder = DirectoryFinder::new(start);
        let result = finder.find_components_ui().unwrap();

        // Should prefer components/ over src/components/ (order of checks)
        assert_eq!(result, components);
    }

    #[test]
    fn discovery_error_debug() {
        // Ensure Debug is implemented (trivial)
        let err = DiscoveryError::NoProjectRoot;
        let _ = format!("{:?}", err);
    }
}
