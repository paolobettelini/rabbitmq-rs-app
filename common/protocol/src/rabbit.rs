extern crate protocol;

pub type Image = Vec<u8>;

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub enum RabbitMessage {
    LoginRequest(LoginRequestData),
    LoginResponse(LoginResponseData),
    RegisterRequest(RegisterRequestData),
    RegisterResponse(RegisterResponseData),
    GetImage(GetImageData),
    GetImageResponse(GetImageResponseData),
    ShrinkAndUpload(ShrinkAndUploadData),
    ShrinkAndUploadResponse(ShrinkAndUploadResponseData),
    GetTotalImages(GetTotalImagesData),
    GetTotalImagesResponse(GetTotalImagesResponseData),
    Log(LogData),
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
    pub token: Vec<u8>,
    pub index: u16,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub struct ShrinkAndUploadData {
    pub token: Vec<u8>,
    pub image: Image,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub struct GetTotalImagesData {
    pub token: Vec<u8>,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub enum GetTotalImagesResponseData {
    Ok(GetTotalImagesResponseDataOk),
    Err(GetTotalImagesResponseDataErr),
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub struct GetTotalImagesResponseDataOk {
    pub amount: u32,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub enum GetTotalImagesResponseDataErr {
    AuthenticationRequired
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub enum ShrinkAndUploadResponseData {
    Ok,
    Err(ShrinkAndUploadResponseDataErr),
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub enum ShrinkAndUploadResponseDataErr {
    InvalidImage,
    AuthenticationRequired,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub enum GetImageResponseData {
    Ok(GetImageResponseDataOk),
    Err(GetImageResponseDataErr),
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub struct GetImageResponseDataOk {
    pub data: Image
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub enum GetImageResponseDataErr {
    InvalidIndex,
    AuthenticationRequired,
}

#[derive(Protocol, Debug, PartialEq)]
#[protocol(discriminant = "integer")]
pub struct LogData {
    pub message: String
}