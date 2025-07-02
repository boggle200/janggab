use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::path::Path;
use chrono::Local;
use serde::{Deserialize, Serialize};
use image::{ImageBuffer, Rgb, RgbImage};
use opencv::{
    prelude::*,
    highgui,
    core::Size,
    imgproc,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PixelData {
    x: i32,
    y: i32,
    r: u8,
    g: u8,
    b: u8,
    timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ScreenFrame {
    width: i32,
    height: i32,
    pixels: Vec<PixelData>,
    frame_id: u64,
}

type ClientMap = Arc<Mutex<HashMap<String, TcpStream>>>;
type FrameBuffer = Arc<Mutex<HashMap<String, ScreenFrame>>>;

pub fn server_main() {
    let server_addr = "0.0.0.0:52525";
    
    println!("🔧 네트워크 디버깅 정보:");
    println!("   - 서버 바인딩 주소: {}", server_addr);
    
    // 저장 디렉토리 생성
    let save_dir = "captured_frames";
    if !Path::new(save_dir).exists() {
        std::fs::create_dir_all(save_dir).expect("디렉토리 생성 실패");
    }
    println!("📁 프레임 저장 디렉토리: {}", save_dir);
    
    match std::process::Command::new("hostname").arg("-I").output() {
        Ok(output) => {
            let ips = String::from_utf8_lossy(&output.stdout);
            println!("   - 사용 가능한 IP 주소들: {}", ips.trim());
        }
        Err(_) => println!("   - IP 주소 확인 실패"),
    }
    
    let listener = TcpListener::bind(server_addr).expect("서버 바인딩 실패");
    println!("🚀 실시간 이미지 스트리밍 서버가 {}에서 시작되었습니다", server_addr);
    println!("📡 클라이언트 연결을 기다리는 중...");
    println!("🖥️ 클라이언트가 연결되면 실시간 영상이 표시됩니다\n");
    
    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));
    let frame_buffer: FrameBuffer = Arc::new(Mutex::new(HashMap::new()));
    
    for (client_id, stream) in listener.incoming().enumerate() {
        match stream {
            Ok(stream) => {
                let client_addr = stream.peer_addr().unwrap();
                let client_name = format!("Client_{}", client_id);
                
                println!("✅ 새 클라이언트 연결: {} ({})", client_name, client_addr);
                
                let clients_clone = Arc::clone(&clients);
                let frame_buffer_clone = Arc::clone(&frame_buffer);
                let stream_clone = stream.try_clone().unwrap();
                
                clients_clone.lock().unwrap().insert(client_name.clone(), stream_clone);
                
                thread::spawn(move || {
                    handle_client(stream, client_name, clients_clone, frame_buffer_clone);
                });
            }
            Err(e) => {
                eprintln!("❌ 클라이언트 연결 오류: {}", e);
            }
        }
    }
}

fn handle_client(stream: TcpStream, client_name: String, clients: ClientMap, frame_buffer: FrameBuffer) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut buffer = String::new();
    
    loop {
        buffer.clear();
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                println!("🔌 {} 연결 종료", client_name);
                clients.lock().unwrap().remove(&client_name);
                frame_buffer.lock().unwrap().remove(&client_name);
                
                // 클라이언트 연결 종료시 해당 창 닫기
                let window_name = format!("Live Stream - {}", client_name);
                highgui::destroy_window(&window_name).unwrap_or_default();
                break;
            }
            Ok(_) => {
                let message = buffer.trim();
                let timestamp = Local::now().format("%H:%M:%S");
                
                // JSON 데이터인지 확인하고 파싱 시도
                if let Ok(screen_frame) = serde_json::from_str::<ScreenFrame>(message) {
                    println!("📸 [{}] {} 실시간 프레임 수신: {}x{}, {} 픽셀, 프레임 ID: {}", 
                            timestamp, client_name, screen_frame.width, screen_frame.height, 
                            screen_frame.pixels.len(), screen_frame.frame_id);
                    
                    // 프레임 버퍼에 저장
                    frame_buffer.lock().unwrap().insert(client_name.clone(), screen_frame.clone());
                    
                    // 실시간으로 이미지 표시
                    if let Err(e) = display_frame_realtime(&screen_frame, &client_name) {
                        eprintln!("❌ 실시간 표시 오류: {}", e);
                    }
                    
                    // 이미지 저장 (선택사항 - 매 10프레임마다만 저장하여 성능 최적화)
                    if screen_frame.frame_id % 10 == 0 {
                        if let Err(e) = assemble_and_save_image(&screen_frame, &client_name) {
                            eprintln!("❌ 이미지 저장 오류: {}", e);
                        }
                    }
                    
                    // 클라이언트에게 확인 메시지 전송
                    let response = format!("프레임 {} 실시간 표시 완료", screen_frame.frame_id);
                    broadcast_message(&clients, &format!("[서버] {}: {}", client_name, response));
                } else {
                    // 일반 텍스트 메시지 처리
                    println!("💬 [{}] {}: {}", timestamp, client_name, message);
                    broadcast_message(&clients, &format!("[{}] {}: {}", timestamp, client_name, message));
                }
            }
            Err(e) => {
                eprintln!("❌ {} 메시지 읽기 오류: {}", client_name, e);
                clients.lock().unwrap().remove(&client_name);
                frame_buffer.lock().unwrap().remove(&client_name);
                
                // 연결 오류시 창 닫기
                let window_name = format!("Live Stream - {}", client_name);
                highgui::destroy_window(&window_name).unwrap_or_default();
                break;
            }
        }
    }
}

fn display_frame_realtime(screen_frame: &ScreenFrame, client_name: &str) -> opencv::Result<()> {
    let width = screen_frame.width;
    let height = screen_frame.height;
    
    // OpenCV Mat 생성 (BGR 형식)
    let mut mat = Mat::zeros(height, width, opencv::core::CV_8UC3)?.to_mat()?;
    
    // 픽셀 데이터를 Mat에 복사
    for pixel_data in &screen_frame.pixels {
        let x = pixel_data.x;
        let y = pixel_data.y;
        
        // 좌표가 이미지 범위 내에 있는지 확인
        if x >= 0 && x < width && y >= 0 && y < height {
            let pixel = mat.at_2d_mut::<opencv::core::Vec3b>(y, x)?;
            // RGB -> BGR 변환하여 저장
            pixel[0] = pixel_data.b; // B
            pixel[1] = pixel_data.g; // G
            pixel[2] = pixel_data.r; // R
        }
    }
    
    // 창 이름 설정
    let window_name = format!("Live Stream - {}", client_name);
    
    // 창이 처음 생성되는 경우 창 설정
    if highgui::get_window_property(&window_name, highgui::WND_PROP_VISIBLE)? < 0.0 {
        highgui::named_window(&window_name, highgui::WINDOW_AUTOSIZE)?;
        println!("🖥️ 새 실시간 스트림 창 생성: {}", window_name);
    }
    
    // 이미지가 너무 작은 경우 크기 조정
    let display_mat = if width < 320 || height < 240 {
        let mut resized_mat = Mat::default();
        let scale_x = (640.0 / width as f64).max(2.0);
        let scale_y = (480.0 / height as f64).max(2.0);
        let scale = scale_x.min(scale_y);
        
        let new_size = Size::new(
            (width as f64 * scale) as i32,
            (height as f64 * scale) as i32
        );
        
        imgproc::resize(&mat, &mut resized_mat, new_size, 0.0, 0.0, imgproc::INTER_NEAREST)?;
        resized_mat
    } else {
        mat
    };
    
    // 실시간으로 이미지 표시
    highgui::imshow(&window_name, &display_mat)?;
    
    // 1ms 대기 (실시간 업데이트를 위해)
    let key = highgui::wait_key(1)?;
    
    // ESC 키나 'q' 키가 눌리면 해당 클라이언트 창만 닫기
    if key == 27 || key == 'q' as i32 {
        println!("🔲 사용자 요청으로 {} 스트림 창을 닫습니다", client_name);
        highgui::destroy_window(&window_name)?;
    }
    
    Ok(())
}

fn assemble_and_save_image(screen_frame: &ScreenFrame, client_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let width = screen_frame.width as u32;
    let height = screen_frame.height as u32;
    
    // 이미지 버퍼 생성
    let mut img: RgbImage = ImageBuffer::new(width, height);
    
    // 받은 픽셀 데이터로 이미지 채우기
    for pixel_data in &screen_frame.pixels {
        let x = pixel_data.x as u32;
        let y = pixel_data.y as u32;
        
        // 좌표가 이미지 범위 내에 있는지 확인
        if x < width && y < height {
            let pixel = img.get_pixel_mut(x, y);
            *pixel = Rgb([pixel_data.r, pixel_data.g, pixel_data.b]);
        }
    }
    
    // 파일명 생성 (타임스탬프 포함)
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("captured_frames/{}_{}_frame_{}.png", 
                          client_name, timestamp, screen_frame.frame_id);
    
    // 이미지 저장
    img.save(&filename)?;
    println!("💾 이미지 저장됨: {}", filename);
    
    // 최근 10개 프레임만 유지
    cleanup_old_frames(client_name, 10)?;
    
    Ok(())
}

fn cleanup_old_frames(client_name: &str, keep_count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let dir = Path::new("captured_frames");
    if !dir.exists() {
        return Ok(());
    }
    
    let mut files: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_name()
                .to_str()
                .map(|name| name.starts_with(client_name) && name.ends_with(".png"))
                .unwrap_or(false)
        })
        .collect();
    
    // 파일을 수정 시간으로 정렬
    files.sort_by_key(|entry| {
        entry.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    
    // 오래된 파일들 삭제
    if files.len() > keep_count {
        for file_entry in files.iter().take(files.len() - keep_count) {
            if let Err(e) = std::fs::remove_file(file_entry.path()) {
                eprintln!("⚠️ 파일 삭제 실패: {}", e);
            }
        }
    }
    
    Ok(())
}

fn broadcast_message(clients: &ClientMap, message: &str) {
    let mut clients_lock = clients.lock().unwrap();
    let mut disconnected_clients = Vec::new();
    
    for (client_name, stream) in clients_lock.iter_mut() {
        if let Err(_) = writeln!(stream, "{}", message) {
            disconnected_clients.push(client_name.clone());
        }
    }
    
    // 연결이 끊어진 클라이언트 제거
    for client_name in disconnected_clients {
        clients_lock.remove(&client_name);
        
        // 연결 끊어진 클라이언트의 창도 닫기
        let window_name = format!("Live Stream - {}", client_name);
        highgui::destroy_window(&window_name).unwrap_or_default();
    }
}