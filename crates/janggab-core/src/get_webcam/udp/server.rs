use std::net::UdpSocket;
use std::time::Duration;
use std::collections::HashMap;
use anyhow::{Result, Context};
use opencv::{
    prelude::*,
    highgui,
    core,
};

const HEADER_SIZE: usize = 4 + 2 + 2;
const MAX_RECV_BUFFER_SIZE: usize = 65535;

#[allow(warnings)]
pub fn server_main(img_width: usize, img_height: usize) -> Result<Vec<u8>> {
    let server_addr = "0.0.0.0:52525";
    let socket = UdpSocket::bind(server_addr).context("서버 바인딩 실패")?;
    socket.set_read_timeout(Some(Duration::from_secs(1)))
        .context("소켓 읽기 타임아웃 설정 실패")?;

    println!("🚀 UDP 서버가 {}에서 시작되었습니다", server_addr);
    println!("📡 클라이언트 연결을 기다리는 중...\n");

    let mut buf = [0u8; MAX_RECV_BUFFER_SIZE];
    let mut client_addr = None;

    // --- HELLO 메시지 수신 대기 ---
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                println!("📨 {}로부터 {} 바이트 수신: {:?}", addr, size, &buf[..size]);

                if &buf[..size] == b"HELLO" {
                    println!("✅ 클라이언트 연결됨: {}", addr);
                    client_addr = Some(addr);
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

    highgui::named_window("Received Image", highgui::WINDOW_AUTOSIZE)
        .context("OpenCV 창 생성 실패")?;

    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                println!("📨 {}로부터 {} 바이트 수신: {:?}", addr, size, &buf[..size]);

                if let Some(expected_addr) = client_addr {
                    if addr != expected_addr {
                        eprintln!("⚠️ 예상치 못한 클라이언트({})로부터 패킷 수신. 무시", addr);
                        continue;
                    }
                }

                if size < HEADER_SIZE {
                    eprintln!("⚠️ 너무 작은 패킷 수신 ({} 바이트). 무시", size);
                    continue;
                }

                let frame_id = u32::from_be_bytes(buf[0..4].try_into().unwrap());
                let total_chunks = u16::from_be_bytes(buf[4..6].try_into().unwrap());
                let chunk_index = u16::from_be_bytes(buf[6..8].try_into().unwrap());
                let chunk_data = &buf[HEADER_SIZE..size];

                let (existing_total_chunks, frame_chunks) = incomplete_frames
                    .entry(frame_id)
                    .or_insert_with(|| (total_chunks, vec![None; total_chunks as usize]));

                if *existing_total_chunks != total_chunks {
                    eprintln!("⚠️ 프레임 ID {}의 총 청크 수 불일치: 기존 {}, 현재 {}", frame_id, *existing_total_chunks, total_chunks);
                }

                if (chunk_index as usize) >= total_chunks as usize {
                    eprintln!("⚠️ 잘못된 청크 인덱스 {} / {}", chunk_index, total_chunks);
                    continue;
                }

                if frame_chunks[chunk_index as usize].is_none() {
                    frame_chunks[chunk_index as usize] = Some(chunk_data.to_vec());
                    *received_counts.entry(frame_id).or_insert(0) += 1;
                }

                let current_received_count = *received_counts.get(&frame_id).unwrap_or(&0);

                if current_received_count == total_chunks {
                    println!("✅ 프레임 ID {}의 {}개 청크 수신 완료!", frame_id, total_chunks);

                    let mut full_image_data = Vec::new();
                    let mut complete = true;
                    for chunk_opt in frame_chunks.iter() {
                        if let Some(chunk) = chunk_opt {
                            full_image_data.extend_from_slice(chunk);
                        } else {
                            eprintln!("❌ 청크 누락 발생. 프레임 ID {}", frame_id);
                            complete = false;
                            break;
                        }
                    }

                    if complete {
                        let expected_width = img_width;
                        let expected_height = img_height;
                        let expected_channels = 3;
                        let expected_total_size = expected_width * expected_height * expected_channels;

                        if full_image_data.len() == expected_total_size {
                            println!("🎨 재구성 완료: {} 바이트", full_image_data.len());

                            let mut image_mat = core::Mat::zeros(
                                expected_height as i32,
                                expected_width as i32,
                                core::CV_8UC3,
                            )
                            .context("MatExpr 생성 실패")?
                            .to_mat()
                            .context("Mat 변환 실패")?;

                            if let Ok(mat_data_slice) = image_mat.data_bytes_mut() {
                                if mat_data_slice.len() == full_image_data.len() {
                                    mat_data_slice.copy_from_slice(&full_image_data);
                                } else {
                                    eprintln!("❌ 이미지 크기 불일치!");
                                    continue;
                                }
                            } else {
                                eprintln!("❌ Mat 바이트 접근 실패");
                                continue;
                            }

                            highgui::imshow("Received Image", &image_mat)
                                .context("이미지 표시 실패")?;
                            highgui::wait_key(1)
                                .context("WaitKey 실패")?;

                            // ✅ 수신한 이미지 데이터를 그대로 반환
                            return Ok(full_image_data);
                        } else {
                            eprintln!("❌ 데이터 길이 불일치! 예상 {} 바이트, 실제 {} 바이트",
                                expected_total_size, full_image_data.len());
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
