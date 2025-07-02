use std::process::Command;
use encoding_rs::EUC_KR;

/* 이걸로 /home/username 에서 작동하도록 만들기 */
pub fn get_wsl_username() -> Option<String> {
    let output = Command::new("wsl")
        .arg("whoami")
        .output()
        .ok()?;

    if output.status.success() {
        let username = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();
        Some(username)
    } else {
        None
    }
}

pub fn glv_env_control(os: &str, cmds: Vec<&'static str>, wsl_pw: &str) {
    if os == "wsl" {
        // 현재 WSL 계정 이름 가져오기
        let username = match get_wsl_username() {
            Some(u) => u,
            None => {
                println!("WSL 계정 이름을 가져올 수 없습니다.");
                return;
            },
        };
        let combined_cmd = cmds.join(" && ");
        let cmd = format!("cd /home/{} && {}", username, combined_cmd);

        println!("Executing combined command in WSL (/home/{}): {}",
                 username, cmd);

        // 먼저 비밀번호없이 sudo -n
        let output = Command::new("wsl")
            .arg("-e")
            .arg("bash")
            .arg("-c")
            .arg(format!("sudo -n bash -c \"{}\"", cmd))
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let (decoded, _, _) = EUC_KR.decode(&output.stdout);
                println!("Command '{}':", cmd);
                println!("{}", decoded);
            }
            _ => {
                // 비밀번호 필요시
                let output = Command::new("wsl")
                    .arg("-e")
                    .arg("bash")
                    .arg("-c")
                    .arg(format!(" echo {} | sudo -S bash -c \"{}\"", wsl_pw, cmd))
                    .output()
                    .expect("Failed to execute WSL command with password");

                if output.status.success() {
                    let (decoded, _, _) = EUC_KR.decode(&output.stdout);
                    println!("Command '{}':", cmd);
                    println!("{}", decoded);
                } else {
                    let (decoded, _, _) = EUC_KR.decode(&output.stderr);
                    println!("Error in command '{}':", cmd);
                    println!("비밀번호가 올바른지 확인하세요: {}.", decoded);
                }
            }
        }
    } else if os == "windows" {
        // Windows: cmd 세션에서 실행
        let combined_cmd = cmds.join(" && ");
        println!("\nExecuting combined command in Windows (C:\\): {}",
                 combined_cmd);

        let output = Command::new("cmd")
            .arg("/C")
            .arg(&combined_cmd)
            .current_dir("C:\\")
            .output()
            .expect("Failed to execute Windows command");

        if output.status.success() {
            let (decoded, _, _) = EUC_KR.decode(&output.stdout);
            println!("Command '{}':", combined_cmd);
            println!("{}", decoded);
        } else {
            let (decoded, _, _) = EUC_KR.decode(&output.stderr);
            println!("Error in command '{}':", combined_cmd);
            println!("{}", decoded);
        }
    } else {
        println!("Unknown operating system: '{}'", os);
    }
}
