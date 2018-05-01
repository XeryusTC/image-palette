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
use std::cmp;

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
    let mut cur_dist: u64;
    let stop_crit = (cmp::max(img.width(), img.height()) as f64).sqrt();
    println!("Stop criterium: {}", stop_crit);

    for _ in 0..labels.capacity() {
        labels.push(0);
    }

    loop {
        // Assign clusters
        let mut center_totals: Vec<[u64; 3]> = Vec::with_capacity(clusters);
        let mut sizes = Vec::with_capacity(clusters);
        for _ in 0..clusters {
            center_totals.push([0, 0, 0]);
            sizes.push(0);
        }
        for (i, pixel) in img.pixels().enumerate() {
            cur_dist = u64::max_value();
            for c in 0..clusters {
                let d = dist(&centers[c], pixel);
                if d <= cur_dist {
                    labels[i] = c;
                    cur_dist = d;
                    center_totals[c][0] += pixel[0] as u64;
                    center_totals[c][1] += pixel[1] as u64;
                    center_totals[c][2] += pixel[2] as u64;
                    sizes[c] += 1;
                }
            }
        }

        // Calculate new centers
        let old_centers = centers.clone();
        for i in 0..clusters {
            assert!(sizes[i] != 0, "A cluster is empty, please try again");
            centers[i] = image::Rgb { data: [
                (center_totals[i][0] / sizes[i]) as u8,
                (center_totals[i][1] / sizes[i]) as u8,
                (center_totals[i][2] / sizes[i]) as u8
            ]};
        }
        let delta: u64 = centers.iter().zip(old_centers.iter())
            .map(|(n, o)| dist(n, o)).sum();
        if delta as f64 <= stop_crit {
            break;
        }
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
