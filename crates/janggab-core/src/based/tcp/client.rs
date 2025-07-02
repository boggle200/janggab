use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;
use std::sync::mpsc;

#[allow(dead_code)]
pub fn client_main(ip: &str) {
    // WSL의 default IP를 여기에 입력하세요
    // ip route | grep default 명령어로 얻은 IP 주소 사용
    // 예: "172.20.240.1:8080"
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
            println!("💡 메시지를 입력하고 Enter를 누르세요. 'quit'를 입력하면 종료됩니다.\n");
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
                    print!("📨 {}", buffer);
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    eprintln!("❌ 메시지 받기 오류: {}", e);
                    tx.send(()).unwrap();
                    break;
                }
            }
        }
    });
    
    // 사용자 입력을 받아 서버로 전송하는 메인 루프
    let stdin = io::stdin();
    loop {
        // 논블로킹으로 연결 상태 확인
        if rx.try_recv().is_ok() {
            break;
        }
        
        print!("💬 메시지 입력: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(_) => {
                let message = input.trim();
                
                if message.eq_ignore_ascii_case("quit") {
                    println!("👋 채팅을 종료합니다.");
                    break;
                }
                
                if !message.is_empty() {
                    if let Err(e) = writeln!(stream, "{}", message) {
                        eprintln!("❌ 메시지 전송 오류: {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ 입력 읽기 오류: {}", e);
                break;
            }
        }
    }
}
