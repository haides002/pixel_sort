struct RgbPixel8 {
    r: u8,
    g: u8,
    b: u8,
}
impl Clone for RgbPixel8 {
    fn clone(&self) -> RgbPixel8 {
        RgbPixel8 {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

struct ImageData {
    height: u32,
    width: u32,
    data: Vec<RgbPixel8>,
}

// ================================================================================================
fn main() {
    let test_pixel = RgbPixel8 {
        r: 66,
        g: 135,
        b: 245,
    };

    print!("{}", saturation(test_pixel));
    let image =
        read_image("/home/linus/development/rust/pixel_sort/testing/test_image.jpg").unwrap();
    write_image(
        "/home/linus/development/rust/pixel_sort/testing/output.jpg",
        image,
    );
}

// ================================================================================================
fn luminosity(values: RgbPixel8) -> f32 {
    let red: f32 = values.r as f32 / 255.0;
    let green: f32 = values.g as f32 / 255.0;
    let blue: f32 = values.b as f32 / 255.0;

    let min: f32 = f32::min(red, f32::min(green, blue)); // for some fucking reason min is not fucking const
    let max: f32 = f32::max(red, f32::max(green, blue)); // min maxxing

    0.5 * (max + min)
}

fn saturation(values: RgbPixel8) -> f32 {
    let red: f32 = values.r as f32 / 255.0;
    let green: f32 = values.g as f32 / 255.0;
    let blue: f32 = values.b as f32 / 255.0;

    let min: f32 = f32::min(red, f32::min(green, blue));
    let max: f32 = f32::max(red, f32::max(green, blue));

    if luminosity(values.clone()) < 1.0 {
        return (max - min) / (1.0 - (2.0 * luminosity(values.clone()) - 1.0).abs());
    } else {
        return 0.0;
    }
}

// ================================================================================================
fn write_image(file_path: &str, image_data: ImageData) {
    // create an imagebuffer with the size of our raw data
    let mut image_buffer =
        image::DynamicImage::new(image_data.width, image_data.height, image::ColorType::Rgb8);

    // write into the image buffer from the raw data
    {
        let mut y: u32 = 0;
        let mut id: usize = 0;
        while y < image_data.height {
            let mut x: u32 = 0;
            while x < image_data.width {
                let pixel = image::Rgb {
                    0: [
                        image_data.data[id].r,
                        image_data.data[id].g,
                        image_data.data[id].b,
                    ],
                };

                image_buffer.as_mut_rgb8().unwrap().put_pixel(x, y, pixel);
                x = x + 1;
                id = id + 1;
            }
            y = y + 1;
        }
    }

    // write the image buffer to disk
    image_buffer.save(file_path).unwrap();
}

fn read_image(file_path: &str) -> Result<ImageData, image::ImageError> {
    // i now haz a image_buffer
    let binding = image::ImageReader::open(file_path)?.decode()?;
    let image_buffer = binding
        .as_rgb8()
        .expect("Failed to convert image to rgb8 format!");

    // create the image data struct & give it width and height
    let mut image_data = ImageData {
        height: image_buffer.height(),
        width: image_buffer.width(),
        data: vec![],
    };

    // fill the data vector with pixel RGB values
    {
        let mut y: u32 = 0;
        while y < image_data.height {
            let mut x: u32 = 0;
            while x < image_data.width {
                let pixel = RgbPixel8 {
                    r: image_buffer.get_pixel(x, y).0[0],
                    g: image_buffer.get_pixel(x, y).0[1],
                    b: image_buffer.get_pixel(x, y).0[2],
                };
                image_data.data.push(pixel); //WARN:not catching the possible panic
                x = x + 1;
            }
            y = y + 1;
        }
    }

    return Ok(image_data);
}
