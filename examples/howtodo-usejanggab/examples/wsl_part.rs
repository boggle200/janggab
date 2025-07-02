// this code must working on wsl

// both width and height windows and wsl must be same

// --- 새로운 상수 추가: 원하는 해상도 설정 ---
const TARGET_WIDTH: usize = 320;
const TARGET_HEIGHT: usize = 240;

fn main() {
    //let a = glove_core::get_webcam::udp::server::server_main();
    let _ = janggab::webcam::Wsl::new(TARGET_WIDTH, TARGET_HEIGHT);
}
