extern crate protocol;

use protocol::{Enum, Parcel};

/* not worky :(
macro_rules! declare_packets {
    (
        $(
            $( #[$meta:meta] )*
            $vis:vis struct $name:ident $body:block
        )*
    ) => {
        $(
            #[derive(Protocol, Debug, PartialEq)]
            #[protocol(discriminant = "integer")]
            $vis struct $name $block
        )*
    };
}*/

pub type Image = Vec<u8>;

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub enum RabbitMessage {
    LoginRequest(LoginRequestData),
    LoginResponse(LoginResponseData),
    RegisterRequest(RegisterRequestData),
    RegisterResponse(RegisterResponseData),
    GetImage(GetImageData),
    ShrinkAndUpload(ShrinkAndUploadData),
    GetTotalImages(GetTotalImagesData),
    GetTotalImagesResponse(GetTotalImagesResponseData),
    // get total images response
    ErrorResponse(ErrorResponseData),
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub struct LoginRequestData {
    pub username: String,
    pub password: Vec<u8>,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub enum LoginResponseData {
    Ok(LoginResponseDataOk),
    Err(LoginResponseDataErr),
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub struct LoginResponseDataOk {
    pub token: Vec<u8>,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub enum LoginResponseDataErr {
    NotFound,
    WrongPassword,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub struct RegisterRequestData {
    pub mail: String,
    pub username: String,
    pub password: Vec<u8>,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub enum RegisterResponseData {
    Ok(RegisterResponseDataOk),
    Err(RegisterResponseDataErr),
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub struct RegisterResponseDataOk {
    pub token: Vec<u8>,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub enum RegisterResponseDataErr {
    UsernameAlreadyExists,
    MailAlreadyExists
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub struct GetImageData {
    pub username: String,
    pub token: Vec<u8>,
    pub index: u16,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub struct ShrinkAndUploadData {
    pub username: String,
    pub token: Vec<u8>,
    pub image: Image,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub struct GetTotalImagesData {
    pub username: String,
    pub token: Vec<u8>,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub struct GetTotalImagesResponseData {
    pub amount: u32,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub enum ErrorResponseData {
    AuthenticationRequired,
    UnknownUsername,
}