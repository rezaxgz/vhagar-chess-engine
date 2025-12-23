const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
const ROWS: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

const DISTANCE_FROM_CENTER_FILE: [u8; 8] = [3, 2, 1, 0, 0, 1, 2, 3];
pub trait SquareUtil<T> {
    fn get_file(&self) -> u8;
    fn get_rank(&self) -> u8;
    fn to_str(&self) -> String;
    #[allow(unused)]
    fn distance_from_center_file(&self) -> u8;
}
pub type Square = u8;
impl SquareUtil<Square> for Square {
    fn get_file(&self) -> u8 {
        return self & 7;
    }
    fn get_rank(&self) -> u8 {
        return self >> 3;
    }
    fn to_str(&self) -> String {
        return format!("{}{}", FILES[self.get_file() as usize], self.get_rank() + 1);
    }
    fn distance_from_center_file(&self) -> u8 {
        return DISTANCE_FROM_CENTER_FILE[self.get_file() as usize];
    }
}
pub fn str_to_square(str: &str) -> Square {
    let chars = str.chars().collect::<Vec<char>>();
    for i in 0..8 {
        if chars[0] == ROWS[i] {
            return (i as u32 + 8 * (chars[1].to_digit(10).unwrap() - 1)) as Square;
        }
    }
    return 0;
}
