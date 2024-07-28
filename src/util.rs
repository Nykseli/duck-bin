use std::io::Read;

pub fn rand_string() -> String {
    let mut file_buf = [0u8; 64];
    let mut rand_file = std::fs::File::open("/dev/random").unwrap();
    let _ = rand_file.read(&mut file_buf).unwrap();
    let user_secret: Vec<u8> = file_buf
        .iter()
        .filter(|b| b.is_ascii_alphanumeric())
        .copied()
        .collect();
    String::from_utf8(user_secret).unwrap()
}
