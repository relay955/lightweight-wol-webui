use std::net::UdpSocket;
use crate::error::SystemError;

// MAC 주소를 바이트 배열로 파싱
pub fn parse_mac_address(mac: &str) -> Result<[u8; 6], SystemError> {
    let mac_clean = mac.replace(":", "").replace("-", "");

    if mac_clean.len() != 12 {
        return Err(SystemError::APIError(400, 0, "Invalid MAC address format".to_string()));
    }

    let mut bytes = [0u8; 6];
    for i in 0..6 {
        bytes[i] = u8::from_str_radix(&mac_clean[i*2..i*2+2], 16)
            .map_err(|_| SystemError::APIError(400, 0, "Invalid MAC address hex value".to_string()))?;
    }

    Ok(bytes)
}

// 매직패킷 생성 (6바이트 0xFF + MAC 주소 16회 반복 = 102바이트)
pub fn create_magic_packet(mac_bytes: [u8; 6]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(102);
    packet.extend_from_slice(&[0xFF; 6]);
    for _ in 0..16 {
        packet.extend_from_slice(&mac_bytes);
    }
    packet
}

pub fn send_magic_packet(mac: &str) -> Result<(), SystemError> {
    let mac_bytes = parse_mac_address(mac)?;
    let magic_packet = create_magic_packet(mac_bytes);

    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| SystemError::APIError(500, 0, format!("Failed to bind UDP socket: {}", e)))?;
    socket.set_broadcast(true)
        .map_err(|e| SystemError::APIError(500, 0, format!("Failed to set broadcast: {}", e)))?;
    socket.send_to(&magic_packet, "255.255.255.255:9")
        .map_err(|e| SystemError::APIError(500, 0, format!("Failed to send magic packet: {}", e)))?;

    Ok(())
}
