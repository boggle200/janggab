// client.rs
use std::net::UdpSocket;
use std::time::{Duration, Instant};
use std::thread;

pub fn client_main() {
    // 서버 IP 입력 (예: "172.20.240.1" for WSL)
    // Enter server IP (e.g., "172.20.240.1" for WSL)
    let server_ip = "your_ip";

    let socket = UdpSocket::bind("0.0.0.0:0").expect("소켓 바인딩 실패"); // Failed to bind socket
    socket.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

    println!("🔄 서버 {}에 연결 시도 중...", server_ip); // Attempting to connect to server...

    // 연결 시도 (HELLO 메시지 반복 전송)
    // Connection attempt (repeatedly send HELLO message)
    let mut connected = false;
    let start = Instant::now();
    while !connected && start.elapsed().as_secs() < 10 {
        socket.send_to(b"HELLO", server_ip).unwrap();
        let mut buf = [0u8; 1024];

        match socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                let msg = String::from_utf8_lossy(&buf[..size]);
                if msg.trim() == "CONNECTED" {
                    println!("✅ 서버에 연결됨: {}", addr); // Connected to server:
                    connected = true;
                    break;
                }
            }
            Err(_) => {
                println!("⏳ 연결 대기중..."); // Waiting for connection...
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

    if !connected {
        println!("❌ 서버 연결 실패. 종료합니다."); // Server connection failed. Exiting.
        return;
    }

    println!("💬 데이터를 쉬지 않고 전송합니다. 종료하려면 Ctrl+C"); // Continuously sending data. Press Ctrl+C to exit.

    // 전송할 샘플 데이터 (vec! 매크로를 사용하여 생성)
    // Sample data to send (created using vec! macro)
    let data_to_send: Vec<u8> = vec![
        0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14,
        0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20,
        0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C,
        0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38,
        0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44,
        0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50,
    ];

    // 데이터 전송 루프 (쉬지 않고 반복)
    // Data transmission loop (repeats continuously)
    loop {
        match socket.send_to(&data_to_send, server_ip) {
            Ok(bytes_sent) => {
                println!("➡️ {} 바이트 전송됨.", bytes_sent); // bytes sent.
            }
            Err(e) => {
                eprintln!("⚠️ 데이터 전송 실패: {}", e); // Failed to send data:
                // 오류 발생 시 잠시 대기 후 재시도
                // Wait briefly and retry on error
                thread::sleep(Duration::from_millis(100));
            }
        }
        // CPU 사용률을 줄이기 위해 잠시 대기
        // Wait briefly to reduce CPU usage
        thread::sleep(Duration::from_millis(10));
    }
}
