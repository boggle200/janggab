// this code must working on wsl

// both width and height windows and wsl must be same

// --- 새로운 상수 추가: 원하는 해상도 설정 ---
const TARGET_WIDTH: usize = 320;
const TARGET_HEIGHT: usize = 240;

fn main() {
    let a = janggab::webcam::Wsl::new(320, 240);
    println!("{:?}", a);
}
