#[cfg(test)]
mod tests {
    use gateway_core::types::SessionId;
    use ksp_core::packet::KspPacket;
    use ksp_core::types::{Flags, PacketType};
    use ksp_transport::session::Session;
    use ksp_crypto::kdf::DerivedKeys;
    use ksp_core::version::ProtocolVersion;
    use ksp_core::capability::{Capabilities, CipherSuite};

    fn mock_session() -> Session {
        let keys = DerivedKeys {
            client_write_key: [0; 32],
            server_write_key: [0; 32],
            client_write_iv: [0; 12],
            server_write_iv: [0; 12],
        };
        Session::new(
            *uuid::Uuid::new_v4().as_bytes(),
            ProtocolVersion::new(1, 0),
            Capabilities::empty(),
            CipherSuite::Aes256Gcm,
            keys,
            false
        )
    }

    #[test]
    fn test_codec_encryption_roundtrip() {
        let mut session = mock_session();
        let payload = b"Hello KSP Gateway!";
        
        // Encrypt the packet
        let (seq, nonce) = session.send_nonce.next();
        let encrypted_packet = session.encrypt_packet(
            PacketType::Data,
            Flags::empty(),
            42,
            seq,
            nonce,
            payload
        ).expect("Failed to encrypt packet");

        assert_eq!(encrypted_packet.packet_type, PacketType::Data);
        assert_eq!(encrypted_packet.stream_id, 42);
        assert!(encrypted_packet.flags.contains(Flags::ENCRYPTED));
        
        // Decrypt the packet
        let decrypted_payload = session.decrypt_packet(&encrypted_packet)
            .expect("Failed to decrypt packet");

        assert_eq!(decrypted_payload, payload);
    }
}
