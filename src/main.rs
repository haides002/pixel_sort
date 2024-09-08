struct ImageData {
    height: u32,
    width: u32,
    data: Vec<[u8;3]>,
}

fn main() {
    let image = read_image("/home/linus/Dev/rust/pixel_sort/test_image.jpg").unwrap();
    write_image("/home/linus/Dev/rust/pixel_sort/output.jpg", image);
}


// ======== Writing the edited Image ======== //
fn write_image(file_path: &str, image_data: ImageData) {
    // create an imagebuffer with the size of our raw data
    let mut image_buffer = image::DynamicImage::new(image_data.width, image_data.height, image::ColorType::Rgb8);

    // write into the image buffer from the raw data
    {
        let mut y:u32 = 0;
        let mut id:usize = 0;
        while y < image_data.height {
            let mut x:u32 = 0;
            while x < image_data.width {
                let pixel = image::Rgb{
                    0: image_data.data[id],
                };

                image_buffer.as_mut_rgb8().unwrap().put_pixel(x, y, pixel);
                x = x + 1;
                id = id + 1;
            }
            y = y +1;
        }
    }

    // write the image buffer to disk
    image_buffer.save(file_path).unwrap();
}



// ============ Reading the File ============ //
fn read_image(file_path: &str) -> Result<ImageData, image::ImageError> {
    // i now haz a image_buffer
    let binding = image::ImageReader::open(file_path)?.decode()?;
    let image_buffer = binding.as_rgb8().expect("Failed to convert image to rgb8 format!");
    
    // create the image data struct & give it width and height
    let mut image_data = ImageData {
        height: image_buffer.height(),
        width: image_buffer.width(),
        data: vec![]
    };

    // fill the data vector with pixel RGB values
    {
        let mut y:u32 = 0;
        while y < image_data.height {
            let mut x:u32 = 0;
            while x < image_data.width {
                image_data.data.push(image_buffer.get_pixel(x,y).0);
                x = x + 1;
            }
            y = y + 1;
        }
    }

    return Ok(image_data);
}
