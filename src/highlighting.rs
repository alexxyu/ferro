use termion::color;

#[derive(PartialEq)]
pub enum Type {
    None,
    Number,
    Match,
    String,
    Character,
    Comment,
}

impl Type {
    pub fn to_color(&self) -> &dyn color::Color {
        match self {
            Type::Number => &color::LightBlue,
            Type::Match => &color::Green,
            Type::String => &color::LightRed,
            Type::Character => &color::Yellow,
            Type::Comment => &color::LightBlack,
            _ => &color::White,
        }
    }
}
