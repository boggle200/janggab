// server.rs
use std::net::UdpSocket;
use std::time::Duration;
use std::collections::HashMap;
use anyhow::{Result, Context};
use opencv::{
    prelude::*,
    highgui, // 이미지 표시를 위한 모듈
    core,
};

// 클라이언트 코드와 동일하게 헤더 크기 정의
const HEADER_SIZE: usize = 4 + 2 + 2; // 프레임 ID (4) + 총 청크 수 (2) + 현재 청크 인덱스 (2)

// UDP 소켓의 최대 수신 버퍼 크기 (이론적인 UDP 최대 패킷 크기보다 약간 더 크게 설정)
const MAX_RECV_BUFFER_SIZE: usize = 65535;

#[allow(warnings)]
pub fn server_main(img_width: usize, img_height: usize) -> Result<()> {
    let server_addr = "0.0.0.0:52525";
    let socket = UdpSocket::bind(server_addr)
        .context("서버 바인딩 실패")?;
    socket.set_read_timeout(Some(Duration::from_secs(1)))
        .context("소켓 읽기 타임아웃 설정 실패")?;

    println!("🚀 UDP 서버가 {}에서 시작되었습니다", server_addr);
    println!("📡 클라이언트 연결을 기다리는 중...\n");

    let mut buf = [0u8; MAX_RECV_BUFFER_SIZE];
    let mut client_addr = None; // 이 변수는 이제 실제로 사용됩니다.

    // --- 초기 연결 대기 ---
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                let msg = String::from_utf8_lossy(&buf[..size]);
                println!("📨 {}로부터 메시지 수신: {}", addr, msg);

                if msg.trim() == "HELLO" {
                    println!("✅ 클라이언트 연결됨: {}", addr);
                    client_addr = Some(addr); // 클라이언트 주소 저장
                    // 연결 확인 응답
                    socket.send_to(b"CONNECTED", addr)
                        .context("CONNECTED 메시지 전송 실패")?;
                    break;
                }
            }
            Err(_) => {
                println!("⏳ 클라이언트 연결 대기중...");
            }
        }
    }

    println!("📊 클라이언트로부터 이미지 데이터 수신 중...\n");

    let mut incomplete_frames: HashMap<u32, (u16, Vec<Option<Vec<u8>>>)> = HashMap::new();
    let mut received_counts: HashMap<u32, u16> = HashMap::new();

    // OpenCV 창 생성
    highgui::named_window("Received Image", highgui::WINDOW_AUTOSIZE)
        .context("OpenCV 창 생성 실패")?;

    // --- 연결 후 데이터 수신 ---
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                // 저장된 클라이언트 주소와 일치하는지 확인 (선택 사항이지만 안전을 위해)
                if let Some(expected_addr) = client_addr {
                    if addr != expected_addr {
                        eprintln!("⚠️ 예상치 못한 클라이언트({})로부터 패킷 수신. 무시합니다.", addr);
                        continue;
                    }
                }

                if size < HEADER_SIZE {
                    eprintln!("⚠️ 너무 작은 패킷 수신 (크기: {}). 무시합니다.", size);
                    continue;
                }

                // 헤더 파싱
                let frame_id = u32::from_be_bytes(buf[0..4].try_into().unwrap());
                let total_chunks = u16::from_be_bytes(buf[4..6].try_into().unwrap());
                let chunk_index = u16::from_be_bytes(buf[6..8].try_into().unwrap());
                let chunk_data = &buf[HEADER_SIZE..size];

                // println!("📦 프레임 ID: {}, 청크 {}/{}, 데이터 크기: {}", frame_id, chunk_index, total_chunks, chunk_data.len());

                // 해당 프레임 ID의 데이터를 HashMap에 저장
                let (existing_total_chunks, frame_chunks) = incomplete_frames
                    .entry(frame_id)
                    .or_insert_with(|| (total_chunks, vec![None; total_chunks as usize]));

                // 총 청크 수가 불일치하면 문제 발생 가능성 (로깅만 하고 일단 진행)
                if *existing_total_chunks != total_chunks {
                    eprintln!("⚠️ 프레임 ID {}에 대해 총 청크 수 불일치! 이전: {}, 현재: {}", frame_id, *existing_total_chunks, total_chunks);
                    // 이 경우, 기존 데이터를 버리고 새로 시작하거나, 더 정교한 로직이 필요할 수 있습니다.
                    // 여기서는 일단 기존 total_chunks를 따르도록 합니다.
                }

                // 청크 인덱스 유효성 검사
                if (chunk_index as usize) >= total_chunks as usize {
                    eprintln!("⚠️ 잘못된 청크 인덱스 {} (총 청크 수 {}). 무시합니다.", chunk_index, total_chunks);
                    continue;
                }

                // 이미 받은 청크인지 확인하고 저장
                if frame_chunks[chunk_index as usize].is_none() {
                    frame_chunks[chunk_index as usize] = Some(chunk_data.to_vec());
                    *received_counts.entry(frame_id).or_insert(0) += 1;
                } else {
                    // 중복 수신된 청크 (UDP 특성상 가능)
                    // println!("💡 프레임 ID {}의 청크 {}은(는) 이미 수신됨.", frame_id, chunk_index);
                }

                let current_received_count = *received_counts.get(&frame_id).unwrap_or(&0);

                // 모든 청크가 수신되었는지 확인
                if current_received_count == total_chunks {
                    println!("✅ 프레임 ID {}의 모든 {}개 청크 수신 완료! 이미지 재구성 시작.", frame_id, total_chunks);

                    let mut full_image_data = Vec::new();
                    let mut complete = true;
                    for chunk_opt in frame_chunks.iter() {
                        if let Some(chunk) = chunk_opt {
                            full_image_data.extend_from_slice(chunk);
                        } else {
                            // 이 경우는 모든 청크가 수신되지 않았는데 카운트가 맞지 않은 경우 (패킷 손실 or 논리적 오류)
                            eprintln!("❌ 오류: 프레임 ID {} 재구성 중 누락된 청크 발견! (카운트 불일치)", frame_id);
                            complete = false;
                            break;
                        }
                    }

                    if complete {
                        let expected_width = img_width;
                        let expected_height = img_height;
                        let expected_channels = 3; // BGR (Blue, Green, Red)

                        let expected_total_size = expected_width * expected_height * expected_channels;

                        if full_image_data.len() == expected_total_size {
                            println!("🎨 재구성된 프레임 ID {}의 전체 이미지 데이터 (Vec<u8> 형식, 총 {} 바이트):",
                                     frame_id, full_image_data.len());
                            // **수정된 출력 부분: full_image_data의 모든 내용을 출력합니다.**
                            println!("{:?}", full_image_data);
                            // **수정된 출력 부분 끝**

                            let mut image_mat = unsafe {
                                Mat::new_rows_cols(
                                    expected_height as i32,
                                    expected_width as i32,
                                    core::CV_8UC3,
                                )
                            }.context("Mat 생성 실패")?;

                            if let Ok(mat_data_slice) = image_mat.data_bytes_mut() {
                                if mat_data_slice.len() == full_image_data.len() {
                                    mat_data_slice.copy_from_slice(&full_image_data);
                                } else {
                                    eprintln!("❌ Mat 데이터 슬라이스 크기 불일치! 예상: {} 실제: {}",
                                                mat_data_slice.len(), full_image_data.len());
                                    continue;
                                }
                            } else {
                                eprintln!("❌ Mat 데이터에 접근 실패.");
                                continue;
                            }

                            highgui::imshow("Received Image", &image_mat)
                                .context("이미지 표시 실패")?;
                            highgui::wait_key(1)
                                .context("Wait key 실패")?;
                            println!("🖼️ 프레임 ID {} 이미지 표시 완료.", frame_id);
                        } else {
                            eprintln!("❌ 프레임 ID {} 재구성된 이미지 크기 불일치! 예상: {} 실제: {}",
                                        frame_id, expected_total_size, full_image_data.len());
                        }
                    }

                    incomplete_frames.remove(&frame_id);
                    received_counts.remove(&frame_id);
                }
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    eprintln!("⚠️ UDP 수신 오류: {}", e);
                }
            }
        }
    }
}
