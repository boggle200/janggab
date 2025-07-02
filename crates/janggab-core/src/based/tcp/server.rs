use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use chrono::Local;

#[allow(dead_code)]
type ClientMap = Arc<Mutex<HashMap<String, TcpStream>>>;

#[allow(dead_code)]
pub fn server_main() {
    // WSL의 default IP 주소를 여기에 입력하세요
    // 예: "172.20.240.1:8080" 또는 ip route | grep default 결과의 IP 사용
    let server_addr = "0.0.0.0:52525"; // 모든 인터페이스에서 연결 허용
    
    println!("🔧 네트워크 디버깅 정보:");
    println!("   - 서버 바인딩 주소: {}", server_addr);
    
    // 현재 네트워크 인터페이스 정보 출력
    match std::process::Command::new("hostname").arg("-I").output() {
        Ok(output) => {
            let ips = String::from_utf8_lossy(&output.stdout);
            println!("   - 사용 가능한 IP 주소들: {}", ips.trim());
        }
        Err(_) => println!("   - IP 주소 확인 실패"),
    }
    
    let listener = TcpListener::bind(server_addr).expect("서버 바인딩 실패");
    println!("🚀 채팅 서버가 {}에서 시작되었습니다", server_addr);
    println!("📡 클라이언트 연결을 기다리는 중...\n");
    
    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));
    
    for (client_id, stream) in listener.incoming().enumerate() {
        match stream {
            Ok(stream) => {
                let client_addr = stream.peer_addr().unwrap();
                let client_name = format!("Client_{}", client_id);
                
                println!("✅ 새 클라이언트 연결: {} ({})", client_name, client_addr);
                
                let clients_clone = Arc::clone(&clients);
                let stream_clone = stream.try_clone().unwrap();
                
                // 클라이언트를 맵에 추가
                clients_clone.lock().unwrap().insert(client_name.clone(), stream_clone);
                
                thread::spawn(move || {
                    handle_client(stream, client_name, clients_clone);
                });
            }
            Err(e) => {
                eprintln!("❌ 클라이언트 연결 오류: {}", e);
            }
        }
    }
}

fn handle_client(stream: TcpStream, client_name: String, clients: ClientMap) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut buffer = String::new();
    
    loop {
        buffer.clear();
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                // 클라이언트 연결 종료
                println!("🔌 {} 연결 종료", client_name);
                clients.lock().unwrap().remove(&client_name);
                break;
            }
            Ok(_) => {
                let message = buffer.trim();
                let timestamp = Local::now().format("%H:%M:%S");
                
                // 서버에서 실시간 대화 정보 출력
                println!("💬 [{}] {}: {}", timestamp, client_name, message);
                
                // 모든 클라이언트에게 메시지 브로드캐스트
                broadcast_message(&clients, &format!("[{}] {}: {}", timestamp, client_name, message));
            }
            Err(e) => {
                eprintln!("❌ {} 메시지 읽기 오류: {}", client_name, e);
                clients.lock().unwrap().remove(&client_name);
                break;
            }
        }
    }
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
    }
}