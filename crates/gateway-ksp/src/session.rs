//! Session manager — tracks active KSP sessions and handles request dispatch.

use std::{net::SocketAddr, sync::Arc};
use dashmap::DashMap;
use tokio::net::TcpStream;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;
use sha2::Digest;
use ed25519_dalek::Signer;

use gateway_core::{
    error::{GatewayError, GatewayResult},
    types::SessionId,
};

use ksp_core::{
    capability::{self, Capabilities},
    constants::HEADER_SIZE,
    packet::KspPacket,
    types::{Flags, PacketType},
};
use ksp_crypto::{
    certificate::KspCertificate,
    kdf,
    x25519::EphemeralKeypair,
};
use ksp_handshake::{
    auth::{AuthMethod, AuthResult},
    messages::{ClientHello, HandshakeFinish, ServerHello},
};
use ksp_transport::session::Session;

/// Metadata about a single active KSP session.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id:        SessionId,
    pub peer_addr: SocketAddr,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub requests:  u64,
    pub bytes_in:  u64,
    pub bytes_out: u64,
}

/// Manages all active KSP sessions.
pub struct SessionManager {
    sessions: DashMap<SessionId, SessionInfo>,
    certificate: KspCertificate,
    signing_key: ed25519_dalek::SigningKey,
    server_capabilities: Capabilities,
}

impl SessionManager {
    pub fn new() -> Self {
        let (cert, key) = ksp_server::server::load_or_generate_cert().expect("Failed to load cert");
        Self {
            sessions: DashMap::new(),
            certificate: cert,
            signing_key: key,
            server_capabilities: Capabilities::all(),
        }
    }

    /// Called by `KspListener` for each accepted TCP connection.
    pub async fn handle_connection(
        self: Arc<Self>,
        mut stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> GatewayResult<()> {
        let session_id = SessionId::new();

        let info = SessionInfo {
            id: session_id.clone(),
            peer_addr,
            connected_at: Utc::now(),
            requests:  0,
            bytes_in:  0,
            bytes_out: 0,
        };
        self.sessions.insert(session_id.clone(), info);
        info!(session = %session_id, peer = %peer_addr, "Session established");

        match self.perform_handshake(&mut stream, peer_addr, session_id.clone()).await {
            Ok(session) => {
                // Handshake success, start packet processing loop
                self.process_session(&mut stream, session, peer_addr).await?;
            }
            Err(e) => {
                warn!(session = %session_id, error = %e, "Handshake failed");
            }
        }

        self.sessions.remove(&session_id);
        info!(session = %session_id, "Session closed");
        Ok(())
    }

    async fn perform_handshake(
        &self,
        stream: &mut TcpStream,
        addr: SocketAddr,
        gateway_session_id: SessionId,
    ) -> GatewayResult<Session> {
        let (client_hello_packet, _) = ksp_server::server::read_packet(stream)
            .await
            .map_err(|e| GatewayError::Internal(e.to_string()))?;

        if client_hello_packet.packet_type != PacketType::ClientHello {
            return Err(GatewayError::Internal("expected ClientHello".into()));
        }

        let client_hello = ClientHello::deserialize(&client_hello_packet.payload)
            .map_err(|e| GatewayError::Internal(e.to_string()))?;

        let selected_version = ksp_core::version::ProtocolVersion::negotiate(
            &client_hello.supported_versions,
            &[ksp_core::CURRENT_VERSION],
        ).map_err(|e| GatewayError::Internal(e.to_string()))?;

        let (selected_caps, cipher_suite) = capability::negotiate_capabilities(
            client_hello.capabilities, 
            self.server_capabilities
        ).map_err(|e| GatewayError::Internal(e.to_string()))?;

        let server_keypair = EphemeralKeypair::generate();
        let mut server_random = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut server_random);
        let session_id = *uuid::Uuid::new_v4().as_bytes();

        let server_hello = ServerHello {
            selected_version,
            selected_capabilities: selected_caps,
            server_random,
            ephemeral_public_key: server_keypair.public_key_bytes(),
            session_id,
        };

        let server_hello_packet = KspPacket::new_handshake(PacketType::ServerHello, server_hello.serialize());
        ksp_server::server::send_packet(stream, &server_hello_packet).await
            .map_err(|e| GatewayError::Internal(e.to_string()))?;

        let mut binding_data = Vec::with_capacity(32 * 4);
        binding_data.extend_from_slice(&client_hello.client_random);
        binding_data.extend_from_slice(&server_random);
        binding_data.extend_from_slice(&client_hello.ephemeral_public_key);
        binding_data.extend_from_slice(&server_hello.ephemeral_public_key);

        let binding_signature = self.signing_key.sign(&binding_data).to_bytes();
        let mut cert_payload = self.certificate.serialize();
        cert_payload.extend_from_slice(&binding_signature);

        let cert_packet = KspPacket::new_handshake(PacketType::Certificate, cert_payload.clone());
        ksp_server::server::send_packet(stream, &cert_packet).await
            .map_err(|e| GatewayError::Internal(e.to_string()))?;

        let shared_secret = server_keypair.diffie_hellman(&client_hello.ephemeral_public_key)
            .map_err(|e| GatewayError::Internal(e.to_string()))?;
        
        let derived_keys = kdf::derive_session_keys(
            shared_secret.as_bytes(),
            &client_hello.client_random,
            &server_random,
        ).map_err(|e| GatewayError::Internal(e.to_string()))?;

        let mut session = Session::new(
            session_id,
            selected_version,
            selected_caps,
            cipher_suite,
            derived_keys,
            false,
        );

        let (auth_packet, _) = ksp_server::server::read_packet(stream).await
            .map_err(|e| GatewayError::Internal(e.to_string()))?;

        if auth_packet.packet_type == PacketType::AuthRequest {
            let auth_result = AuthResult::Success; // Gateway accepts anonymous auth by default
            
            let (seq, nonce) = session.send_nonce.next();
            let auth_response_packet = session.encrypt_packet(
                PacketType::AuthResponse,
                Flags::empty(),
                0,
                seq,
                nonce,
                &auth_result.serialize(),
            ).map_err(|e| GatewayError::Internal(e.to_string()))?;
            
            ksp_server::server::send_packet(stream, &auth_response_packet).await
                .map_err(|e| GatewayError::Internal(e.to_string()))?;
        }

        let mut transcript = Vec::new();
        transcript.extend_from_slice(&client_hello_packet.payload);
        transcript.extend_from_slice(&server_hello_packet.payload);
        transcript.extend_from_slice(&cert_payload);

        let (client_finish_packet, _) = ksp_server::server::read_packet(stream).await
            .map_err(|e| GatewayError::Internal(e.to_string()))?;

        if client_finish_packet.packet_type != PacketType::HandshakeFinish {
            return Err(GatewayError::Internal("expected HandshakeFinish".into()));
        }

        let client_finish_payload = session.decrypt_packet(&client_finish_packet)
            .map_err(|e| GatewayError::Internal(e.to_string()))?;
        
        let client_finish = HandshakeFinish::deserialize(&client_finish_payload)
            .map_err(|e| GatewayError::Internal(e.to_string()))?;

        let expected_client_verify = ksp_crypto::compute_finished_mac(&session.keys.client_write_key, &transcript);
        if client_finish.verify_data != expected_client_verify {
            return Err(GatewayError::Internal("client HandshakeFinish verification failed".into()));
        }

        let server_verify_data = ksp_crypto::compute_finished_mac(&session.keys.server_write_key, &transcript);
        let server_finish = HandshakeFinish { verify_data: server_verify_data };
        
        let (seq, nonce) = session.send_nonce.next();
        let server_finish_packet = session.encrypt_packet(
            PacketType::HandshakeFinish,
            Flags::empty(),
            0,
            seq,
            nonce,
            &server_finish.serialize(),
        ).map_err(|e| GatewayError::Internal(e.to_string()))?;
        
        ksp_server::server::send_packet(stream, &server_finish_packet).await
            .map_err(|e| GatewayError::Internal(e.to_string()))?;

        Ok(session)
    }

    async fn process_session(&self, stream: &mut TcpStream, mut session: Session, peer_addr: SocketAddr) -> GatewayResult<()> {
        debug!("Processing KSP session for peer {}", peer_addr);
        
        loop {
            let (packet, _) = match ksp_server::server::read_packet(stream).await {
                Ok(p) => p,
                Err(e) => {
                    break;
                }
            };

            match packet.packet_type {
                PacketType::KeepAlive => {
                    let ack = KspPacket::new_handshake(PacketType::KeepAliveAck, Vec::new());
                    let _ = ksp_server::server::send_packet(stream, &ack).await;
                    session.keepalive.record_activity();
                }
                PacketType::GoAway => {
                    break;
                }
                PacketType::Data | PacketType::StreamData => {
                    // Gateway specific packet handling
                    let plaintext = session.decrypt_packet(&packet)
                        .map_err(|e| GatewayError::Internal(e.to_string()))?;
                        
                    // For Milestone 2: just echo back the data as proof of connection
                    let (seq, nonce) = session.send_nonce.next();
                    let response = session.encrypt_packet(
                        PacketType::Data,
                        Flags::empty(),
                        packet.stream_id,
                        seq,
                        nonce,
                        &plaintext,
                    ).map_err(|e| GatewayError::Internal(e.to_string()))?;
                    
                    let _ = ksp_server::server::send_packet(stream, &response).await;
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn sessions(&self) -> Vec<SessionInfo> {
        self.sessions.iter().map(|e| e.value().clone()).collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
