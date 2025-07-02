use crate::tools::get_wsl_username;
use std::io;

pub fn spawn_glove_wsl_code() -> io::Result<()> { // 원하는 폴더를 통째로 옮겨서 실행시켜 버리기 (혹은 복사해서 실행시켜버리기)
    // 자동으로 사용자 이름 가져오기
    let username = match get_wsl_username() {
        Some(name) => name,
        None => {
            eprintln!("❌ WSL 사용자 이름을 가져올 수 없습니다.");
            return Ok(()); // 그냥 넘어갑니다.
        }
    };

    // 사용할 WSL 배포판들
    let distros = ["Ubuntu", "Ubuntu-22.04", "Ubuntu-24.04"];

    // 새로 작성할 Rust main.rs
    let new_content = r#"
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::Command;
use std::time::Duration;

/// WSL2에서 현재 IP 주소를 얻는 함수
fn get_wsl_ip() -> Option<String> {
    let output = Command::new("wsl")
        .arg("hostname")
        .arg("-I") // 대문자 i
        .output()
        .ok()?;

    if output.status.success() {
        let ip = String::from_utf8_lossy(&output.stdout)
            .trim()
            .split_whitespace()
            .next()? // 여러 IP가 나올 경우 첫 번째 사용
            .to_string();
        Some(ip)
    } else {
        None
    }
}

fn main() {
    let server_port = 7878;
    let server_ip = match get_wsl_ip() {
        Some(ip) => ip,
        None => {
            eprintln!("❌ WSL2 IP 주소를 가져오지 못했습니다.");
            return;
        }
    };

    let address = format!("{}:{}", server_ip, server_port);
    println!("🔍 Connecting to WSL2 server at {}", address);

    match TcpStream::connect(&address) {
        Ok(mut stream) => {
            println!("✅ Connected to the server!");

            // 타임아웃 설정 (선택)
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            stream.set_write_timeout(Some(Duration::from_secs(5))).unwrap();

            let message = "Hello from Windows client!";
            stream.write_all(message.as_bytes()).unwrap();
            println!("📤 Sent: {}", message);

            let mut buffer = [0; 512];
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
                    println!("📩 Received: {}", response);
                }
                Err(e) => {
                    eprintln!("❌ Read error: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Could not connect to server: {}", e);
        }
    }
}

"#;

    let mut success = false;

    for distro in distros {
        println!("🔍 {} 배포판에서 glvvv/src/main.rs 변경을 시도합니다.", distro);

        let cmd = format!(
            "cd /home/{}/glvvv/src && echo '{}' > main.rs",
            username,
            new_content
        );

        let output = std::process::Command::new("wsl")
            .arg("-d")
            .arg(distro)
            .arg("-e")
            .arg("bash")
            .arg("-c")
            .arg(cmd)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                println!("✅ {} 배포판에서 main.rs 변경 완료!", distro);
                success = true;
                break;
            }
            Ok(output) => {
                eprintln!("❌ {} 배포판에서 오류!", distro);
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => {
                eprintln!("❌ {} 배포판에서 WSL 커맨드 실행 오류!", distro);
                eprintln!("{}", e);
            }
        }
    }

    if !success {
        eprintln!("❌ 어느 WSL 배포판에도 main.rs 변경을 할 수 없습니다.");
    }

    Ok(())
}
