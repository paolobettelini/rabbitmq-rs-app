use lazy_regex::regex;

pub fn validate_email(value: &str) -> bool {
    let regex = regex!(r"^\w+([\.-]?\w+)*@\w+([\.-]?\w+)*(\.\w{2,3})+$");

    regex.is_match(value)
}

pub fn validate_password(value: &str) -> bool {
    let regex1 = regex!(r"\d+");
    let regex2 = regex!(r"[A-Za-z]+");
    let regex3 = regex!(r"^.{8,}$");

    regex1.is_match(value) && regex2.is_match(value) && regex3.is_match(value)
}

pub fn validate_username(value: &str) -> bool {
    let regex = regex!(r"^[a-zA-Z0-9._]{4,20}$");

    regex.is_match(value)
}