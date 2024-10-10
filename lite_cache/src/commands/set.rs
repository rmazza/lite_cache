#[derive(Debug)]
pub struct SetCommand {
    pub key: String,
    pub value: String,
    nx: bool,
    xx: bool,
}

impl SetCommand {
    pub fn new(key: String, value: String) -> SetCommand {
        SetCommand { key, value, nx: false, xx: false }
    }
}