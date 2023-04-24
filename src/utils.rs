use crate::cell::Cell;
use crate::error::Error;
use crate::grid_map::GridMap;
use crate::position::Position;

use image::io::Reader;
use std::path::Path;
use yaml_rust::YamlLoader;

pub fn load_ros_yaml<P: AsRef<Path>>(yaml_path: P) -> Result<GridMap<u8>, Error> {
    let yaml_str = std::fs::read_to_string(yaml_path)?;
    let docs = YamlLoader::load_from_str(&yaml_str)?;
    let doc = &docs[0];
    let pgm_path = doc["image"]
        .as_str()
        .ok_or(Error::Other("Failed to load yaml: [image]".to_string()))?;
    let origin = doc["origin"]
        .as_vec()
        .ok_or(Error::Other("Failed to load yaml: [origin]".to_string()))?;
    let origin_x = origin[0].as_f64().ok_or(Error::Other(
        "Failed to load yaml: parse origin x".to_string(),
    ))?;
    let origin_y = origin[1].as_f64().ok_or(Error::Other(
        "Failed to load yaml: parse origin y".to_string(),
    ))?;
    let resolution = doc["resolution"].as_f64().ok_or(Error::Other(
        "Failed to load yaml: [resolution]".to_string(),
    ))?;
    load_pgm(
        pgm_path,
        Position {
            x: origin_x,
            y: origin_y,
        },
        resolution,
    )
}

pub fn load_pgm<P: AsRef<Path>>(
    path: P,
    origin: Position,
    resolution: f64,
) -> Result<GridMap<u8>, Error> {
    let img = Reader::open(path)?.decode()?;
    let gray_image = img
        .as_luma8()
        .ok_or(Error::Other("Failed to convert to luma8".to_string()))?;
    let w = gray_image.width() as f64;
    let h = gray_image.height() as f64;
    let max_point = Position::new(origin.x + w * resolution, origin.y + h * resolution);
    let mut map = GridMap::new(origin, max_point, resolution);
    *map.cells_mut() = gray_image
        .as_raw()
        .iter()
        .map(|d| Cell::from_value(*d))
        .collect::<Vec<_>>();
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_file() {
        let map = load_pgm(
            "test/map.pgm",
            Position {
                x: -2.485,
                y: -2.174,
            },
            0.05,
        )
        .unwrap();
        assert_eq!(map.width(), 194);
        assert_eq!(map.height(), 170);
        let mut num_clear = 0;
        let mut num_wall = 0;
        let mut num_unknown = 0;
        for cell in map.cells() {
            assert!(cell.value().is_some());
            let v = cell.value().unwrap();
            match v {
                0 => num_wall += 1,
                205 => num_unknown += 1,
                254 => num_clear += 1,
                _ => {}
            }
        }
        assert_eq!(num_clear, 15372);
        assert_eq!(num_wall, 1335);
        assert_eq!(num_unknown, 16273);
    }

    #[test]
    fn load_ros_file() {
        let map = load_ros_yaml("test/map.yaml").unwrap();
        assert_eq!(map.width(), 194);
        assert_eq!(map.height(), 170);
        for cell in map.cells() {
            assert!(cell.value().is_some());
        }
    }
}
