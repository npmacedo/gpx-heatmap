use std::f64::consts::PI;
use std::ffi::OsStr;
use std::fs;

use std::fs::File;
use std::io::BufReader;
use std::io::Error;

use bevy::math::Vec3;
use gpx::read;
use gpx::Gpx;
use reqwest::header::USER_AGENT;

mod viewer;

const OSM_TILE_SERVER: &str = "https://maps.wikimedia.org/osm-intl";
const HEATMAP_MAX_SIZE: (i32, i32) = (2160, 3840);
const OSM_TILE_SIZE: i32 = 256;
const OSM_MAX_ZOOM: i32 = 19;

#[derive(Debug)]
struct Bounding {
    lat_min: f64,
    lat_max: f64,
    lon_min: f64,
    lon_max: f64,
}

#[derive(Debug)]
pub struct MapTiles {
    pub zoom: i32,
    pub x_tile_min: i32,
    pub y_tile_max: i32,
    pub x_tile_max: i32,
    pub y_tile_min: i32,
}

impl MapTiles {
    fn get_nx(&self) -> i32 {
        return self.x_tile_max - self.x_tile_min + 1;
    }

    fn get_ny(&self) -> i32 {
        return self.y_tile_max - self.y_tile_min + 1;
    }
}

pub fn deg_to_xy(lat_deg: f64, lon_deg: f64, zoom: i32) -> (f64, f64) {
    let lat_rad = lat_deg.to_radians();
    let n = 2.0f64.powi(zoom);
    let x = (lon_deg + 180.0) / 360.0 * n;
    let y = (1.0 - lat_rad.tan().asinh() / PI) / 2.0 * n;
    (x, y)
}

fn read_gpx_files() -> Result<Vec<Gpx>, Error> {
    let paths = fs::read_dir("./gpx").unwrap();

    let mut activities: Vec<Gpx> = vec![];

    for path in paths {
        let path = path?.path();
        if path.extension().and_then(OsStr::to_str) == Some("gpx") {
            let file = File::open(path.into_os_string().into_string().unwrap()).unwrap();
            let reader = BufReader::new(file);
            let gpx: Gpx = read(reader).unwrap();
            activities.push(gpx);
        }        
    }

    Ok(activities)
}

fn get_lat_lon(files: &Vec<Gpx>) -> Vec<(f64, f64)> {
    let mut lat_lon: Vec<(f64, f64)> = Vec::new();

    for gpx in files {
        let track = &gpx.tracks[0];
        let segment = &track.segments[0];
        let points = &segment.points;

        points
            .iter()
            .for_each(|pt| lat_lon.push((pt.point().y(), pt.point().x())));
    }

    lat_lon
}

fn get_bounding(lat_lon: Vec<(f64, f64)>) -> Result<Bounding, Error> {
    let lat_min: f64 = lat_lon.iter().map(|e| e.0).reduce(f64::min).unwrap();
    let lat_max: f64 = lat_lon.iter().map(|e| e.0).reduce(f64::max).unwrap();
    let lon_min: f64 = lat_lon.iter().map(|e| e.1).reduce(f64::min).unwrap();
    let lon_max: f64 = lat_lon.iter().map(|e| e.1).reduce(f64::max).unwrap();

    Ok(Bounding {
        lat_min,
        lat_max,
        lon_min,
        lon_max,
    })
}

fn download_tile(tile_url: &str, tile_file: &str) {
    let client = reqwest::blocking::Client::new();

    let img_bytes = client
        .get(tile_url)
        .header(USER_AGENT, "Mozilla/5.0")
        .send()
        .unwrap()
        .bytes()
        .unwrap();

    let mut image = image::load_from_memory(&img_bytes).unwrap().grayscale();
    image.invert();

    image.save(tile_file).unwrap();

    println!("downloading {tile_url} to {tile_file}");
}

fn download_tiles(map_tiles: &MapTiles) {
    let zoom = map_tiles.zoom;

    for x in map_tiles.x_tile_min..=map_tiles.x_tile_max {
        for y in map_tiles.y_tile_min..=map_tiles.y_tile_max {
            let tile_url = format!("{OSM_TILE_SERVER}/{zoom}/{x}/{y}.png");
            let tile_file = format!("assets/tiles/tile_{zoom}_{x}_{y}.png");

            let exists = fs::metadata(&tile_file).is_ok();

            if !exists {
                download_tile(&tile_url, &tile_file)
            };
        }
    }
}

fn get_map_tiles(bounding: Bounding) -> MapTiles {
    let mut zoom = -1;

    let mut x_tile_min;
    let mut y_tile_max;
    let mut x_tile_max;
    let mut y_tile_min;

    if zoom > -1 {
        let zoom = OSM_MAX_ZOOM;
        x_tile_min = deg_to_xy(bounding.lat_min, bounding.lon_min, zoom).0 as i32;
        y_tile_max = deg_to_xy(bounding.lat_min, bounding.lon_min, zoom).1 as i32;
        x_tile_max = deg_to_xy(bounding.lat_max, bounding.lon_max, zoom).0 as i32;
        y_tile_min = deg_to_xy(bounding.lat_max, bounding.lon_max, zoom).1 as i32;
    } else {
        zoom = OSM_MAX_ZOOM;
        loop {
            x_tile_min = deg_to_xy(bounding.lat_min, bounding.lon_min, zoom).0 as i32;
            y_tile_max = deg_to_xy(bounding.lat_min, bounding.lon_min, zoom).1 as i32;
            x_tile_max = deg_to_xy(bounding.lat_max, bounding.lon_max, zoom).0 as i32;
            y_tile_min = deg_to_xy(bounding.lat_max, bounding.lon_max, zoom).1 as i32;

            if (x_tile_max - x_tile_min + 1) * OSM_TILE_SIZE <= HEATMAP_MAX_SIZE.0
                && (y_tile_max - y_tile_min + 1) * OSM_TILE_SIZE <= HEATMAP_MAX_SIZE.1
            {
                break;
            }

            zoom -= 1;
        }
    }

    MapTiles {
        zoom,
        x_tile_min,
        y_tile_max,
        x_tile_max,
        y_tile_min,
    }
}

fn get_polylines(map_tiles: &MapTiles, activities: &Vec<Gpx>) -> Vec<Vec<Vec3>> {
    let mut polylines: Vec<Vec<Vec3>> = Vec::new();

    let z_min = activities
        .iter()
        .flat_map(|activitie| &activitie.tracks[0].segments[0].points)
        .map(|pt| pt.elevation.unwrap())
        .reduce(f64::min)
        .unwrap() as f32;

    let z_max = activities
        .iter()
        .flat_map(|activitie| &activitie.tracks[0].segments[0].points)
        .map(|pt| pt.elevation.unwrap())
        .reduce(f64::max)
        .unwrap() as f32;

    let dz = z_max - z_min;

    let tx_min = map_tiles.x_tile_min as f32;
    let ty_min = map_tiles.y_tile_min as f32;
    let nx = map_tiles.get_nx() as f32;
    let ny = map_tiles.get_ny() as f32;

    for activitie in activities {
        let mut vertices: Vec<Vec3> = Vec::new();

        let points = &activitie.tracks[0].segments[0].points;

        points.iter().for_each(|pt| {
            let (x, y) = deg_to_xy(pt.point().y(), pt.point().x(), map_tiles.zoom);
            let z = pt.elevation.unwrap() as f32;

            let x = x as f32;
            let y = y as f32;

            let x = (x - tx_min) / nx;
            let y = (y - ty_min) / ny;

            let x: f32 = (-0.5 + x) * nx * 2.56;
            let y: f32 = ny * (-0.5 + y) * 2.56;
            let z = 0.1 + (z - z_min) / dz * 2.0;

            vertices.push(Vec3::new(x as f32, z as f32, y as f32));
        });

        polylines.push(vertices);
    }

    polylines
}

fn main() -> Result<(), Error> {
    fs::create_dir_all("./assets/tiles")?;

    let activities = read_gpx_files()?;

    let bounding = get_bounding(get_lat_lon(&activities))?;

    let map_tiles = get_map_tiles(bounding);

    download_tiles(&map_tiles);

    let polylines: Vec<Vec<Vec3>> = get_polylines(&map_tiles, &activities);

    viewer::run(map_tiles, polylines);

    Ok(())
}
