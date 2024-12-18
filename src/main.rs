use clap::Parser;

#[derive(Clone, Copy)]
struct Filter {
    kind: HslComponent,
    top: f32,
    bottom: f32,
}

// ================================================================================================
#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy)]
enum HslComponent {
    Hue,
    Saturation,
    Luminosity,
}

// ================================================================================================
#[derive(Clone)]
struct Span {
    position: Position,
    data: Vec<Pixel>,
}

#[derive(Clone, Copy)]
struct Position {
    x: u32,
    y: u32,
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
    let args: Args = PreArgs::parse().into_args();

    pixel_sort(
        args.input.as_str(),
        args.output.as_str(),
        Filter {
            kind: args.kind,
            bottom: args.bottom,
            top: args.top,
        },
        args.direction,
    );
}

fn pixel_sort(source_path: &str, target_path: &str, filter: Filter, direction: Direction) {
    print!("reading image:		");
    let mut image = read_image(source_path).unwrap();
    print!("done\n");

    print!("extracting spans:	");
    let mut spans = extract_spans(&image, filter, direction);
    print!("done({})\n", spans.len());

    print!("sorting spans:		");
    let sorted_spans = sort_spans(&mut spans, filter.kind);
    print!("done\n");

    print!("inserting spans:	");
    insert_spans(&mut image, sorted_spans, direction);
    print!("done\n");

    print!("writing image:		");
    write_image(target_path, image);
    print!("done\n");
}

// ================================================================================================
// argument parsing

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct PreArgs {
    /// Path (absoloute or relative) to the input image.
    #[arg(short, long)]
    input: String,

    /// Path (absoloute or relative) to the output image. The output will be overwritten something on the path allready exists.
    #[arg(short, long)]
    output: String,

    /// The metric to select spans and sort by.
    #[arg(short, long, default_value = "Luminosity")]
    kind: String,

    /// The bottom of the range to select.
    #[arg(short, long, default_value_t = 0.0)]
    bottom: f32,

    /// The top of the range to select.
    #[arg(short, long, default_value_t = 0.7)]
    top: f32,

    // The direction to select span in.
    #[arg(short, long, default_value = "right")]
    direction: String,
}
struct Args {
    input: String,
    output: String,

    kind: HslComponent,
    bottom: f32,
    top: f32,

    direction: Direction,
}

impl PreArgs {
    fn into_args(&self) -> Args {
        let input_path: String = std::fs::canonicalize(self.input.clone())
            .expect("Invalid input path!")
            .to_str()
            .unwrap()
            .into();

        let output_path: String;

        if std::path::absolute(self.output.clone())
            .unwrap()
            .parent()
            .unwrap()
            .is_dir()
        {
            output_path = std::path::absolute(self.output.clone())
                .unwrap()
                .to_str()
                .unwrap()
                .into()
        } else {
            panic!("Invalid output path!");
        }

        Args {
            input: input_path,
            output: output_path,
            kind: match self.kind.as_str() {
                "Hue" | "hue" | "h" => HslComponent::Hue,
                "Saturation" | "saturation" | "s" => HslComponent::Saturation,
                "Luminosity" | "luminosity" | "l" => HslComponent::Luminosity,

                _ => {
                    panic!("Invalid filter kind!");
                }
            },
            bottom: self.bottom,
            top: self.top,
            direction: match self.direction.as_str() {
                "Up" | "up" | "u" => Direction::Up,
                "Down" | "down" | "d" => Direction::Down,
                "Left" | "left" | "l" => Direction::Left,
                "Right" | "right" | "r" => Direction::Right,

                _ => {
                    panic!("Invalid direction!");
                }
            },
        }
    }
}

// ================================================================================================
// span fuckery
fn extract_spans(image: &ImageData, filter: Filter, direction: Direction) -> Vec<Span> {
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

    let mut spans: Vec<Span> = Vec::new();
    let mut current_span: Span = Span {
        position: Position { x: 0, y: 0 },
        data: Vec::new(),
    };

    //for (i, pixel) in image.data.iter().enumerate() {
    let mut id: u32 = match direction {
        Direction::Right => 0,
        Direction::Down => 0,
        Direction::Up => image.width * image.height - 1,
        Direction::Left => image.width * image.height - 1,
    };

    loop {
        // check if we need to span break
        if !check_eligibility(image.data[id as usize], filter)
            || calculate_position(id as u32, image.width).x == 0
            || calculate_position(id as u32, image.width).y == 0
        {
            if current_span.data.len() > 1 {
                spans.push(current_span);
            };

            current_span = Span {
                position: Position { x: 0, y: 0 },
                data: Vec::new(),
            };
        }

        if check_eligibility(image.data[id as usize], filter) {
            if current_span.data.is_empty() {
                current_span.position = calculate_position(id, image.width);
            }
            current_span.data.push(image.data[id as usize]);
        }

        match next_pixel_id(id, image.width, image.height, direction) {
            None => {
                break;
            }
            Some(new_id) => {
                id = new_id;
                continue;
            }
        }
    }

    return spans;
}

fn insert_spans(image: &mut ImageData, spans: Vec<Span>, direction: Direction) -> &ImageData {
    for span in spans {
        // Calculate span origin position as ID.
        let mut current_pixel_id: u32 = calculate_id(
            Position {
                x: span.position.x,
                y: span.position.y,
            },
            image.width,
        );
        //println!("Calculating span at id: {}.", current_pixel_id);

        for pixel in span.data {
            image.data[current_pixel_id as usize] = pixel;

            match next_pixel_id(current_pixel_id, image.width, image.height, direction) {
                Some(value) => {
                    current_pixel_id = value;
                }
                None => {
                    break;
                }
            }
        }
    }
    return image;
}

fn sort_spans(spans: &mut Vec<Span>, sort_type: HslComponent) -> Vec<Span> {
    fn sort_pixels(mut pixels: Vec<Pixel>, sort_type: HslComponent) -> Vec<Pixel> {
        // return if sorting is not necessary
        if pixels.len() <= 1 {
            return pixels;
        }

        // chose pivot and initialize left and right parts
        let pivot: Pixel = pixels.pop().unwrap();
        let mut left: Vec<Pixel> = Vec::new();
        let mut right: Vec<Pixel> = Vec::new();

        // divide pixels into two
        for element in pixels {
            if element.hsl.luminosity < pivot.hsl.luminosity {
                left.push(element);
            } else {
                right.push(element);
            }
        }

        // asseble and return final vector
        let mut sorted: Vec<Pixel> = Vec::new();
        sorted.append(&mut sort_pixels(left, sort_type.clone()));
        sorted.push(pivot);
        sorted.append(&mut sort_pixels(right, sort_type.clone()));

        sorted
    }

    let mut sorted_spans: Vec<Span> = Vec::new();
    for span in spans {
        span.data = sort_pixels(span.data.clone(), sort_type);
        sorted_spans.push(span.clone());
    }

    return sorted_spans;
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
// image helper funktions
const fn calculate_id(position: Position, width: u32) -> u32 {
    (position.y * width) + position.x
}

const fn calculate_position(id: u32, width: u32) -> Position {
    Position {
        y: id / width,
        x: id % width,
    }
}

const fn next_pixel_id(id: u32, width: u32, height: u32, direction: Direction) -> Option<u32> {
    if id >= width * height {
        panic!("next_pixel_id: id is out of bounds");
    }

    match direction {
        Direction::Right => {
            if id + 1 >= width * height {
                return None;
            } else {
                return Some(id + 1);
            }
        }
        Direction::Left => {
            return id.checked_sub(1);
        }
        Direction::Down => {
            if (id + width) < (width * height) {
                return Some(id + width);
            } else if id == width * height - 1 {
                return None;
            } else {
                return Some((id % width) + 1); // but im not a rapper
            }
        }
        Direction::Up => {
            if id == 0 {
                return None;
            } else if (id as i32 - width as i32) >= 0 {
                return Some(id - width);
            } else {
                return Some(((height - 1) * width) + (id - 1));
            }
        }
    }
}

// ================================================================================================
// File interaction functions using:
// https://docs.rs/image/latest/image
fn write_image(file_path: &str, image_data: ImageData) {
    // Create an imagebuffer with the size of our raw data.
    let mut image_buffer: image::RgbImage =
        image::DynamicImage::new(image_data.width, image_data.height, image::ColorType::Rgb8)
            .into_rgb8();

    // Write into the image buffer from the raw data.
    for (i, pixel) in image_buffer.pixels_mut().enumerate() {
        pixel[0] = image_data.data[i].rgb.r;
        pixel[1] = image_data.data[i].rgb.g;
        pixel[2] = image_data.data[i].rgb.b;
    }

    // Write the image buffer to disk.
    image_buffer.save(file_path).unwrap();
}

fn read_image(file_path: &str) -> Result<ImageData, image::ImageError> {
    // I now haz a image_buffer.
    let image_buffer = image::ImageReader::open(file_path)?.decode()?.into_rgb8();

    // Create the image data struct and give it width and height.
    let mut image_data = ImageData {
        height: image_buffer.height(),
        width: image_buffer.width(),
        data: Vec::new(),
    };

    // Fill the data vector with pixel RGB values.
    image_data.data = image_buffer
        .pixels()
        .into_iter()
        .map(|pixel: &image::Rgb<u8>| -> Pixel { Pixel::new(pixel.0[0], pixel.0[1], pixel.0[2]) })
        .collect::<Vec<Pixel>>();

    return Ok(image_data);
}
