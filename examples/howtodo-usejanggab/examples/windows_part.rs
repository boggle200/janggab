// this code must working on windows

/*
if you need to find your ip, use under code.

hostname -I

then your ip is showing.
*/

// --- 새로운 상수 추가: 원하는 해상도 설정 ---
const TARGET_WIDTH: i32 = 320;
const TARGET_HEIGHT: i32 = 240;

fn main() {
    let _ = janggab::webcam::Windows::new("your ip", TARGET_WIDTH, TARGET_HEIGHT);
}
