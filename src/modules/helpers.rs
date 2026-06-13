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

pub fn path_to_material(material: &TileType) -> &'static str {
    match material {
        TileType::Rock => "textures/rock.png",
        TileType::Dirt => "textures/dirt.png",
        TileType::Bricks => "textures/bricks.png",
        TileType::Concrete => "textures/concrete.png",
        TileType::PlayerBlock => {
            let mut rng = rand::thread_rng();

            match rng.gen_range(0..=1) {
                0 => "textures/player_block0.png",
                _ => "textures/player_block1.png",
            }
        }
        TileType::DirtTop => "textures/dirt-top.png",
    }
}
