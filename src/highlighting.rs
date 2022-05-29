use termion::color;

#[derive(PartialEq, Clone, Copy)]
pub enum Type {
    None,
    Start,
    Number,
    Match,
    String,
    Character,
    Comment,
    MultilineComment,
    PrimaryKeywords,
    SecondaryKeywords,
}

/*
 * 216-color chart: https://www.web-source.net/216_color_chart.htm
 */
impl Type {
    pub fn to_color(&self) -> color::AnsiValue {
        match self {
            Type::Number => color::AnsiValue::rgb(5, 1, 5),
            Type::Match => color::AnsiValue::rgb(0, 5, 0),
            Type::String => color::AnsiValue::rgb(5, 2, 2),
            Type::Character => color::AnsiValue::rgb(5, 4, 0),
            Type::Comment | Type::MultilineComment => color::AnsiValue::rgb(3, 3, 3),
            Type::PrimaryKeywords => color::AnsiValue::rgb(0, 4, 5),
            Type::SecondaryKeywords => color::AnsiValue::rgb(0, 5, 4),
            _ => color::AnsiValue::rgb(5, 5, 5),
        }
    }
}
