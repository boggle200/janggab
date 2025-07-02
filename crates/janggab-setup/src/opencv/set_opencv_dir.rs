use std::io;
use std::process::Command;
use std::fs;
use std::path::Path;
use regex::Regex;

pub fn dirs_cmd() -> io::Result<()> {
    let opencv_workd_version = get_opencv_num_and_version();
    
    let opencv_world = format!("opencv_world{}", opencv_workd_version);
    let opencv_link_libs = opencv_world.as_str();
    let opencv_link_libs_name = "OPENCV_LINK_LIBS";

    let opencv_link_paths = "C:\\tools\\opencv\\build\\x64\\vc16\\lib";
    let opencv_link_paths_name = "OPENCV_LINK_PATHS";

    let opencv_include_paths = "C:\\tools\\opencv\\build\\include";
    let opencv_include_paths_name = "OPENCV_INCLUDE_PATHS";

    let set_dirs = format!(
        "[Environment]::SetEnvironmentVariable('{}', '{}', 'Machine')
        [Environment]::SetEnvironmentVariable('{}', '{}', 'Machine')
        [Environment]::SetEnvironmentVariable('{}', '{}', 'Machine')",
        opencv_link_libs_name,
        opencv_link_libs,
        opencv_include_paths_name,
        opencv_include_paths,
        opencv_link_paths_name,
        opencv_link_paths
    );

    let command = format!(
        "Start-Process powershell -Verb RunAs -ArgumentList \"-NoProfile -ExecutionPolicy Bypass -Command {}\" -Wait",
        set_dirs
    );

    match run_elevated_command(&command) {
        Ok(_) => {
            println!("필요한 환경 변수가 모두 성공적으로 설정되었습니다.");
            Ok(())
        }
        Err(e) => {
            eprintln!("오류 발생: {}", e);
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("명령어 실행 실패: {}", e)
            ))
        },
    }
}

fn get_opencv_num_and_version() -> u32 {
    let opencv_lib_path = "C:\\tools\\opencv\\build\\x64\\vc16\\lib";
    let opencv_world_prefix = "opencv_world";

    // 라이브러리 경로가 존재하지 않으면 패닉
    if !Path::new(opencv_lib_path).exists() {
        panic!("OpenCV 라이브러리 경로가 존재하지 않습니다: {}", opencv_lib_path);
    }

    // 디렉토리 읽기 실패시 패닉
    let entries = fs::read_dir(opencv_lib_path)
        .expect("디렉토리를 읽을 수 없습니다");

    // opencv_world 뒤의 숫자를 찾기 위한 정규식
    let re = Regex::new(&format!(r"{}(\d+)\.lib", opencv_world_prefix)).unwrap();

    for entry in entries {
        let entry = entry.expect("디렉토리 엔트리를 읽을 수 없습니다");
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        // 정규식에 매칭되는 파일 찾기
        if let Some(captures) = re.captures(&file_name) {
            if let Some(version_str) = captures.get(1) {
                let version = version_str.as_str().parse::<u32>()
                    .expect("버전 숫자가 u32 형식으로 변환되지 않습니다");
                
                println!("발견된 OpenCV 라이브러리: {}", file_name);
                println!("추출된 버전 숫자: {}", version);
                return version;
            }
        }
    }

    panic!("opencv_world 라이브러리를 찾을 수 없습니다");
}

pub fn run_elevated_command(cmd: &str) -> io::Result<()> {
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
