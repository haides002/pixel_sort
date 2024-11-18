#[derive(Clone, Copy)]
enum HslComponent {
    Hue,
    Saturation,
    Luminosity,
}

struct Span {
    x: u32,
    y: u32,
    data: Vec<Pixel>,
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
    let image =
        read_image("/home/linus/development/rust/pixel_sort/testing/test_image.jpg").unwrap();
    let spans = extract_spans(&image, HslComponent::Luminosity, 0.4, 0.8);
    let sorted_spans = sort_spans(spans, HslComponent::Luminosity);
    write_image(
        "/home/linus/development/rust/pixel_sort/testing/output.jpg",
        image,
    );
}

// ================================================================================================
// span fuckery
fn extract_spans(
    image: &ImageData,
    metric: HslComponent,
    bottom: f32,
    top: f32,
    /* direction:? */
) -> Vec<Span> {
    let mut spans: Vec<Span> = Vec::new();
    let mut current_span: Span = Span {
        x: 0,
        y: 0,
        data: Vec::new(),
    };

    for (i, value) in image.data.iter().enumerate() {
        let amogus: f32 = match metric {
            HslComponent::Hue => todo!("mek good!!!"),
            HslComponent::Luminosity => value.hsl.luminosity,
            HslComponent::Saturation => value.hsl.saturation,
        };

        if amogus > bottom && amogus < top {
            current_span.data.push(value.clone());
            if current_span.data.len() == 0 {
                // when the first pixel of a span is found
                current_span.y = i as u32 / image.width;
                current_span.x = i as u32 % image.width;
            }
        } else {
            if current_span.data.len() > 2 {
                spans.push(current_span);
            }
            current_span = Span {
                x: 0,
                y: 0,
                data: Vec::new(),
            };
        }
    }

    return spans;
}

// this one is ultra ass and absoloutly needs a rework
fn sort_span(p_span: Span, sort_type: HslComponent) -> Span {
    //TODO: make it respect HslComponent
    let mut span: Span = p_span;
    fn quicksort(mut array: &mut Span, low: usize, high: usize) {
        if low < high {
            let pivot: usize = partition(&mut array, low, high);

            quicksort(&mut array, low, pivot - 1);
            quicksort(&mut array, pivot + 1, high);
        }
    }
    fn partition(array: &mut Span, low: usize, high: usize) -> usize {
        let pivot: Pixel = *array.data.last().unwrap();
        let mut i: usize = low;

        for j in low..(high - 1) {
            if array.data[j].hsl.luminosity <= pivot.hsl.luminosity {
                //TODO does not respect sort_type
                array.data.swap(i, j);
                i = i + 1;
            }
        }

        array.data.swap(i, high);
        return i;
    }

    let len = span.data.len();
    quicksort(&mut span, 0, len - 1);
    span
}

fn sort_spans(spans: Vec<Span>, sort_type: HslComponent) -> Vec<Span> {
    let mut sorted_spans: Vec<Span> = Vec::new();
    for span in spans {
        sorted_spans.push(sort_span(span, sort_type));
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
