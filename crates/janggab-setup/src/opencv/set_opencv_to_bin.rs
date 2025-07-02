use std::io;
use std::process::Command;

pub fn bin_cmd() -> io::Result<()> {
    //let path = "Path";
    //let machine = "Machine";
    //let comma = ";";
    let opencv_bin_path = ";C:\\tools\\opencv\\build\\x64\\vc16\\bin";

    let set_opencv_bin_to_path = format!("[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'Machine') + '{}', 'Machine')", opencv_bin_path);
    
    let command = format!(
        "Start-Process powershell -Verb RunAs -ArgumentList \"-NoProfile -ExecutionPolicy Bypass -Command {}\" -Wait",
        set_opencv_bin_to_path
    );

    match run_elevated_command(&command) {
        Ok(_) => {
            println!("환경 변수가 성공적으로 설정 되었습니다.");
            Ok(())
        },
        Err(e) => {
            eprintln!("환경 변수 설정 실패: {}", e);
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("명령어 실행 실패: {}", e)
            ))
        },
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