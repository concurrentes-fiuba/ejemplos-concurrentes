extern crate image;

use std::arch::aarch64::{vaddvq_f32, vld1q_f32, vld1q_f32_x4, vmov_n_u8, vmulq_f32, vset_lane_u8, vst1_u8};
use std::time::Instant;

use image::{GrayImage, ImageBuffer};

fn main() {

    let input_path = "data/totk.jpg";
    let input_image = &image::open(input_path).unwrap().to_rgba32f();

    let (width, height) = input_image.dimensions();
    let mut output_image:GrayImage = ImageBuffer::new(width, height);

    let mul = unsafe {
        vld1q_f32([0.299 * 255.0, 0.587 * 255.0, 0.144 * 255.0, 1.].as_ptr())
    };

    let start = Instant::now();

    let input_image_vec = input_image.as_raw();
    let output_image_vec = output_image.as_mut();

    for y in 0..height {
        for x in (0..width).step_by(8) {
            let pixel = (y * width + x) as usize;
            let coord = (pixel * 4) as usize;

            unsafe {
                let pixels = vld1q_f32_x4(input_image_vec[coord..].as_ptr());
                let pixels2 = vld1q_f32_x4(input_image_vec[coord+16..].as_ptr());
                let mut multiplied = vmulq_f32(pixels.0, mul);
                let mut out_pixel = vaddvq_f32(multiplied) as u8;

                let out_pixels = vmov_n_u8(out_pixel);

                multiplied = vmulq_f32(pixels.1, mul);
                out_pixel = vaddvq_f32(multiplied) as u8;
                vset_lane_u8::<1>(out_pixel, out_pixels);

                multiplied = vmulq_f32(pixels.2, mul);
                out_pixel = vaddvq_f32(multiplied) as u8;
                vset_lane_u8::<2>(out_pixel, out_pixels);

                multiplied = vmulq_f32(pixels.3, mul);
                out_pixel = vaddvq_f32(multiplied) as u8;
                vset_lane_u8::<3>(out_pixel, out_pixels);

                multiplied = vmulq_f32(pixels2.0, mul);
                out_pixel = vaddvq_f32(multiplied) as u8;
                vset_lane_u8::<4>(out_pixel, out_pixels);

                multiplied = vmulq_f32(pixels2.1, mul);
                out_pixel = vaddvq_f32(multiplied) as u8;
                vset_lane_u8::<5>(out_pixel, out_pixels);

                multiplied = vmulq_f32(pixels2.2, mul);
                out_pixel = vaddvq_f32(multiplied) as u8;
                vset_lane_u8::<6>(out_pixel, out_pixels);

                multiplied = vmulq_f32(pixels2.3, mul);
                out_pixel = vaddvq_f32(multiplied) as u8;
                vset_lane_u8::<7>(out_pixel, out_pixels);

                vst1_u8(output_image_vec[pixel..].as_mut_ptr(), out_pixels)

            };
        }
    };

    println!("{:?}", start.elapsed());

    output_image.save(concat!(env!("CARGO_MANIFEST_DIR"), "/target/output.jpg")).unwrap();

}