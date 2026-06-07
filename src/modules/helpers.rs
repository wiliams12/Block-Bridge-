use crate::modules::general::*;
use rand::Rng;

pub fn get_random_shape() -> ShapeType {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..7) {
        0 => ShapeType::LShape,
        1 => ShapeType::JShape,
        2 => ShapeType::OShape,
        3 => ShapeType::FourLine,
        4 => ShapeType::TShape,
        5 => ShapeType::SShape,
        _ => ShapeType::ZShape,
    }
}
