#![feature(portable_simd)]

extern crate image;

use std::ops::Mul;
use std::simd::{f32x16, f32x8, mask32x16, SimdFloat};
use std::time::Instant;

use image::{GenericImageView, ImageBuffer, RgbImage};

fn main() {

    let input_path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/totk.jpg");
    let input_image = &image::open(input_path).unwrap().to_rgba32f();

    let (width, height) = input_image.dimensions();
    let mut output_image:RgbImage = ImageBuffer::new(width, height);

    let start = Instant::now();

    let mul = f32x16::from_array([
        0.299 * 255.0, 0.587 * 255.0, 0.144 * 255.0,
        0.299 * 255.0, 0.587 * 255.0, 0.144 * 255.0,
        0.299 * 255.0, 0.587 * 255.0, 0.144 * 255.0,
        0.299 * 255.0, 0.587 * 255.0, 0.144 * 255.0,
        0.299 * 255.0, 0.587 * 255.0, 0.144 * 255.0,
        0.
    ]) ;

    let zeros = f32x16::splat(0.);

    let input_image_vec = input_image.as_raw();
    let output_image_vec = output_image.as_mut();

    let mask0 = mask32x16::from_array([
        true, true, true,
        false, false, false,
        false, false, false,
        false, false, false,
        false, false, false, false]);

    let mask1 = mask32x16::from_array([
        false, false, false,
        true, true, true,
        false, false, false,
        false, false, false,
        false, false, false, false]);

    let mask2 = mask32x16::from_array([
        false, false, false,
        false, false, false,
        true, true, true,
        false, false, false,
        false, false, false, false]);

    let mask3 = mask32x16::from_array([
        false, false, false,
        false, false, false,
        false, false, false,
        true, true, true,
        false, false, false, false]);

    let mask4 = mask32x16::from_array([
        false, false, false,
        false, false, false,
        false, false, false,
        false, false, false,
        true, true, true, false]);

    for y in 0..height {
        for x in (0..width).step_by(5) {
            let pixel = (y * width + x);
            let coord = (pixel * 4) as usize;
            let pixels = f32x16::from_slice(&input_image_vec[coord..])
                .mul(mul);
            let pixel0 = mask0.select(pixels, zeros).reduce_sum() as u8;
            let pixel1 = mask1.select(pixels, zeros).reduce_sum() as u8;
            let pixel2 = mask2.select(pixels, zeros).reduce_sum() as u8;
            let pixel3 = mask3.select(pixels, zeros).reduce_sum() as u8;
            let pixel4 = mask4.select(pixels, zeros).reduce_sum() as u8;
            let out_coord = (pixel * 3) as usize;
            output_image_vec[out_coord..(out_coord+15)].copy_from_slice(&[pixel0, pixel0, pixel0, pixel1, pixel1, pixel1, pixel2, pixel2, pixel2, pixel3, pixel3, pixel3, pixel4, pixel4, pixel4]);
        }
    };

    println!("{:?}", start.elapsed());

    output_image.save(concat!(env!("CARGO_MANIFEST_DIR"), "/target/output.jpg")).unwrap();

}