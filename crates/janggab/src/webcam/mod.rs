pub struct Windows;

pub struct Wsl;

impl Windows {
    pub fn new(ip: &str, width: i32, height: i32) {
        janggab_core::get_webcam::udp::client::client_main(ip, width, height).unwrap();
    }
}

impl Wsl {
    pub fn new(width: usize, height: usize) -> Vec<u8> {
        let jgb_full_data = janggab_core::get_webcam::udp::server::server_main(width, height).unwrap();
        jgb_full_data
    }
}
