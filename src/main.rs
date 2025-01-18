use clap::{Parser, ValueEnum};

use core::fmt;
use std::io::Write;

#[derive(Clone, Copy)]
struct Filter {
    kind: HslComponent,
    top: f32,
    bottom: f32,
}

// ================================================================================================
#[derive(Clone, Copy, ValueEnum, Default, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    #[default]
    Right,
}
impl fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Up => write!(f, "up"),
            Direction::Left => write!(f, "left"),
            Direction::Down => write!(f, "down"),
            Direction::Right => write!(f, "right"),
        }
    }
}

#[derive(Clone, Copy, ValueEnum, Default, Debug)]
enum HslComponent {
    Hue,
    Saturation,
    #[default]
    Luminosity,
}
impl fmt::Display for HslComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HslComponent::Hue => write!(f, "hue"),
            HslComponent::Saturation => write!(f, "saturation"),
            HslComponent::Luminosity => write!(f, "luminosity"),
        }
    }
}

// ================================================================================================
#[derive(Clone, Copy)]
struct Rgb8 {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Clone, Copy)]
struct Hsl {
    hue: f32,
    saturation: f32,
    luminosity: f32,
}

#[derive(Clone, Copy)]
struct Pixel {
    rgb: Rgb8,
    hsl: Hsl,
}
impl Pixel {
    fn new(red: u8, green: u8, blue: u8) -> Pixel {
        let rgb_values: Rgb8 = Rgb8 {
            r: red,
            g: green,
            b: blue,
        };

        Pixel {
            rgb: rgb_values.clone(),
            hsl: calculate_hsl(rgb_values),
        }
    }
}

struct ImageData {
    height: u32,
    width: u32,
    data: Vec<Pixel>,
}

// ================================================================================================
fn main() {
    let arguments = Args::parse();

    pixel_sort(
        arguments.input.as_str(),
        arguments.output.as_str(),
        Filter {
            kind: arguments.kind,
            bottom: arguments.bottom,
            top: arguments.top,
        },
        arguments.direction,
    );
}

fn pixel_sort(source_path: &str, target_path: &str, filter: Filter, direction: Direction) {
    fn check_eligibility(pixel: Pixel, filter: Filter) -> bool {
        match filter.kind {
            HslComponent::Hue => {
                todo!("mek good!!!");
            }
            HslComponent::Luminosity => {
                pixel.hsl.luminosity >= filter.bottom && pixel.hsl.luminosity <= filter.top
            }
            HslComponent::Saturation => {
                pixel.hsl.saturation >= filter.bottom && pixel.hsl.saturation <= filter.top
            }
        }
    }

    print!("reading image: ");
    std::io::stdout().flush().unwrap();
    let mut image = read_image(source_path, direction).unwrap();
    print!("done\n");

    print!("sorting spans: ");
    std::io::stdout().flush().unwrap();
    {
        let mut current_span: Option<usize> = None;
        for i in 0..(image.width * image.height) {
            let i: usize = i as usize;

            if (!check_eligibility(image.data[i], filter) || i % image.width as usize == 0)
                && current_span.is_some()
            {
                let span = &mut image.data[current_span.unwrap()..i];

                span.sort_unstable_by(|x: &Pixel, y: &Pixel| -> std::cmp::Ordering {
                    match filter.kind {
                        HslComponent::Luminosity => {
                            x.hsl.luminosity.partial_cmp(&y.hsl.luminosity).unwrap()
                        }
                        HslComponent::Saturation => {
                            x.hsl.saturation.partial_cmp(&y.hsl.saturation).unwrap()
                        }
                        HslComponent::Hue => x.hsl.hue.partial_cmp(&y.hsl.hue).unwrap(),
                    }
                });
                current_span = None;
            } else if check_eligibility(image.data[i], filter) && current_span.is_none() {
                current_span = Some(i)
            }
        }
    }
    print!("done\n");

    print!("writing image: ");
    std::io::stdout().flush().unwrap();
    write_image(target_path, image, direction);
    print!("done\n");
}

// ================================================================================================
// argument parsing
#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path (absoloute or relative) to the input image.
    #[arg(short, long)]
    input: String,

    /// Path (absoloute or relative) to the output image. The output will be overwritten something on the path allready exists.
    #[arg(short, long)]
    output: String,

    /// The metric to select spans and sort by.
    #[arg(short, long, default_value_t)]
    kind: HslComponent,

    /// The bottom of the range to select.
    #[arg(short, long, default_value_t = 0.0)]
    bottom: f32,

    /// The top of the range to select.
    #[arg(short, long, default_value_t = 0.7)]
    top: f32,

    // The direction to select span in.
    #[arg(short, long, default_value_t)]
    direction: Direction,
}

// ================================================================================================
// Implement HSL calculation according to
// https://www.niwa.nu/2013/05/math-behind-colorspace-conversions-rgb-hsl
fn calculate_hsl(rgb_values: Rgb8) -> Hsl {
    let red: f32 = rgb_values.r as f32 / 255.0;
    let green: f32 = rgb_values.g as f32 / 255.0;
    let blue: f32 = rgb_values.b as f32 / 255.0;

    let min: f32 = f32::min(red, f32::min(green, blue)); // for some fucking reason min() is not fucking const
    let max: f32 = f32::max(red, f32::max(green, blue));
    // min maxxing

    let luminosity: f32 = 0.5 * (max + min);

    let saturation: f32;
    if luminosity < 1.0 {
        saturation = (max - min) / (1.0 - (2.0 * luminosity - 1.0).abs());
    } else {
        saturation = 0.0;
    };

    let hue: f32;
    let pre_hue: f32;
    if red == max {
        pre_hue = 60.0 * (0.0 + (green - blue) / (max - min));
    } else if green == max {
        pre_hue = 60.0 * (2.0 + (blue - red) / (max - min));
    } else if blue == max {
        pre_hue = 60.0 * (4.0 + (red - green) / (max - min));
    } else {
        panic!();
    }
    if pre_hue >= 0.0 {
        hue = pre_hue;
    } else {
        hue = pre_hue + 360.0;
    }

    Hsl {
        hue,
        saturation,
        luminosity,
    }
}

// ================================================================================================
// File interaction functions using:
// https://docs.rs/image/latest/image
fn read_image(file_path: &str, direction: Direction) -> Result<ImageData, image::ImageError> {
    // I now haz a image.
    let mut image = image::ImageReader::open(file_path)?.decode()?;

    // Turn the image so the sorting direction is accounted for.
    image = match direction {
        Direction::Right => image,
        Direction::Up => image.rotate90(),
        Direction::Left => image.rotate180(),
        Direction::Down => image.rotate270(),
    };

    // Create the image data struct and give it width and height.
    let mut image_data = ImageData {
        height: image.height(),
        width: image.width(),
        data: Vec::new(),
    };

    // Fill the data vector with pixel RGB values.
    image_data.data = image
        .into_rgb8()
        .pixels()
        .into_iter()
        .map(|pixel: &image::Rgb<u8>| -> Pixel { Pixel::new(pixel.0[0], pixel.0[1], pixel.0[2]) })
        .collect();

    return Ok(image_data);
}

fn write_image(file_path: &str, image_data: ImageData, direction: Direction) {
    // Create an imagebuffer with the size of our raw data.
    let mut image =
        image::DynamicImage::new(image_data.width, image_data.height, image::ColorType::Rgb8);

    // Write into the image buffer from the raw data.
    for (x, y, pixel) in image.as_mut_rgb8().unwrap().enumerate_pixels_mut() {
        pixel[0] = image_data.data[((y * image_data.width) + x) as usize].rgb.r;
        pixel[1] = image_data.data[((y * image_data.width) + x) as usize].rgb.g;
        pixel[2] = image_data.data[((y * image_data.width) + x) as usize].rgb.b;
    }

    // Turn the image back.
    image = match direction {
        Direction::Right => image,
        Direction::Up => image.rotate270(),
        Direction::Left => image.rotate180(),
        Direction::Down => image.rotate90(),
    };

    // Write the image buffer to disk.
    image.save(file_path).unwrap();
}
