use crate::highlighting;
use crate::HighlightingOptions;
use crate::SearchDirection;
use std::vec;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;

/// Represents a row of text within the document.
#[derive(Default)]
pub struct Row {
    /// Whether the row should be highlighted
    pub is_highlighted: bool,
    /// The content contained in the row
    string: String,
    /// The highlight of each grapheme in the row
    highlighting: Vec<highlighting::Type>,
    /// The length of the row's content
    len: usize,
    /// A list of tuples (start, len) of selections made in the row
    selections: Vec<[usize; 2]>,
}

impl Row {
    /// Replaces all tabs in the row with spaces.
    /// 
    /// # Arguments
    /// 
    /// * `spaces_per_tab` - the number of spaces each tab is replaced iwth
    pub fn replace_tabs_with_spaces(&mut self, spaces_per_tab: usize) {
        self.string = self
            .string
            .replace("\t", " ".repeat(spaces_per_tab).as_str());
    }

    /// Renders the row, both the string content of the row and any highlighting.
    /// 
    /// # Arguments
    /// 
    /// * `start` - the index to start rendering from
    /// * `end` - the index to stop rendering at
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = end.min(self.string.len());
        let start = start.min(end);
        let mut result = String::new();
        let mut current_highlighting = &highlighting::Type::Start;

        self.string[..]
            .graphemes(true)
            .enumerate()
            .skip(start)
            .take(end - start)
            .for_each(|(index, grapheme)| {
                if let Some(c) = grapheme.chars().next() {
                    let highlighting_type = self
                        .highlighting
                        .get(index)
                        .unwrap_or(&highlighting::Type::None);

                    if highlighting_type != current_highlighting {
                        current_highlighting = highlighting_type;
                        let start_highlight =
                            format!("{}", termion::color::Fg(highlighting_type.to_color()));
                        result.push_str(&start_highlight[..]);
                    }

                    if c == '\t' {
                        result.push_str("  ");
                    } else {
                        result.push(c);
                    }
                }
            });

        let end_highlight = format!("{}", termion::color::Fg(color::Reset));
        result.push_str(&end_highlight[..]);
        result
    }

    /// Inserts a character at the given position in the row.
    /// 
    /// # Arguments
    /// 
    /// * `at` - the position to insert the character at
    /// * `c` - the character to insert
    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
            self.len += 1;
            return;
        }

        let mut result: String = String::new();
        let mut length = 0;

        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            length += 1;
            if index == at {
                length += 1;
                result.push(c);
            }
            result.push_str(grapheme);
        }

        self.string = result;
        self.len = length;
    }

    /// Appends another row to the current row.
    /// 
    /// # Arguments
    /// 
    /// * `other` - the row to append to this row
    pub fn append(&mut self, other: &Self) {
        self.string = format!("{}{}", self.string, other.string);
        self.len += other.len;
    }

    /// Deletes the character at the given position in the row.
    /// 
    /// # Arguments
    /// 
    /// * `at` - the position of the character in the row to delete
    pub fn delete(&mut self, at: usize) {
        if at < self.len() {
            let mut result: String = String::new();
            let mut length = 0;

            for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
                if index != at {
                    length += 1;
                    result.push_str(grapheme);
                }
            }

            self.string = result;
            self.len = length;
        }
    }

    /// Splits the row into two halves. Afterward, the current row contains the first half while 
    /// the returned row contains the second half.
    /// 
    /// # Arguments
    /// 
    /// * `at` - the index in the row to split at
    pub fn split(&mut self, at: usize) -> Self {
        let mut row: String = String::new();
        let mut length = 0;
        let mut splitted_row: String = String::new();
        let mut splitted_length = 0;

        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index < at {
                length += 1;
                row.push_str(grapheme);
            } else {
                splitted_length += 1;
                splitted_row.push_str(grapheme);
            }
        }

        self.string = row;
        self.len = length;
        self.is_highlighted = false;
        Self {
            is_highlighted: false,
            string: splitted_row,
            highlighting: Vec::new(),
            len: splitted_length,
            selections: Vec::new(),
        }
    }

    /// Finds the index of a string within the row.
    /// 
    /// # Arguments
    /// 
    /// * `query` - the string to find
    /// * `at` - the index to start finding from
    /// * `direction` - the [SearchDirection] to use
    pub fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        if at > self.len || query.is_empty() {
            return None;
        }

        let start = if direction == SearchDirection::Forward {
            at
        } else {
            0
        };

        let end = if direction == SearchDirection::Forward {
            self.len
        } else {
            at
        };

        let substring: String = self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect();

        let matching_byte_index = if direction == SearchDirection::Forward {
            substring.find(query)
        } else {
            substring.rfind(query)
        };

        if let Some(matching_byte_index) = matching_byte_index {
            for (grapheme_index, (byte_index, _)) in
                substring[..].grapheme_indices(true).enumerate()
            {
                if matching_byte_index == byte_index {
                    return Some(start + grapheme_index);
                }
            }
        }
        None
    }

    /// Finds the index of the next word in the row.
    /// 
    /// # Arguments
    /// 
    /// * `at` - the index to start finding from
    /// * `direction` - the [SearchDirection] to use
    pub fn find_next_word(&self, at: usize, direction: SearchDirection) -> Option<usize> {
        if direction == SearchDirection::Forward {
            self.find_word_forward(at)
        } else {
            self.find_word_backward(at)
        }
    }

    /// Finds the index of the next word in the row in the forward direction.
    /// 
    /// The index of the word, if found, will be the index immediately preceding
    /// the start of that word.
    /// 
    /// # Arguments
    /// 
    /// * `start` - the starting index of the row
    /// 
    /// # Example
    /// 
    /// ```
    /// let row = Row::from("Foo Bar");
    /// assert_eq!(row.find_word_forward(1), Some(4));
    /// ```
    fn find_word_forward(&self, start: usize) -> Option<usize> {
        if start >= self.len() {
            return None;
        }

        let substring: String = self.string[..]
            .graphemes(true)
            .skip(start)
            .collect();

        let mut x_skip = 0;
        if substring.chars().nth(0).unwrap().is_alphanumeric() {
            // If the cursor is currently on a word, we need to find the next separator
            // character before we can find the next word.
            if let Some(sep_idx) = substring.find(is_separator) {
                x_skip = sep_idx;
            } else {
                return None;
            }
        }

        // Look for the next alphanumeric character, which is the start of the next word.
        if let Some(x) = substring[x_skip..].find(|c: char| c.is_alphanumeric()) {
            Some(x.saturating_add(start).saturating_add(x_skip))
        } else {
            None
        }
    }

    /// Finds the index of the next word in the row in the backward direction.
    /// 
    /// The index of the word, if found, will be the index immediately following
    /// the end of that word.
    /// 
    /// # Arguments
    /// 
    /// * `end` - the ending index of the row
    /// 
    /// # Example
    /// 
    /// ```
    /// let row = Row::from("Foo Bar");
    /// assert_eq!(row.find_word_backward(5), Some(3));
    /// ```
    fn find_word_backward(&self, mut end: usize) -> Option<usize> {
        if end == 0 {
            return None;
        }

        let substring: String = self.string[..]
            .graphemes(true)
            .take(end)
            .collect();

        if substring.chars().nth_back(0).unwrap().is_alphanumeric() {
            // If the cursor is currently on a word, we need to find the next separator
            // character before we can find the next word.
            if let Some(sep_idx) = substring.rfind(is_separator) {
                end = sep_idx;
            } else {
                return Some(0);
            }
        }

        // Look for the next alphanumeric character, which is the start of the next word.
        if let Some(x) = substring[..end].rfind(|c: char| c.is_alphanumeric()) {
            Some(x.saturating_add(1))
        } else {
            Some(0)
        }
    }

    /// Replaces all selections made in the row.
    /// 
    /// # Arguments
    /// 
    /// * `replace` - the string to replace the selections with
    pub fn replace_selections(&mut self, word: &Option<String>) {
        if self.selections.len() > 0 {
            // See https://stackoverflow.com/a/64921799
            let mut selections = std::mem::take(&mut self.selections);

            selections.sort_by(|a, b| a[0].cmp(&b[0]));
            let mut merged_selections = vec![selections[0].clone()];
            let mut prev = &mut merged_selections[0];

            for curr in selections[1..].iter_mut() {
                if curr[0] >= prev[0] && curr[0] < prev[1] {
                    prev[1] = curr[1].max(prev[1]);
                } else {
                    merged_selections.push(*curr);
                    prev = curr;
                }
            }

            merged_selections.iter().rev().for_each(|[at, end]| {
                (*at..*end).for_each(|_| {
                    self.delete(*at);
                });

                if let Some(word) = word {
                    word.chars().rev().for_each(|c| {
                        self.insert(*at, c);
                    });
                }
            });

            self.selections = selections;
            self.is_highlighted = false;
            self.reset_selections();
        }
    }

    /// Adds a selection in this row.
    /// 
    /// # Arguments
    /// 
    /// * `at` - the index of the selection in this row
    /// * `len` - the length of the selection
    pub fn add_selection(&mut self, at: usize, len: usize) {
        self.selections
            .push([at, at.saturating_add(len).min(self.string.len())]);
    }

    /// Resets all selections made in the row.
    pub fn reset_selections(&mut self) {
        self.selections.clear();
    }

    /// Checks whether there is a string match in the row to highlight.
    /// 
    /// # Arguments
    /// 
    /// * `index` - the index to check from; this gets updated to the end of the highlight
    /// * `substring` - the string to highlight
    /// * `chars` - the characters in the row
    /// * `hl_type` - the specific highlighting type to use
    pub fn highlight_str(
        &mut self,
        index: &mut usize,
        substring: &str,
        chars: &[char],
        hl_type: highlighting::Type,
    ) -> bool {
        if substring.is_empty() {
            return false;
        }

        for (substring_index, c) in substring.chars().enumerate() {
            if let Some(next_char) = chars.get(index.saturating_add(substring_index)) {
                if *next_char != c {
                    return false;
                }
            } else {
                return false;
            }
        }

        for _ in 0..substring.len() {
            self.highlighting.push(hl_type);
            *index += 1;
        }

        true
    }

    /// Checks whether there is a keyword to be highlighted.
    /// 
    /// # Arguments
    /// 
    /// * `index` - the index to check from; this gets updated to the end of the highlight
    /// * `chars` - the characters in the row
    /// * `keywords` - the keywords that should be highlighted
    /// * `hl_type` - the specific highlighting type to use
    pub fn highlight_keywords(
        &mut self,
        index: &mut usize,
        chars: &[char],
        keywords: &[String],
        hl_type: highlighting::Type,
    ) -> bool {
        if *index > 0 {
            let prev_char = chars[*index - 1];
            if !is_separator(prev_char) {
                return false;
            }
        }

        for word in keywords {
            if *index < chars.len().saturating_sub(word.len()) {
                let next_char = chars[*index + word.len()];
                if !is_separator(next_char) {
                    continue;
                }
            }

            if self.highlight_str(index, &word, chars, hl_type) {
                return true;
            }
        }

        return false;
    }

    /// Checks whether there is a primary keyword to be highlighted.
    /// 
    /// # Arguments
    /// 
    /// * `index` - the index to check from; this gets updated to the end of the highlight
    /// * `opts` - the [HighlightingOptions] to use
    /// * `chars` - the characters in the row
    fn highlight_primary_keywords(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        chars: &[char],
    ) -> bool {
        self.highlight_keywords(
            index,
            chars,
            opts.primary_keywords(),
            highlighting::Type::PrimaryKeywords,
        )
    }

    /// Checks whether there is a secondary keyword to be highlighted.
    /// 
    /// # Arguments
    /// 
    /// * `index` - the index to check from; this gets updated to the end of the highlight
    /// * `opts` - the [HighlightingOptions] to use
    /// * `chars` - the characters in the row
    fn highlight_secondary_keywords(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        chars: &[char],
    ) -> bool {
        self.highlight_keywords(
            index,
            chars,
            opts.secondary_keywords(),
            highlighting::Type::SecondaryKeywords,
        )
    }

    /// Highlights any matches in the row.
    /// 
    /// # Arguments
    /// 
    /// * `word` - the word to highlight
    fn highlight_match(&mut self, word: &Option<String>) {
        if let Some(word) = word {
            if word.is_empty() {
                return;
            }

            let mut index = 0;
            while let Some(search_match) = self.find(word, index, SearchDirection::Forward) {
                if let Some(next_index) = search_match.checked_add(word[..].graphemes(true).count())
                {
                    for i in search_match..next_index {
                        self.highlighting[i] = highlighting::Type::Match;
                    }
                    index = next_index;
                } else {
                    break;
                }
            }
        }
    }

    /// Highlights any selections in the row.
    fn highlight_selection(&mut self) {
        for [at, end] in self.selections.iter() {
            for i in *at..*end {
                self.highlighting[i] = highlighting::Type::Selection;
            }
        }
    }

    /// Checks whether there is a character literal to be highlighted.
    /// 
    /// # Arguments
    /// 
    /// * `index` - the index to check from; this gets updated to the end of the highlight
    /// * `opts` - the [HighlightingOptions] to use
    /// * `c` - the character at `chars[index]`
    /// * `chars` - the characters in the row
    pub fn highlight_char(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.characters() && c == '\'' {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                let closing_index = if *next_char == '\\' {
                    index.saturating_add(3)
                } else {
                    index.saturating_add(2)
                };
                if let Some(closing_char) = chars.get(closing_index) {
                    if *closing_char == '\'' {
                        for _ in 0..=closing_index.saturating_sub(*index) {
                            self.highlighting.push(highlighting::Type::Character);
                            *index += 1;
                        }
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Checks whether there is an inline comment to be highlighted.
    /// 
    /// # Arguments
    /// 
    /// * `index` - the index to check from; this gets updated to the end of the highlight
    /// * `opts` - the [HighlightingOptions] to use
    /// * `_` - the character at `chars[index]`
    /// * `chars` - the characters in the row
    pub fn highlight_comment(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        _: char,
        chars: &[char],
    ) -> bool {
        if let Some(comment_delim) = opts.comments() {
            for (i, d) in comment_delim.chars().enumerate() {
                if *index + i >= chars.len() || chars[*index + i] != d {
                    return false;
                }
            }

            for _ in *index..chars.len() {
                self.highlighting.push(highlighting::Type::Comment);
                *index += 1;
            }
            return true;
        }

        false
    }

    /// Checks whether there is a multiline comment to be highlighted.
    /// 
    /// # Arguments
    /// 
    /// * `index` - the index to check from; this gets updated to the end of the highlight
    /// * `opts` - the [HighlightingOptions] to use
    /// * `c` - the character at `chars[index]`
    /// * `chars` - the characters in the row
    pub fn highlight_multiline_comment(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.multiline_comments() && c == '/' && *index < chars.len() {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                if *next_char == '*' {
                    let closing_index =
                        if let Some(closing_index) = self.string[*index + 2..].find("*/") {
                            *index + closing_index + 4
                        } else {
                            chars.len()
                        };
                    for _ in *index..closing_index {
                        self.highlighting.push(highlighting::Type::MultilineComment);
                        *index += 1;
                    }

                    return true;
                }
            }
        }

        false
    }

    /// Checks whether there is a string literal to be highlighted.
    /// 
    /// # Arguments
    /// 
    /// * `index` - the index to check from; this gets updated to the end of the highlight
    /// * `opts` - the [HighlightingOptions] to use
    /// * `c` - the character at `chars[index]`
    /// * `chars` - the characters in the row
    pub fn highlight_string(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.strings() && c == '"' {
            let mut prev_char = c;
            loop {
                self.highlighting.push(highlighting::Type::String);
                *index += 1;
                if let Some(next_char) = chars.get(*index) {
                    if prev_char != '\\' && *next_char == '"' {
                        break;
                    }
                    prev_char = *next_char;
                } else {
                    break;
                }
            }

            self.highlighting.push(highlighting::Type::String);
            *index += 1;
            true
        } else {
            false
        }
    }

    /// Checks whether there is a number literal to be highlighted.
    /// 
    /// # Arguments
    /// 
    /// * `index` - the index to check from; this gets updated to the end of the highlight
    /// * `opts` - the [HighlightingOptions] to use
    /// * `c` - the character at `chars[index]`
    /// * `chars` - the characters in the row
    pub fn highlight_number(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.numbers() && c.is_ascii_digit() {
            if *index > 0 {
                let prev_char = chars[*index - 1];
                if !is_separator(prev_char) {
                    return false;
                }
            }

            loop {
                self.highlighting.push(highlighting::Type::Number);
                *index += 1;
                if let Some(next_char) = chars.get(*index) {
                    if *next_char != '.' && !next_char.is_ascii_digit() {
                        break;
                    }
                } else {
                    break;
                }
            }

            true
        } else {
            false
        }
    }

    /// Computes the highlighting (if any) of every grapheme in this row. This function returns
    /// whether a multi-line comment continues into the next line.
    /// 
    /// # Arguments
    /// 
    /// * `opts` - the `HighlightingOptions` to use
    /// * `word` - the word to highlight (if any)
    /// * `start_with_comment` - whether the start of the row is part of a multi-line comment
    pub fn highlight(
        &mut self,
        opts: &HighlightingOptions,
        word: &Option<String>,
        start_with_comment: bool,
    ) -> bool {
        let chars: Vec<char> = self.string.chars().collect();
        if self.is_highlighted && word.is_none() {
            return false;
        }

        self.highlighting = Vec::new();
        let mut index = 0;
        let mut in_multiline_comment = start_with_comment;

        if in_multiline_comment {
            let closing_index = if let Some(closing_index) = self.string.find("*/") {
                closing_index + 2
            } else {
                chars.len()
            };
            for _ in 0..closing_index {
                self.highlighting.push(highlighting::Type::MultilineComment);
            }
            index = closing_index;
        }

        while let Some(c) = chars.get(index) {
            if self.highlight_multiline_comment(&mut index, opts, *c, &chars) {
                in_multiline_comment = true;
                continue;
            }
            in_multiline_comment = false;
            if self.highlight_char(&mut index, opts, *c, &chars)
                || self.highlight_comment(&mut index, opts, *c, &chars)
                || self.highlight_primary_keywords(&mut index, opts, &chars)
                || self.highlight_secondary_keywords(&mut index, opts, &chars)
                || self.highlight_string(&mut index, opts, *c, &chars)
                || self.highlight_number(&mut index, opts, *c, &chars)
            {
                continue;
            }

            self.highlighting.push(highlighting::Type::None);
            index += 1;
        }

        self.highlight_match(word);
        self.highlight_selection();

        if in_multiline_comment && &self.string[self.string.len().saturating_sub(2)..] != "*/" {
            return true;
        }

        self.is_highlighted = true;
        false
    }

    /// Gets the length of the row.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Gets whether this row is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Gets the row's contents as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    /// Gets the number of leading spaces in the row.
    pub fn get_leading_spaces(&self) -> Option<usize> {
        let mut index = 0;
        let chars: Vec<char> = self.string.chars().collect();

        while index < self.string.len() && chars[index] == ' ' {
            index += 1;
        }

        return if index == self.string.len() || index == 0 {
            None
        } else {
            Some(index)
        };
    }
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            is_highlighted: false,
            string: String::from(slice),
            highlighting: Vec::new(),
            len: slice.graphemes(true).count(),
            selections: Vec::new(),
        }
    }
}

fn is_separator(c: char) -> bool {
    c.is_ascii_punctuation() || c.is_ascii_whitespace()
}

#[cfg(test)]
mod test {
    use crate::highlighting::Type;
    use crate::row::Row;
    use crate::{SearchDirection, FileType};

    #[test]
    fn basics() {
        let mut row = Row::from("Hello, World!");
        assert_eq!(row.len(), "Hello, World!".len());
        assert_eq!(row.find("ello", 0, SearchDirection::Forward), Some(1));
        assert_eq!(row.find("ello", 2, SearchDirection::Forward), None);
        assert_eq!(row.find("ello", 8, SearchDirection::Backward), Some(1));

        row = Row::from("    x = 3;");
        assert_eq!(row.get_leading_spaces(), Some(4));

        row = Row::from("\tx = 3;");
        assert_eq!(row.get_leading_spaces(), None);
    }

    #[test]
    fn edit() {
        let mut row1 = Row::from("Hello ");

        row1.append(&Row::from("World!"));
        assert_eq!(row1.string, "Hello World!");

        row1.insert(0, '!');
        row1.insert(6, ',');
        row1.delete(0);
        assert_eq!(row1.string, "Hello, World!");

        let row2 = row1.split(7);
        assert_eq!(row1.string, "Hello, ");
        assert_eq!(row2.string, "World!");

        let row3 = row1.split(7);
        assert_eq!(row1.len(), 7);
        assert_eq!(row3.len(), 0);

        let row4 = row1.split(0);
        assert_eq!(row1.len(), 0);
        assert_eq!(row4.len(), 7);
    }

    #[test]
    fn select_and_edit() {
        let mut row = Row::from("Hello, World!");

        row.add_selection(2, 1);
        row.add_selection(3, 1);
        row.add_selection(10, 1);
        row.replace_selections(&Some("1".to_string()));
        assert_eq!(row.string, "He11o, Wor1d!");

        row = Row::from("var += pq * xy;");
        row.add_selection(0, 3);
        row.add_selection(7, 2);
        row.add_selection(12, 2);
        row.replace_selections(&Some("foo".to_string()));
        assert_eq!(row.string, "foo += foo * foo;");

        row = Row::from("humuhumuhuma nukunukunukunuku apua");
        row.add_selection(8, 4);
        row.add_selection(13, 8);
        row.replace_selections(&None);
        assert_eq!(row.string, "humuhumu nukunuku apua");
    }

    #[test]
    fn find_next_word() {
        let mut row = Row::from("Foo Bar");
        assert_eq!(row.find_next_word(1, SearchDirection::Forward), Some(4));
        assert_eq!(row.find_next_word(4, SearchDirection::Forward), None);
        assert_eq!(row.find_next_word(5, SearchDirection::Backward), Some(3));
        assert_eq!(row.find_next_word(1, SearchDirection::Backward), Some(0));
        assert_eq!(row.find_next_word(0, SearchDirection::Backward), None);
        
        row = Row::from("my___new_constant");
        assert_eq!(row.find_next_word(0, SearchDirection::Forward), Some(5));
        assert_eq!(row.find_next_word(8, SearchDirection::Backward), Some(2));
        
        row = Row::from("");
        assert_eq!(row.find_next_word(0, SearchDirection::Forward), None);
        assert_eq!(row.find_next_word(0, SearchDirection::Backward), None);
    }

    #[test]
    fn highlight() {
        // TODO: flesh out highlighting unit tests
        let filetype = FileType::from("foo.rs");

        let mut row = Row::from("let foo=3;");
        assert!(!row.highlight(&filetype.highlighting_options(), &None, false));
        let mut base = vec![
            Type::PrimaryKeywords, Type::PrimaryKeywords, Type::PrimaryKeywords,    // let
            Type::None,
            Type::None, Type::None, Type::None,                                     // foo
            Type::None,                                                             // =
            Type::Number,                                                           // 3
            Type::None                                                              // ;
        ];
        assert!(row.highlighting.eq(&base));

        row = Row::from("\"a3\"/*3");
        assert!(row.highlight(&filetype.highlighting_options(), &None, false));
        base = vec![
            Type::String, Type::String, Type::String, Type::String,                     // "a3"
            Type::MultilineComment, Type::MultilineComment, Type::MultilineComment      // /*3
        ];
        assert!(row.highlighting.eq(&base));
    }
}
