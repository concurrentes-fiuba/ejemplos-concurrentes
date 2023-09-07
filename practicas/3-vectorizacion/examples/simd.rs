#![feature(portable_simd)]

extern crate image;

use std::ops::Mul;
use std::simd::{f32x4, SimdFloat};
use std::time::Instant;

use image::{GenericImageView, ImageBuffer, RgbImage};

fn main() {

    let input_path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/totk.jpg");
    let input_image = &image::open(input_path).unwrap().to_rgba32f();

    let (width, height) = input_image.dimensions();
    let mut output_image:RgbImage = ImageBuffer::new(width, height);

    let start = Instant::now();

    let mul = f32x4::from_array([0.299 * 255.0, 0.587 * 255.0, 0.144 * 255.0, 0.]);

    let input_image_vec = input_image.as_raw();
    let output_image_vec = output_image.as_mut();


    for y in 0..height {
        for x in (0..width) {
            let pixel = (y * width + x);
            let coord = (pixel * 4) as usize;
            let pixel0 = f32x4::from_slice(&input_image_vec[coord..])
                .mul(mul)
                .reduce_sum() as u8;
            let out_coord = (pixel * 3) as usize;
            output_image_vec[out_coord..(out_coord+3)].copy_from_slice(&[pixel0, pixel0, pixel0])
        }
    };

    println!("{:?}", start.elapsed());

    output_image.save(concat!(env!("CARGO_MANIFEST_DIR"), "/target/output.jpg")).unwrap();

}