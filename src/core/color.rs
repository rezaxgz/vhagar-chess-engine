use std::ops::Not;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Color {
    White = 0,
    Black = 1,
}
impl Not for Color {
    type Output = Color;
    fn not(self) -> Self::Output {
        if self == Color::White {
            return Color::Black;
        }
        return Color::White;
    }
}
