use image::{Luma, Rgb};
use imageproc::{map::map_colors, pixelops::weighted_sum};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello world!");
    let args = env::args().collect::<Vec<_>>();
    let img = image::open(&args[1])?;

    let start = time::Instant::now();
    let img = img.to_luma8();
    let blue = Rgb([0u8, 0u8, 255u8]);

    println!("Begin tinted");

    let tinted = map_colors(&img, |pix| {|gray: Luma<u8>, color: Rgb<u8>| -> Rgb<u8> {
        let dist_from_mid = ((gray[0] as f32 - 128f32).abs()) / 255f32;
        let scale_factor = 1f32 - 4f32 * dist_from_mid.powi(2);
        weighted_sum(Rgb([gray[0]; 3]), color, 1.0, scale_factor)
    }}(pix, blue));

    println!("End tinted");

    let end = time::Instant::now();
    println!("Took {} seconds to process image.", end - start);
    tinted.save("test.jpg")?;

    Ok(())
}
