mod install_with_choco;
mod set_opencv_dir;
mod set_opencv_to_bin;

use install_with_choco::{choco_cmd, install_equipments};
use set_opencv_dir::dirs_cmd;
use set_opencv_to_bin::bin_cmd;

pub fn opencv_setup_main() -> std::io::Result<()> {
    match choco_cmd() {
        Ok(_) => {
            match install_equipments() {
                Ok(_) => {
                    match dirs_cmd(){
                        Ok(_) => {
                            match bin_cmd() {
                                Ok(_) => {
                                    println!("필요한 모든 준비가 끝났습니다.");
                                    Ok(())
                                },
                                Err(e) => {
                                    eprintln!("문제 발생: {}", e);
                                    Err(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        format!("명령어 실행 실패: {}", e)
                                    ))
                                },
                            }
                        },
                        Err(e) => {
                            eprintln!("문제 발생: {}", e);
                            Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("명령어 실행 실패: {}", e)
                            ))
                        },
                    }
                },
                Err(e) => {
                    eprintln!("문제 발생: {}", e);
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("명령어 실행 실패: {}", e)
                    ))
                },
            }
        },
        Err(e) => {
            eprintln!("문제 발생: {}", e);
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("명령어 실행 실패: {}", e)
            ))
        },
    }
}
