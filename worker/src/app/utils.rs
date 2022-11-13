use image::{error::ImageResult, DynamicImage};
use sha2::Digest;

pub fn hash(data: &Vec<u8>) -> Vec<u8> {
    let res = sha2::Sha256::digest(data);

    // let x: [u8; 32] = res.as_slice().try_into().unwrap();

    res.as_slice().to_vec()
}

pub fn load_image(data: &Vec<u8>) -> ImageResult<DynamicImage> {
    image::load_from_memory(data)
}

// DynamicImage::resize;

pub fn is_mail_valid(mail: &str) -> bool {
    true
}

pub fn is_username_valid(mail: &str) -> bool {
    true
}

pub fn generate_random_token() -> Vec<u8> {
    use rand::{thread_rng, RngCore};

    let mut data = vec![0; 256];
    rand::thread_rng().fill_bytes(&mut data);

    data
}
