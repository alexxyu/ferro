use crate::HighlightingOptions;
use crate::highlighting;
use crate::SearchDirection;
use std::vec;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    pub is_highlighted: bool,
    string: String,
    highlighting: Vec<highlighting::Type>,
    len: usize,
    selections: Vec<[usize; 2]>,
}

impl Row {
    pub fn replace_tabs_with_spaces(&mut self, spaces_per_tab: usize) {
        self.string = self.string.replace("\t", " ".repeat(spaces_per_tab).as_str());
    }

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

    pub fn append(&mut self, other: &Self) {
        self.string = format!("{}{}", self.string, other.string);
        self.len += other.len;
    }

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

    pub fn replace_selections(&mut self, word: &Option<String>) {
        if self.selections.len() > 0 {
            // See https://stackoverflow.com/a/64921799
            let mut selections = std::mem::take(&mut self.selections);

            selections.sort_by(|a,b| a[0].cmp(&b[0]));
            let mut merged_selections = vec![selections[0].clone()];
            let mut prev = &mut merged_selections[0];

            for curr in selections[1..].iter_mut() {
                if curr[0] >= prev[0] && curr[0] <= prev[1] {
                    prev[1] = curr[1].max(prev[1]);
                } else {
                    merged_selections.push(*curr);
                    prev = curr;
                }
            }

            merged_selections
                .iter()
                .rev()
                .for_each(|[at, end]| {
                    (*at..*end).for_each(|_| { 
                        self.delete(*at);
                    });

                    if let Some(word) = word {
                        word.chars()
                            .rev()
                            .for_each(|c| {
                                self.insert(*at, c);
                            });
                    }
                });

            self.selections = selections;
            self.is_highlighted = false;
            self.reset_selections();
        }
    }

    pub fn add_selection(&mut self, at: usize, len: usize) {
        self.selections.push(
            [at, at.saturating_add(len).min(self.string.len())]
        );
    }

    pub fn reset_selections(&mut self) {
        self.selections.clear();
    }

    pub fn highlight_str(
        &mut self,
        index: &mut usize,
        substring: &str,
        chars: &[char],
        hl_type: highlighting::Type
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

    pub fn highlight_keywords(
        &mut self,
        index: &mut usize,
        chars: &[char],
        keywords: &[String],
        hl_type: highlighting::Type
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

    fn highlight_match(&mut self, word: &Option<String>) {
        if let Some(word) = word {
            if word.is_empty() {
                return;
            }

            let mut index = 0;
            while let Some(search_match) = self.find(word, index, SearchDirection::Forward) {
                if let Some(next_index) = search_match.checked_add(word[..].graphemes(true).count()) {
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

    fn highlight_selection(&mut self) {
        for [at, end] in self.selections.iter() {
            for i in *at..*end {
                self.highlighting[i] = highlighting::Type::Selection;
            }
        }
    }

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

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    pub fn get_leading_spaces(&self) -> Option<usize> {
        let mut index = 0;
        let chars: Vec<char> = self.string.chars().collect();

        while index < self.string.len() && chars[index] == ' ' {
            index += 1;
        }

        return if index == self.string.len() || index == 0 { None } else { Some(index) };
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
