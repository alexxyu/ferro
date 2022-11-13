use lazy_static::lazy_static;
use termbg::{self, Theme};
use termion::color;

lazy_static! {
    static ref SHOULD_USE_DARK_THEME: bool = termbg::theme(std::time::Duration::from_millis(100))
        .map_or(true, |res| {
            match res {
                Theme::Dark => true,
                _ => false,
            }
        });
}

/// The different types of highlighting.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Type {
    None,
    Start,
    Number,
    Match,
    Selection,
    String,
    Character,
    Comment,
    MultilineComment,
    PrimaryKeywords,
    SecondaryKeywords,
}

impl Type {
    /// Gets the ANSI value representation of a highlighting type to be used for highlight rendering.
    ///
    /// For more information, a 216-color chart that was used for reference can be found here:
    /// <https://www.web-source.net/216_color_chart.htm>
    pub fn to_color(&self) -> color::AnsiValue {
        match self {
            Type::Number => color::AnsiValue::rgb(5, 1, 5),
            Type::Match => color::AnsiValue::rgb(0, 5, 0),
            Type::Selection => color::AnsiValue::rgb(2, 2, 5),
            Type::String => color::AnsiValue::rgb(5, 2, 2),
            Type::Character => color::AnsiValue::rgb(5, 4, 0),
            Type::Comment | Type::MultilineComment => color::AnsiValue::rgb(3, 3, 3),
            Type::PrimaryKeywords => color::AnsiValue::rgb(0, 4, 5),
            Type::SecondaryKeywords => color::AnsiValue::rgb(0, 5, 4),
            _ => {
                if *SHOULD_USE_DARK_THEME {
                    color::AnsiValue::rgb(5, 5, 5)
                } else {
                    color::AnsiValue::rgb(0, 0, 0)
                }
            }
        }
    }
}
