use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use opencv::{
    prelude::*,
    videoio::{self, VideoCapture, CAP_ANY},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct PixelData {
    x: i32,
    y: i32,
    r: u8,
    g: u8,
    b: u8,
    timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ScreenFrame {
    width: i32,
    height: i32,
    pixels: Vec<PixelData>,
    frame_id: u64,
}

pub fn client_main(ip: &str) {
    let server_addrs = vec![
        "127.0.0.1:52525",     // localhost
        "localhost:52525",     // localhost 호스트명
        ip,
    ];
        
    println!("🔧 연결 시도할 주소들:");
    for addr in &server_addrs {
        println!("   - {}", addr);
    }
        
    let mut stream = None;
    let mut connected_addr = String::new();
        
    for server_addr in &server_addrs {
        println!("🔄 서버 {}에 연결 중...", server_addr);
                
        match TcpStream::connect(server_addr) {
            Ok(s) => {
                println!("✅ {}에 연결 성공!", server_addr);
                stream = Some(s);
                connected_addr = server_addr.to_string();
                break;
            }
            Err(e) => {
                println!("❌ {} 연결 실패: {}", server_addr, e);
            }
        }
    }
        
    let mut stream = match stream {
        Some(s) => {
            println!("🎉 서버 {}에 연결되었습니다!", connected_addr);
            println!("🚀 최적화된 스트리밍 모드로 화면 캡처를 시작합니다.");
            println!("📺 영상은 서버에서만 표시되며, 클라이언트는 백그라운드에서 전송만 합니다.");
            println!("⏹️  Ctrl+C를 눌러 종료하세요.\n");
            s
        }
        None => {
            eprintln!("❌ 모든 서버 주소 연결 실패");
            eprintln!("💡 다음을 확인해주세요:");
            eprintln!("   1. WSL에서 서버가 실행 중인지 확인");
            eprintln!("   2. 방화벽 설정 확인");
            eprintln!("   3. IP 주소가 올바른지 확인");
            return;
        }
    };
            
    let stream_clone = stream.try_clone().unwrap();
        
    // 서버로부터 메시지를 받는 스레드
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stream_clone);
        let mut buffer = String::new();
                
        loop {
            buffer.clear();
            match reader.read_line(&mut buffer) {
                Ok(0) => {
                    println!("🔌 서버 연결이 종료되었습니다.");
                    tx.send(()).unwrap();
                    break;
                }
                Ok(_) => {
                    // 서버 메시지는 조용히 처리 (성능 최적화)
                    if buffer.trim().contains("처리 완료") {
                        // 간단한 확인 메시지만 출력
                        print!(".");
                        io::stdout().flush().unwrap();
                    }
                }
                Err(e) => {
                    eprintln!("❌ 메시지 받기 오류: {}", e);
                    tx.send(()).unwrap();
                    break;
                }
            }
        }
    });

    // 최적화된 화면 캡처 시작 (화면 표시 없음)
    match start_optimized_capture(&mut stream, &rx) {
        Ok(_) => println!("\n👋 스트리밍을 종료합니다."),
        Err(e) => eprintln!("❌ 화면 캡처 오류: {}", e),
    }
}

fn start_optimized_capture(stream: &mut TcpStream, rx: &mpsc::Receiver<()>) -> opencv::Result<()> {
    // 웹캠 초기화
    let mut cap = VideoCapture::new(0, CAP_ANY)?;
    
    if !cap.is_opened()? {
        eprintln!("❌ 카메라를 열 수 없습니다. 웹캠이 연결되어 있는지 확인하세요.");
        return Ok(());
    }

    // 최적화된 캡처 해상도 설정 (성능 우선)
    cap.set(videoio::CAP_PROP_FRAME_WIDTH, 320.0)?;  // 해상도 낮춤
    cap.set(videoio::CAP_PROP_FRAME_HEIGHT, 240.0)?; // 해상도 낮춤
    cap.set(videoio::CAP_PROP_FPS, 15.0)?;           // FPS 제한

    let mut frame = Mat::default();
    let mut frame_id = 0u64;
    let mut last_frame_time = Instant::now();
    let mut last_stats_time = Instant::now();
    let frame_interval = Duration::from_millis(66); // 약 15 FPS
    let mut total_pixels_sent = 0u64;
    let mut frames_sent = 0u64;

    println!("📹 최적화된 스트리밍 시작! (화면 표시 없음)");
    println!("📊 성능 통계:");
    println!("   - 해상도: 320x240");
    println!("   - 프레임율: ~15 FPS");
    println!("   - UI 오버헤드: 없음");
    println!();

    loop {
        // 연결 상태 확인
        if rx.try_recv().is_ok() {
            break;
        }

        // 프레임 속도 제한
        if last_frame_time.elapsed() < frame_interval {
            thread::sleep(Duration::from_millis(5));
            continue;
        }

        // 프레임 캡처
        cap.read(&mut frame)?;
        if frame.empty() {
            continue;
        }

        last_frame_time = Instant::now();
        frame_id += 1;
        frames_sent += 1;

        // 전체 픽셀 데이터 전송 (최적화된 버전)
        let screen_frame = get_optimized_frame(&frame, frame_id)?;
        total_pixels_sent += screen_frame.pixels.len() as u64;
        
        // JSON으로 직렬화하여 서버로 전송
        match serde_json::to_string(&screen_frame) {
            Ok(json_data) => {
                if let Err(e) = writeln!(stream, "{}", json_data) {
                    eprintln!("❌ 데이터 전송 오류: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("❌ JSON 직렬화 오류: {}", e);
                continue;
            }
        }

        // 성능 통계 출력 (5초마다)
        if last_stats_time.elapsed() >= Duration::from_secs(5) {
            let avg_fps = frames_sent as f64 / last_stats_time.elapsed().as_secs_f64();
            let avg_pixels_per_sec = total_pixels_sent as f64 / last_stats_time.elapsed().as_secs_f64();
            
            println!("📊 성능 통계 (최근 5초):");
            println!("   - 평균 FPS: {:.1}", avg_fps);
            println!("   - 전송 픽셀/초: {:.0}", avg_pixels_per_sec);
            println!("   - 총 프레임: {}", frame_id);
            println!("   - 메모리 효율: 화면 표시 없음으로 최적화됨");
            println!();
            
            // 통계 리셋
            last_stats_time = Instant::now();
            frames_sent = 0;
            total_pixels_sent = 0;
        }
    }

    println!("📊 최종 통계:");
    println!("   - 총 전송 프레임: {}", frame_id);
    println!("   - 최적화 효과: UI 렌더링 오버헤드 제거로 성능 향상");

    Ok(())
}

fn get_optimized_frame(frame: &Mat, frame_id: u64) -> opencv::Result<ScreenFrame> {
    let height = frame.rows();
    let width = frame.cols();
    let mut pixels = Vec::with_capacity((width * height) as usize); // 메모리 최적화
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // 모든 픽셀 데이터 읽기 (최적화된 버전)
    for y in 0..height {
        for x in 0..width {
            // 픽셀 값 읽기 (BGR 형식이므로 순서 주의)
            let pixel = frame.at_2d::<opencv::core::Vec3b>(y, x)?;
            
            pixels.push(PixelData {
                x,
                y,
                r: pixel[2], // BGR -> RGB 변환
                g: pixel[1],
                b: pixel[0],
                timestamp,
            });
        }
    }

    Ok(ScreenFrame {
        width,
        height,
        pixels,
        frame_id,
    })
}

// 키보드 입력 처리 (백그라운드에서 종료 감지)
#[allow(dead_code)]
fn check_exit_condition() -> bool {
    // 실제 구현에서는 더 정교한 종료 조건 확인 가능
    // 현재는 Ctrl+C로만 종료 가능
    false
}

// 네트워크 상태 모니터링
#[allow(dead_code)]
fn monitor_network_performance(stream: &TcpStream) {
    match stream.peer_addr() {
        Ok(addr) => {
            println!("🌐 연결된 서버: {}", addr);
        }
        Err(_) => {
            println!("⚠️ 네트워크 상태 확인 불가");
        }
    }
}

// 메모리 사용량 최적화를 위한 헬퍼 함수
#[allow(dead_code)]
fn optimize_memory_usage() {
    // 가비지 컬렉션 힌트 (Rust에서는 자동이므로 실제로는 불필요)
    println!("🧹 메모리 최적화: 자동 메모리 관리 활성화됨");
}
