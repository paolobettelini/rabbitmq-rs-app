use image::{DynamicImage, ImageFormat};

pub fn image_to_format(image: DynamicImage, format: ImageFormat) -> Vec<u8> {
    use std::io::{Cursor, Read, Seek, SeekFrom, Write};

    let color = image.color();
    let width = image.width();
    let height = image.height();

    // Implements `Seek` and `Write`
    let mut cursor = Cursor::new(Vec::new());

    image::write_buffer_with_format(
        &mut cursor,
        &mut image.into_bytes(),
        width,
        height,
        color,
        format,
    );

    // Read result
    cursor.seek(SeekFrom::Start(0)).unwrap();
    let mut buffer = Vec::new();
    cursor.read_to_end(&mut buffer).unwrap();

    buffer
}

pub fn hash(data: &Vec<u8>) -> Vec<u8> {
    use sha2::Digest;

    let res = sha2::Sha256::digest(data);

    // let x: [u8; 32] = res.as_slice().try_into().unwrap();

    res.as_slice().to_vec()
}

pub fn generate_random_token() -> Vec<u8> {
    use rand::{thread_rng, RngCore};

    let mut data = vec![0; 32];
    rand::thread_rng().fill_bytes(&mut data);

    data
}

pub fn is_mail_valid(mail: &str) -> bool {
    true
}

pub fn is_username_valid(mail: &str) -> bool {
    true
}
