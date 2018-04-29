#[macro_use]
extern crate clap;
extern crate image;
extern crate rand;

use clap::{App, Arg};

use image::GenericImage;
use image::Pixel;
use rand::thread_rng;
use rand::seq::sample_iter;
use std::collections::HashMap;

fn dist(c1: &image::Rgb<u8>, c2: &image::Rgb<u8>) -> u64
{
    let d1 = if c1[0] > c2[0] { c1[0] - c2[0] } else { c2[0] - c1[0] } as u64;
    let d2 = if c1[1] > c2[1] { c1[1] - c2[1] } else { c2[1] - c1[1] } as u64;
    let d3 = if c1[2] > c2[2] { c1[2] - c2[2] } else { c2[2] - c1[2] } as u64;
    ((d1 * d1 + d2 * d2 + d3 * d3) as f64).sqrt() as u64
}

fn kmeans(img: &image::RgbImage, clusters: usize) -> Vec<image::Rgb<u8>> {
    let mut rng = thread_rng();
    let mut centers = sample_iter(&mut rng, img.pixels(), clusters).unwrap()
        .iter().map(|&x| x.clone()).collect::<Vec<image::Rgb<u8>>>();
    let mut labels = Vec::with_capacity((img.width() * img.height()) as usize);
    let mut old_labels;
    let mut stable = false;

    for _ in 0..labels.capacity() {
        labels.push(0);
    }

    while !stable {
        old_labels = labels.clone();
        // Assign clusters
        for (i, pixel) in img.pixels().enumerate() {
            let mut cur_dist = u64::max_value();
            for c in 0..clusters {
                let d = dist(&centers[c], pixel);
                if d <= cur_dist {
                    labels[i] = c;
                    cur_dist = d;
                }
            }
        }

        // Calculate new centers
        let mut new_centers: Vec<[u64; 3]> = Vec::with_capacity(centers.len());
        for _ in 0..clusters {
            new_centers.push([0, 0, 0]);
        }
        for (i, pixel) in img.pixels().enumerate() {
            new_centers[labels[i]][0] += pixel[0] as u64;
            new_centers[labels[i]][1] += pixel[1] as u64;
            new_centers[labels[i]][2] += pixel[2] as u64;
        }
        for idx in 0..new_centers.len() {
            let size = labels.iter().filter(|l| **l == idx).count() as u64;
            assert!(size != 0);
            if size != 0 {
                new_centers[idx][0] /= size;
                new_centers[idx][1] /= size;
                new_centers[idx][2] /= size;
            }
        }
        for idx in 0..centers.len() {
            centers[idx] = image::Rgb { data: [new_centers[idx][0] as u8,
                            new_centers[idx][1] as u8,
                            new_centers[idx][2] as u8]};
        }
        stable = labels == old_labels;
    }
    centers
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
            if dist(color, group) < distance as u64 {
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

    // Find most often used by k-means
    println!("Grouped by k-means clustering");
    let groups = kmeans(&img.to_rgb(), results);
    for (rank, color) in groups.iter().enumerate() {
        if rank >= results {
            break;
        }
        println!("{:>2}: #{:02x}{:02x}{:02x}",
                 rank + 1, color[0], color[1], color[2]);
    }
}
