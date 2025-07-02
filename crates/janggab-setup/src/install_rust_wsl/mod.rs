use crate::tools::get_wsl_username;

use std::io;

#[allow(dead_code)]
pub fn install_rust_wsl_main() -> io::Result<()> {
    let username = match get_wsl_username() {
        Some(name) => name,
        None => {
            eprintln!("❌ WSL 사용자 이름을 가져올 수 없습니다.");
            return Ok(()); 
        }
    };

    let distros = ["Ubuntu", "Ubuntu-22.04", "Ubuntu-24.04"];

    for distro in distros {
        println!("🔍 {} 배포판에서 Rust 설치 확인합니다.", distro);

        let cmd = format!("cd /home/{username} && which rustc");

        let output = std::process::Command::new("wsl")
            .arg("-d")
            .arg(distro)
            .arg("-e")
            .arg("bash")
            .arg("-c")
            .arg(cmd)
            .output();

        if output.unwrap().status.success() {
            println!("✅ {} 배포판에 Rust가 이미 설치되어 있습니다.", distro);
            return Ok(()); // 그냥 넘어갑니다
        }
    }

    println!("⚡ Rust가 설치되어 있지 않습니다. 설치를 진행합니다.");

    let cmd = format!(
        "cd /home/{username} && echo 1 | curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
        username = username
    );

    for distro in distros {
        println!("🔍 {} 배포판에서 Rust 설치를 시도합니다.", distro);

        let output = std::process::Command::new("wsl")
            .arg("-d")
            .arg(distro)
            .arg("-e")
            .arg("bash")
            .arg("-c")
            .arg(&cmd)
            .output();

        let output = output.unwrap().clone();
        if output.status.success() {
            println!("✅ {} 배포판에서 Rust 설치 완료!", distro);
            break;
        } else {
            eprintln!("❌ {} 배포판에서 오류!", distro);
            eprintln!("일반적으로 rust가 설치 되어 있다면 발생하는 에러입니다.\nrustc를 입력해 rust의 설치를 확인해보세요!");
            eprintln!("{}", String::from_utf8_lossy(&output.clone().stderr));
        }
    }

    Ok(())
}
