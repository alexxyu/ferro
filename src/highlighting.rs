use termion::color;

#[derive(PartialEq)]
pub enum Type {
    None,
    Number,
}

impl Type {
    pub fn to_color(&self) -> &dyn color::Color {
        match self {
            Type::Number => &color::LightBlue,
            _ => &color::White,
        }
    }
}
