extern crate image;

use std::ptr;
use std::time::Instant;

use image::{GenericImageView, ImageBuffer, RgbImage};

#[derive(Copy, Clone)]
struct SendPointer(*mut u8);
unsafe impl Sync for SendPointer {

}

unsafe impl Send for SendPointer {

}

fn main() {

    let input_path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/totk.jpg");
    let input_image = &image::open(input_path).unwrap().to_rgb8();

    let (width, height) = input_image.dimensions();
    let mut output_image:RgbImage = ImageBuffer::new(width, height);
    let nasty_output = SendPointer(output_image.as_mut_ptr());

    rayon::ThreadPoolBuilder::new().build_global();


    let start = Instant::now();

    rayon::scope(|s| {

        for y in 0..height {
            //s.spawn(move |t| {
                for x in 0..width {
                    let pixel = input_image.get_pixel(x, y);
                    //t.spawn(move |_| {
                        let grayscale_value = (pixel[0] as f32 * 0.299 + pixel[1] as f32 * 0.587 + pixel[2] as f32 * 0.114) as u8;
                        let coord = ((y * width + x) * 3) as isize;
                        unsafe {
                            ptr::write_bytes(nasty_output.0.offset(coord), grayscale_value, 3);
                        }
                    //})
                }
            //})
        }

    });

    println!("{:?}", start.elapsed());

    output_image.save(concat!(env!("CARGO_MANIFEST_DIR"), "/target/output.jpg")).unwrap();

}