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

#[derive(Clone)]
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
    let mut image =
        read_image("/home/linus/development/rust/pixel_sort/testing/test_image.jpg").unwrap();

    print!("extracting spans:");
    let mut spans = extract_spans(
        &image,
        Filter {
            kind: HslComponent::Luminosity,
            bottom: 0.55,
            top: 1.0,
        },
    );
    print!(" done({})\n", spans.len());

    println!("sorting spans");
    let sorted_spans = sort_spans(&mut spans, HslComponent::Luminosity);

    println!("inserting spans");
    insert_spans(&mut image, sorted_spans, Direction::Right);

    write_image(
        "/home/linus/development/rust/pixel_sort/testing/output.jpg",
        image,
    );
}

// ================================================================================================
// span fuckery
fn extract_spans(image: &ImageData, filter: Filter) -> Vec<Span> {
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

    for (i, pixel) in image.data.iter().enumerate() {
        // check if we need to span break
        if !check_eligibility(*pixel, filter) || calculate_position(i as u32, image.width).x == 0 {
            if current_span.data.len() > 1 {
                spans.push(current_span);
            }

            current_span = Span {
                position: Position { x: 0, y: 0 },
                data: Vec::new(),
            };
        }

        if check_eligibility(*pixel, filter) {
            if current_span.data.len() == 0 {
                current_span.position = calculate_position(i as u32, image.width);
            }
            current_span.data.push(*pixel);
        }
    }

    return spans;
}

fn insert_spans(image: &mut ImageData, spans: Vec<Span>, direction: Direction) -> &ImageData {
    for span in spans {
        let mut current_pixel_id: u32 = calculate_id(
            Position {
                x: span.position.x,
                y: span.position.y,
            },
            image.width,
        );

        for pixel in span.data {
            image.data[current_pixel_id as usize] = pixel;
            current_pixel_id = next_pixel_id(current_pixel_id, image.width, direction)
        }
    }
    return image;
}

//TODO merge sort_span into sort spans
fn sort_span(mut span: Span, sort_type: HslComponent) -> Span {
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

    span.data = sort_pixels(span.data, sort_type);
    span
}

fn sort_spans(spans: &mut Vec<Span>, sort_type: HslComponent) -> Vec<Span> {
    let mut sorted_spans: Vec<Span> = Vec::new();
    for span in spans {
        sorted_spans.push(sort_span(span.clone(), sort_type).clone());
    }

    return sorted_spans;
}

// ================================================================================================
// implement HSL calculation according to
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

//TODO implement wraping
const fn next_pixel_id(id: u32, width: u32, direction: Direction) -> u32 {
    match direction {
        Direction::Right => id + 1,
        Direction::Left => id - 1,
        Direction::Down => id + width,
        Direction::Up => id - width,
    }
}

// ================================================================================================
// file interaction functions using
// https://docs.rs/image/latest/image

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
                        image_data.data[id].rgb.r,
                        image_data.data[id].rgb.g,
                        image_data.data[id].rgb.b,
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
                image_data.data.push(Pixel::new(
                    image_buffer.get_pixel(x, y).0[0],
                    image_buffer.get_pixel(x, y).0[1],
                    image_buffer.get_pixel(x, y).0[2],
                ));
                x = x + 1;
            }
            y = y + 1;
        }
    }

    return Ok(image_data);
}
