extern crate clap;
extern crate image;

use clap::*;

use image::GenericImage;
use image::Pixel;
use std::collections::HashMap;

fn main() {
    let matches = App::new("Theme palette")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("IMAGE")
             .help("The image to create a palette for")
             .required(true)
             .index(1))
        .get_matches();

    let image_name = matches.value_of("IMAGE").unwrap();
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
        let d = hist.entry(pixel).or_insert(0 as u8);
        *d += 1;
    }
    let mut count = hist.iter().collect::<Vec<(&image::Rgb<u8>, &u8)>>();
    count.sort_by_key(|&(_k, v)| v);
    println!("Most common colors");
    for (rank, color) in count.iter().rev().enumerate() {
        if rank >= 8 {
            break;
        }
        println!("{:>2}: #{:02x}{:02x}{:02x}",
                 rank + 1, color.0[0], color.0[1], color.0[2]);
    }
}
