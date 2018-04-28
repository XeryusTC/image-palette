#[macro_use]
extern crate clap;
extern crate image;

use clap::{App, Arg};

use image::GenericImage;
use image::Pixel;
use std::collections::HashMap;

fn dist(c1: &image::Rgb<u8>, c2: &image::Rgb<u8>) -> u8
{
    let d1 = if c1[0] > c2[0] { c1[0] - c2[0] } else { c2[0] - c1[0] } as u64;
    let d2 = if c1[1] > c2[1] { c1[1] - c2[1] } else { c2[1] - c1[1] } as u64;
    let d3 = if c1[2] > c2[2] { c1[2] - c2[2] } else { c2[2] - c1[2] } as u64;
    ((d1 * d1 + d2 * d2 + d3 * d3) as f64).sqrt() as u8
}

fn main() {
    let matches = App::new("Theme palette")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("IMAGE")
             .help("The image to create a palette for")
             .required(true)
             .index(1))
        .arg(Arg::with_name("results")
             .help("Number of colors to return")
             .takes_value(true)
             .short("r")
             .long("results")
             .default_value("8"))
        .arg(Arg::with_name("group_distance")
             .help("Max Euclidean distance between colors in group")
             .takes_value(true)
             .short("d")
             .long("distance")
             .default_value("16"))
        .get_matches();

    let image_name = matches.value_of("IMAGE").unwrap();
    let results = value_t!(matches, "results", usize).unwrap();
    let distance = value_t!(matches, "group_distance", u8).unwrap();
    println!("Loading {}...", image_name);
    let img = image::open(image_name).unwrap();

    // Calculate mean
    let mut red: u64 = 0;
    let mut green: u64 = 0;
    let mut blue: u64 = 0;
    for (_, _, pixel) in img.pixels() {
        let pixel = pixel.to_rgb();
        red += pixel[0] as u64;
        green += pixel[1] as u64;
        blue += pixel[2] as u64;
    }
    let pixels = img.width() as u64 * img.height() as u64;
    red /= pixels;
    green /= pixels;
    blue /= pixels;

    println!("Mean color: #{:x}{:x}{:x}", red, green, blue);

    // Calculate most often used
    let mut hist = HashMap::new();
    for (_, _, pixel) in img.pixels() {
        let pixel = pixel.to_rgb();
        let d = hist.entry(pixel).or_insert(0 as usize);
        *d += 1;
    }
    let mut count = hist.iter().collect::<Vec<(&image::Rgb<u8>, &usize)>>();
    count.sort_by_key(|&(_k, v)| v);
    println!("Most common colors");
    for (rank, color) in count.iter().rev().enumerate() {
        if rank >= results {
            break;
        }
        println!("{:>2}: #{:02x}{:02x}{:02x}",
                 rank + 1, color.0[0], color.0[1], color.0[2]);
    }

    let mut group_count: HashMap<image::Rgb<u8>, usize> = HashMap::new();
    'outer: for &(color, cnt) in count.iter() {
        for (group, val) in group_count.iter_mut() {
            if dist(color, group) < distance {
                *val += cnt;
                continue 'outer;
            }
        }
        group_count.insert(*color, *cnt);
    }
    let mut grouped = group_count.iter()
        .collect::<Vec<(&image::Rgb<u8>, &usize)>>();
    grouped.sort_by_key(|&(_, v)| v);
    println!("Grouped most common colors");
    for (rank, color) in grouped.iter().rev().enumerate() {
        if rank >= results {
            break;
        }
        println!("{:>2}: #{:02x}{:02x}{:02x}",
                 rank + 1, color.0[0], color.0[1], color.0[2]);
    }
}
