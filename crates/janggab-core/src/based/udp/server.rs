// server.rs
use std::net::UdpSocket;
use std::time::Duration;

#[allow(warnings)]
pub fn server_main() {
    let server_addr = "0.0.0.0:52525";
    let socket = UdpSocket::bind(server_addr).expect("서버 바인딩 실패");
    socket.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

    println!("🚀 UDP 서버가 {}에서 시작되었습니다", server_addr);
    println!("📡 클라이언트 연결을 기다리는 중...\n");

    let mut buf = [0u8; 1024];
    let mut client_addr = None;

    // 초기 연결 대기
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                let msg = String::from_utf8_lossy(&buf[..size]);
                println!("📨 {}로부터 메시지 수신: {}", addr, msg);

                if msg.trim() == "HELLO" {
                    println!("✅ 클라이언트 연결됨: {}", addr);
                    client_addr = Some(addr);
                    // 연결 확인 응답
                    socket.send_to(b"CONNECTED", addr).unwrap();
                    break;
                }
            }
            Err(_) => {
                println!("⏳ 클라이언트 연결 대기중...");
            }
        }
    }

    // 연결 후 데이터 수신
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                let msg = String::from_utf8_lossy(&buf[..size]);
                println!("💬 [{}] {}", addr, msg.trim());
            }
            Err(_) => continue,
        }
    }
}
