use std::io;
use std::process::Command;

pub fn choco_cmd() -> io::Result<()> {
    let choco_install_cmd = "Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))";

    let command = format!(
        "Start-Process powershell -Verb RunAs -ArgumentList \"-NoProfile -ExecutionPolicy Bypass -Command {}\" -Wait",
        choco_install_cmd
    );

    match run_elevated_command(&command) {
        Ok(_) => {
            println!("choco가 성공적으로 설치되었습니다.");
            Ok(())
        },
        Err(e) => {
            eprintln!("오류 발생: {}", e);
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("명령어 실행 실패: {}", e)
            ))
        }
    }
}

pub fn install_equipments() -> io::Result<()> {
    let equipments_install_cmd = "choco install llvm opencv";

    let command = format!(
        "Start-Process powershell -Verb RunAs -ArgumentList \"-NoProfile -ExecutionPolicy Bypass -Command {}\" -Wait",
        equipments_install_cmd
    );

    match run_elevated_command(&command) {
        Ok(_) => {
            println!("choco가 성공적으로 설치되었습니다.");
            Ok(())
        },
        Err(e) => {
            eprintln!("오류 발생: {}", e);
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("명령어 실행 실패: {}", e)
            ))
        }
    }
}

fn run_elevated_command(cmd: &str) -> io::Result<()> {
    let output = Command::new("powershell")
        .args(&[
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            cmd
        ])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("명령어 실행 실패: {}", error_message)
        ))
    }
}