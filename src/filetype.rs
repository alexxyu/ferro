use serde::Deserialize;

use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

/// The file type of a document.
#[derive(Deserialize)]
pub struct FileType {
    /// The type associated with this [FileType] (e.g. "Rust" for ".rs" files)
    name: String,
    /// The filename extensions associated with this [FileType] (e.g. ".rs" for Rust)
    extension: Vec<String>,
    /// The associated [HighlightingOptions] for this [FileType]
    highlighting_options: HighlightingOptions,
}

/// The highlighting options that determine what gets highlighted in a file.
#[derive(Default, Deserialize)]
pub struct HighlightingOptions {
    numbers: bool,
    characters: bool,
    strings: Option<Vec<char>>,
    comments: Option<String>,
    multiline_comments: Option<Vec<(String, String)>>,
    primary_keywords: Vec<String>,
    secondary_keywords: Vec<String>,
}

impl HighlightingOptions {
    pub fn numbers(&self) -> bool {
        self.numbers
    }
    pub fn characters(&self) -> bool {
        self.characters
    }
    pub fn strings(&self) -> &Option<Vec<char>> {
        &self.strings
    }
    pub fn comments(&self) -> &Option<String> {
        &self.comments
    }
    pub fn multiline_comments(&self) -> &Option<Vec<(String, String)>> {
        &self.multiline_comments
    }
    pub fn primary_keywords(&self) -> &Vec<String> {
        &self.primary_keywords
    }
    pub fn secondary_keywords(&self) -> &Vec<String> {
        &self.secondary_keywords
    }
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            name: String::from("No filetype"),
            extension: vec![],
            highlighting_options: HighlightingOptions::default(),
        }
    }
}

impl FileType {
    /// Constructs the FileType based on a given filename
    ///
    /// # Arguments
    ///
    /// * `filename` - the name of the file
    pub fn from(filename: &str) -> Self {
        let filename_extension = String::from(
            Path::new(filename)
                .extension()
                .and_then(OsStr::to_str)
                .unwrap_or(""),
        );

        let configs = fs::read_dir(Path::new("src").join("filetype_config")).unwrap();
        for config in configs {
            let file = File::open(config.unwrap().path()).unwrap();
            let reader = BufReader::new(file);
            let u: Self = serde_json::from_reader(reader).unwrap();
            if u.extension.contains(&filename_extension) {
                return u;
            }
        }

        Self::default()
    }

    /// Gets the name of the FileType.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Gets the [HighlightingOptions] of the FileType.
    pub fn highlighting_options(&self) -> &HighlightingOptions {
        &self.highlighting_options
    }
}

#[cfg(test)]
mod test {
    use crate::FileType;

    #[test]
    fn test_filetypes() {
        assert_eq!(FileType::from("a.rs").name, "Rust");
        assert_eq!(FileType::from("a.java").name, "Java");
        assert_eq!(FileType::from("a.txt").name, "No filetype");
        assert_eq!(FileType::from("foo").name, "No filetype");
    }
}
