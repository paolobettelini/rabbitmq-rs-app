use lazy_regex::regex;

pub fn validate_email(value: &str) -> bool {
    let regex = regex!(r"^\w+([\.-]?\w+)*@\w+([\.-]?\w+)*(\.\w{2,3})+$");

    regex.is_match(value)
}

pub fn validate_password(value: &str) -> bool {
    //let regex = regex!(r"^(?=.*[A-Za-z])(?=.*\d).{8,}$");

    //regex.is_match(value)
    false
}

pub fn validate_username(value: &str) -> bool {
    let regex = regex!(r"^[a-zA-Z0-9._]{4,20}$");

    regex.is_match(value)
}