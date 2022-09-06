use std::collections::HashMap;

/// The file type of a document.
pub struct FileType {
    /// The type associated with this [FileType] (e.g. "Rust" for ".rs" files)
    name: String,
    /// The associated [HighlightingOptions] for this FileType
    hl_opts: HighlightingOptions,
}

/// The highlighting options that determine what gets highlighted in a file.
#[derive(Default)]
pub struct HighlightingOptions {
    numbers: bool,
    strings: bool,
    characters: bool,
    comments: Option<String>,
    multiline_comments: Option<HashMap<String, String>>,
    primary_keywords: Vec<String>,
    secondary_keywords: Vec<String>,
}

impl HighlightingOptions {
    pub fn numbers(&self) -> bool {
        self.numbers
    }
    pub fn strings(&self) -> bool {
        self.strings
    }
    pub fn characters(&self) -> bool {
        self.characters
    }
    pub fn comments(&self) -> &Option<String> {
        &self.comments
    }
    pub fn multiline_comments(&self) -> &Option<HashMap<String, String>> {
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
            hl_opts: HighlightingOptions::default(),
        }
    }
}

impl FileType {
    /// Constructs the FileType based on a given filename
    /// 
    /// # Arguments
    /// 
    /// * `file_name` - the name of the file
    pub fn from(file_name: &str) -> Self {
        if file_name.ends_with(".rs") {
            return Self {
                name: String::from("Rust"),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: Some("//".to_string()),
                    multiline_comments: Some(HashMap::from([
                        ("/*".to_string(), "*/".to_string()),
                    ])),
                    primary_keywords: vec![
                        "as".to_string(),
                        "break".to_string(),
                        "const".to_string(),
                        "continue".to_string(),
                        "crate".to_string(),
                        "else".to_string(),
                        "enum".to_string(),
                        "extern".to_string(),
                        "false".to_string(),
                        "fn".to_string(),
                        "for".to_string(),
                        "if".to_string(),
                        "impl".to_string(),
                        "in".to_string(),
                        "let".to_string(),
                        "loop".to_string(),
                        "match".to_string(),
                        "mod".to_string(),
                        "move".to_string(),
                        "mut".to_string(),
                        "pub".to_string(),
                        "ref".to_string(),
                        "return".to_string(),
                        "self".to_string(),
                        "Self".to_string(),
                        "static".to_string(),
                        "struct".to_string(),
                        "super".to_string(),
                        "trait".to_string(),
                        "true".to_string(),
                        "type".to_string(),
                        "unsafe".to_string(),
                        "use".to_string(),
                        "where".to_string(),
                        "while".to_string(),
                        "dyn".to_string(),
                        "abstract".to_string(),
                        "become".to_string(),
                        "box".to_string(),
                        "do".to_string(),
                        "final".to_string(),
                        "macro".to_string(),
                        "override".to_string(),
                        "priv".to_string(),
                        "typeof".to_string(),
                        "unsized".to_string(),
                        "virtual".to_string(),
                        "yield".to_string(),
                        "async".to_string(),
                        "await".to_string(),
                        "try".to_string(),
                    ],
                    secondary_keywords: vec![
                        "bool".to_string(),
                        "char".to_string(),
                        "i8".to_string(),
                        "i16".to_string(),
                        "i32".to_string(),
                        "i64".to_string(),
                        "isize".to_string(),
                        "u8".to_string(),
                        "u16".to_string(),
                        "u32".to_string(),
                        "u64".to_string(),
                        "usize".to_string(),
                        "f32".to_string(),
                        "f64".to_string(),
                    ],
                },
            };
        } else if file_name.ends_with(".java") {
            return Self {
                name: String::from("Java"),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: Some("//".to_string()),
                    multiline_comments: Some(HashMap::from([
                        ("/*".to_string(), "*/".to_string()),
                    ])),
                    primary_keywords: vec![
                        "abstract".to_string(),
                        "assert".to_string(),
                        "break".to_string(),
                        "case".to_string(),
                        "catch".to_string(),
                        "class".to_string(),
                        "const".to_string(),
                        "continue".to_string(),
                        "default".to_string(),
                        "do".to_string(),
                        "else".to_string(),
                        "enum".to_string(),
                        "extends".to_string(),
                        "false".to_string(),
                        "final".to_string(),
                        "finally".to_string(),
                        "for".to_string(),
                        "if".to_string(),
                        "implements".to_string(),
                        "import".to_string(),
                        "instanceof".to_string(),
                        "interface".to_string(),
                        "native".to_string(),
                        "new".to_string(),
                        "null".to_string(),
                        "package".to_string(),
                        "private".to_string(),
                        "protected".to_string(),
                        "public".to_string(),
                        "return".to_string(),
                        "static".to_string(),
                        "super".to_string(),
                        "switch".to_string(),
                        "synchronized".to_string(),
                        "this".to_string(),
                        "throw".to_string(),
                        "throws".to_string(),
                        "transient".to_string(),
                        "true".to_string(),
                        "try".to_string(),
                        "void".to_string(),
                        "volatile".to_string(),
                        "while".to_string(),
                    ],
                    secondary_keywords: vec![
                        "boolean".to_string(),
                        "byte".to_string(),
                        "char".to_string(),
                        "double".to_string(),
                        "float".to_string(),
                        "int".to_string(),
                        "long".to_string(),
                        "short".to_string(),
                    ],
                },
            }
        } else if file_name.ends_with(".py") {
            return Self {
                name: String::from("Python"),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: Some("#".to_string()),
                    multiline_comments: Some(HashMap::from([
                        ("'''".to_string(), "'''".to_string()),
                        ("\"\"\"".to_string(), "\"\"\"".to_string()),
                    ])),
                    primary_keywords: vec![
                        "False".to_string(),
                        "None".to_string(),
                        "True".to_string(),
                        "and".to_string(),
                        "as".to_string(),
                        "assert".to_string(),
                        "async".to_string(),
                        "await".to_string(),
                        "break".to_string(),
                        "class".to_string(),
                        "continue".to_string(),
                        "def".to_string(),
                        "del".to_string(),
                        "elif".to_string(),
                        "else".to_string(),
                        "except".to_string(),
                        "finally".to_string(),
                        "for".to_string(),
                        "from".to_string(),
                        "global".to_string(),
                        "if".to_string(),
                        "import".to_string(),
                        "in".to_string(),
                        "is".to_string(),
                        "lambda".to_string(),
                        "nonlocal".to_string(),
                        "not".to_string(),
                        "or".to_string(),
                        "pass".to_string(),
                        "raise".to_string(),
                        "return".to_string(),
                        "try".to_string(),
                        "while".to_string(),
                        "with".to_string(),
                        "yield".to_string(),
                    ],
                    secondary_keywords: vec![],
                },
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
        &self.hl_opts
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
