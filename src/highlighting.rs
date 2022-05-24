use termion::color;

#[derive(PartialEq, Clone, Copy)]
pub enum Type {
    None,
    Number,
    Match,
    String,
    Character,
    Comment,
    PrimaryKeywords,
    SecondaryKeywords,
}

impl Type {
    pub fn to_color(&self) -> &dyn color::Color {
        match self {
            Type::Number => &color::LightMagenta,
            Type::Match => &color::Green,
            Type::String => &color::LightRed,
            Type::Character => &color::Yellow,
            Type::Comment => &color::LightBlack,
            Type::PrimaryKeywords => &color::LightBlue,
            Type::SecondaryKeywords => &color::LightCyan,
            _ => &color::White,
        }
    }
}
