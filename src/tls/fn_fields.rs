use rustls::hash_hs::HandshakeHash;
use rustls::msgs::enums::{Compression, NamedGroup};
use rustls::msgs::handshake::{Random, ServerECDHParams, ServerExtension, SessionID};
use rustls::{CipherSuite, NoKeyLog, ProtocolVersion};

use super::error::FnError;
use rustls::msgs::codec::Codec;
use rustls::msgs::codec::Reader;
use libafl::bolts::tuples::Prepend;

pub fn fn_protocol_version13() -> Result<ProtocolVersion, FnError> {
    Ok(ProtocolVersion::TLSv1_3)
}

pub fn fn_protocol_version12() -> Result<ProtocolVersion, FnError> {
    Ok(ProtocolVersion::TLSv1_2)
}

pub fn fn_new_session_id() -> Result<SessionID, FnError> {
    let mut id: Vec<u8> = Vec::from([3u8; 32]);
    id.insert(0, 32);
    let id = SessionID::read(&mut Reader::init(id.as_slice()));
    Ok(id.unwrap())
}

pub fn fn_new_random() -> Result<Random, FnError> {
    let random_data: [u8; 32] = [1; 32];
    Ok(Random::from(random_data))
}

pub fn fn_compressions() -> Result<Vec<Compression>, FnError> {
    Ok(vec![Compression::Null])
}

pub fn fn_verify_data(
    server_extensions: &Vec<ServerExtension>,
    verify_transcript: &HandshakeHash,
    client_handshake_traffic_secret_transcript: &HandshakeHash,
) -> Result<Vec<u8>, FnError> {
    let client_random = &[1u8; 32]; // todo see op_random() https://gitlab.inria.fr/mammann/tlspuffin/-/issues/45
    let suite = &rustls::suites::TLS13_AES_128_GCM_SHA256; // todo see op_cipher_suites()

    let group = NamedGroup::secp384r1; // todo https://gitlab.inria.fr/mammann/tlspuffin/-/issues/45

    let keyshare = super::tls13_get_server_key_share(server_extensions)?;
    let server_public_key = keyshare.payload.0.as_slice();

    let mut key_schedule = super::tls12_key_schedule(server_public_key, suite, group)?;

    key_schedule.client_handshake_traffic_secret(
        &client_handshake_traffic_secret_transcript.get_current_hash(),
        &NoKeyLog,
        client_random,
    );

    let pending = key_schedule.into_traffic_with_client_finished_pending();

    let bytes = pending.sign_client_finish(&verify_transcript.get_current_hash());
    Ok(Vec::from(bytes.as_ref()))
}

// ----
// seed_client_attacker12()
// ----

pub fn fn_sign_transcript(
    server_random: &Random,
    server_ecdh_params: &ServerECDHParams,
    transcript: &HandshakeHash,
) -> Result<Vec<u8>, FnError> {
    let secrets = super::tls12_new_secrets(server_random, server_ecdh_params)?;

    let vh = transcript.get_current_hash();
    Ok(secrets.client_verify_data(&vh))
}

// ----
// Cipher Suites
// ----

pub fn fn_new_cipher_suites() -> Result<Vec<CipherSuite>, FnError> {
    Ok(vec![])
}

// todo implement functions for all supported cipher suites as constants
//      https://gitlab.inria.fr/mammann/tlspuffin/-/issues/65
pub fn fn_append_cipher_suite(
    suites: &Vec<CipherSuite>,
    suite: &CipherSuite,
) -> Result<Vec<CipherSuite>, FnError> {
    let mut new: Vec<CipherSuite> = suites.clone();
    new.push(suite.clone());
    Ok(new)
}

pub fn fn_cipher_suite12() -> Result<CipherSuite, FnError> {
    Ok(
        CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
        /*CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
        CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
        CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256
        CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256*/
    )
}

pub fn fn_cipher_suite13() -> Result<CipherSuite, FnError> {
    Ok(CipherSuite::TLS13_AES_128_GCM_SHA256)
}

pub fn fn_weak_export_cipher_suite() -> Result<CipherSuite, FnError> {
    Ok(CipherSuite::TLS_RSA_EXPORT_WITH_DES40_CBC_SHA)
}

// todo should be removed because it returns the ready vec instead of a single one
pub fn fn_weak_export_cipher_suites_remove_me() -> Result<Vec<CipherSuite>, FnError> {
    Ok(vec![CipherSuite::TLS_RSA_EXPORT_WITH_DES40_CBC_SHA])
}

pub fn fn_secure_rsa_cipher_suite12() -> Result<CipherSuite, FnError> {
    Ok(CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA256)
}
