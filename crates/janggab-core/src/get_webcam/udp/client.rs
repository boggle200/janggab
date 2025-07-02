use std::net::UdpSocket;
use std::time::{Duration, Instant};
use std::thread;
use anyhow::{Result, Context};
use std::sync::atomic::{AtomicUsize, Ordering};

// OpenCV 관련 모듈 임포트
use opencv::{
    prelude::*,
    videoio,
    core,
    imgproc, // imgproc 모듈 추가 (resize 함수 사용을 위함)
};

const CONNECTION_TIMEOUT_SECS: u64 = 5;

const MAX_PAYLOAD_SIZE: usize = 1400;
const HEADER_SIZE: usize = 4 + 2 + 2;
const CHUNK_DATA_SIZE: usize = MAX_PAYLOAD_SIZE - HEADER_SIZE;

static FRAME_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn client_main(server_ip: &str, img_width: i32, img_height: i32) -> Result<()> {
    let server_ip: &str = &format!("{}:52525", server_ip);
    let socket = UdpSocket::bind("0.0.0.0:0")
        .context("UDP 소켓 바인딩 실패")?;
    socket.set_read_timeout(Some(Duration::from_secs(1)))
        .context("소켓 읽기 타임아웃 설정 실패")?;

    println!("🔄 서버 {}에 연결 시도 중...", server_ip);

    let mut connected = false;
    let start = Instant::now();
    while !connected && start.elapsed().as_secs() < CONNECTION_TIMEOUT_SECS {
        socket.send_to(b"HELLO", server_ip)
            .context("HELLO 메시지 전송 실패")?;
        let mut buf = [0u8; 1024];

        match socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                let msg = String::from_utf8_lossy(&buf[..size]);
                if msg.trim() == "CONNECTED" {
                    println!("✅ 서버에 연결됨: {}", addr);
                    connected = true;
                    break;
                }
            }
            Err(_) => {
                println!("⏳ 연결 대기중...");
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

    if !connected {
        println!("❌ 서버 연결 실패. 종료합니다.");
        return Ok(());
    }

    println!("💬 이미지 데이터를 분할하여 쉬지 않고 전송합니다. 종료하려면 Ctrl+C");

    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)
        .context("카메라 초기화 실패")?;
    if !videoio::VideoCapture::is_opened(&cam)? {
        anyhow::bail!("카메라를 열 수 없습니다! 다른 카메라 인덱스를 시도하거나 카메라 연결을 확인하세요.");
    }

    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame)
            .context("프레임 읽기 실패")?;

        if frame.empty() {
            println!("⚠️ 빈 프레임 수신. 다시 시도합니다.");
            thread::sleep(Duration::from_millis(100));
            continue;
        }

        // --- 추가된 부분: 프레임 크기 조절 ---
        let mut resized_frame = Mat::default();
        let dsize = core::Size::new(img_width, img_height);
        imgproc::resize(&frame, &mut resized_frame, dsize, 0.0, 0.0, imgproc::INTER_LINEAR)
            .context("프레임 크기 조절 실패")?;
        // ------------------------------------

        let rows = resized_frame.rows() as usize;
        let cols = resized_frame.cols() as usize;
        let channels = resized_frame.channels() as usize;
        let total_image_size = rows * cols * channels;

        // 수정: 원본 frame 대신 resized_frame의 데이터 사용
        let image_data = resized_frame.data_bytes()
            .context("이미지 데이터를 바이트로 변환 실패")?;

        let current_frame_id = FRAME_ID_COUNTER.fetch_add(1, Ordering::SeqCst) as u32;

        let num_chunks = (image_data.len() + CHUNK_DATA_SIZE - 1) / CHUNK_DATA_SIZE;
        println!("🚀 새 프레임 (ID: {} / {}x{}x{} / 총 {} 바이트) {}개 청크로 분할 전송 시작...",
                 current_frame_id, cols, rows, channels, total_image_size, num_chunks);

        let chunks_to_send_at_once = 6;

        for i_start in (0..num_chunks).step_by(chunks_to_send_at_once) {
            for j in 0..chunks_to_send_at_once {
                let current_chunk_index = i_start + j;
                if current_chunk_index >= num_chunks {
                    break; // No more chunks to send
                }

                let start_index = current_chunk_index * CHUNK_DATA_SIZE;
                let end_index = (start_index + CHUNK_DATA_SIZE).min(image_data.len());
                let chunk_data = &image_data[start_index..end_index];

                let mut packet_buffer = vec![0u8; HEADER_SIZE + chunk_data.len()];

                packet_buffer[0..4].copy_from_slice(&current_frame_id.to_be_bytes());
                packet_buffer[4..6].copy_from_slice(&(num_chunks as u16).to_be_bytes());
                packet_buffer[6..8].copy_from_slice(&(current_chunk_index as u16).to_be_bytes());

                packet_buffer[HEADER_SIZE..].copy_from_slice(chunk_data);

                match socket.send_to(&packet_buffer, server_ip) {
                    Ok(bytes_sent) => {
                        println!("  ➡️ 청크 {}/{} 전송됨 ({} 바이트)", current_chunk_index + 1, num_chunks, bytes_sent);
                    }
                    Err(e) => {
                        eprintln!("⚠️ 청크 전송 실패 (ID: {}, {}/{}): {}", current_frame_id, current_chunk_index + 1, num_chunks, e);
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            }
            thread::sleep(Duration::from_millis(1));
        }

        println!("✅ 프레임 (ID: {}) 전송 완료.\n", current_frame_id);

        thread::sleep(Duration::from_millis(30));
    }
}