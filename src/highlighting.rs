use termion::color;

#[derive(PartialEq)]
pub enum Type {
    None,
    Number,
    Match,
}

impl Type {
    pub fn to_color(&self) -> &dyn color::Color {
        match self {
            Type::Number => &color::LightBlue,
            Type::Match => &color::Green,
            _ => &color::White,
        }
    }
}
