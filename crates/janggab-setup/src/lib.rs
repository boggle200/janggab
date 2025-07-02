mod games;
mod install_rust_wsl;
mod make_server;
mod opencv;
mod tools;

use crate::{
    games::wgpu_setup_main, 
    make_server::{spawn_glove_wsl_code},
};

use opencv::opencv_setup_main;

pub fn help(lang: &str) -> std::io::Result<()> {
    if lang == "ko" {
        println!(r#"
setup_all : 필요한 모든 패키지를 자동으로 설치합니다.
setup_server : windows에서 사용할 서버 코드를 작성합니다.
setup_opencv : windows에서 사용할 opencv를 설치하고 이에 필요한 설정을 자동으로 처리합니다.
setup_pkgs : wsl에서 필요한 패키지들을 설치합니다.
        "#);
    } else if lang == "en" {
        println!(r#"
setup_all : install all packages that we need. (not recommended)
setup_server : write server code used from windows.
setup_opencv : install opencv to windows and do all settings we need automatically .
setup_pkgs : install packages that need from wsl.
        "#);
    } else {
        println!("plz, choose only one from under this line.\nthe other languages still need time to prepare.\n1. 한국어\n2. English");
    }

    Ok(())
}

pub fn setup_all() -> std::io::Result<()> {
    Ok(())
}

pub fn setup_server() -> std::io::Result<()> {
    spawn_glove_wsl_code().unwrap();
    Ok(())
}

pub fn setup_opencv() -> std::io::Result<()> {
    opencv_setup_main().unwrap();
    Ok(())
}

/*pub fn setup_rust() -> std::io::Result<()> {
    install_rust_wsl_main().unwrap();
    Ok(())
}*/
pub fn setup_pkgs(password: &str) -> std::io::Result<()> {
    wgpu_setup_main(password).unwrap();
    Ok(())
}
