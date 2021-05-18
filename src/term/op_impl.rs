use rand::random;
use rand::seq::SliceRandom;
use ring::hkdf::{KeyType, Prk, HKDF_SHA256};
use ring::hmac::Key;
use ring::rand::SystemRandom;
use ring::{hkdf, hmac};
use rustls::internal::msgs::alert::AlertMessagePayload;
use rustls::internal::msgs::codec::Codec;
use rustls::internal::msgs::enums::ContentType::Handshake as RecordHandshake;
use rustls::internal::msgs::enums::{AlertDescription, Compression, HandshakeType, ServerNameType, NamedGroup};
use rustls::internal::msgs::handshake::{ClientExtension, ClientHelloPayload, HandshakeMessagePayload, HandshakePayload, Random, ServerName, ServerNamePayload, SessionID, KeyShareEntry};
use rustls::internal::msgs::message::MessagePayload::Handshake;
use rustls::internal::msgs::message::{Message, MessagePayload};
use rustls::{CipherSuite, ProtocolVersion, SignatureScheme};
use rustls::internal::msgs::base::PayloadU16;

// -----
// utils
// -----

enum SecretKind {
    ResumptionPSKBinderKey,
    ClientEarlyTrafficSecret,
    ClientHandshakeTrafficSecret,
    ServerHandshakeTrafficSecret,
    ClientApplicationTrafficSecret,
    ServerApplicationTrafficSecret,
    ExporterMasterSecret,
    ResumptionMasterSecret,
    DerivedSecret,
}

impl SecretKind {
    fn to_bytes(&self) -> &'static [u8] {
        match self {
            SecretKind::ResumptionPSKBinderKey => b"res binder",
            SecretKind::ClientEarlyTrafficSecret => b"c e traffic",
            SecretKind::ClientHandshakeTrafficSecret => b"c hs traffic",
            SecretKind::ServerHandshakeTrafficSecret => b"s hs traffic",
            SecretKind::ClientApplicationTrafficSecret => b"c ap traffic",
            SecretKind::ServerApplicationTrafficSecret => b"s ap traffic",
            SecretKind::ExporterMasterSecret => b"exp master",
            SecretKind::ResumptionMasterSecret => b"res master",
            SecretKind::DerivedSecret => b"derived",
        }
    }
}

fn derive_secret<L, F, T>(
    secret: &hkdf::Prk,
    kind: SecretKind,
    algorithm: L,
    context: &Vec<u8>,
    into: F,
) -> T
where
    L: KeyType,
    F: for<'b> FnOnce(hkdf::Okm<'b, L>) -> T,
{
    const LABEL_PREFIX: &[u8] = b"tls13 ";

    let label = kind.to_bytes();
    let output_len = u16::to_be_bytes(algorithm.len() as u16);
    let label_len = u8::to_be_bytes((LABEL_PREFIX.len() + label.len()) as u8);
    let context_len = u8::to_be_bytes(context.len() as u8);

    let info = &[
        &output_len[..],
        &label_len[..],
        LABEL_PREFIX,
        label,
        &context_len[..],
        context,
    ];
    let okm = secret.expand(info, algorithm).unwrap();
    into(okm)
}

// ----
// Concrete implementations
// ----

pub fn op_hmac256_new_key() -> Key {
    // todo maybe we need a context for rng? Maybe also for hs_hash?
    let random = SystemRandom::new();
    let key = hmac::Key::generate(hmac::HMAC_SHA256, &random).unwrap();
    key
}

pub fn op_arbitrary_to_key(key: &Vec<u8>) -> Key {
    Key::new(hmac::HMAC_SHA256, key.as_slice())
}

pub fn op_hmac256(key: &Key, msg: &Vec<u8>) -> Vec<u8> {
    let tag = hmac::sign(&key, msg);
    Vec::from(tag.as_ref())
}

// https://github.com/ctz/rustls/blob/d03bf27e0b520fe73c901d0027bab12753a42bb6/rustls/src/key_schedule.rs#L164
pub fn op_client_handshake_traffic_secret(secret: &hkdf::Prk, hs_hash: &Vec<u8>) -> Prk {
    let secret: hkdf::Prk = derive_secret(
        secret,
        SecretKind::ClientHandshakeTrafficSecret,
        HKDF_SHA256, // todo make configurable
        hs_hash,
        |okm| okm.into(),
    );

    secret
}

pub fn op_client_hello(
    client_version: &ProtocolVersion,
    random: &Random,
    session_id: &SessionID,
    cipher_suites: &Vec<CipherSuite>,
    compression_methods: &Vec<Compression>,
    extensions: &Vec<ClientExtension>,
) -> Message {
    let payload = Handshake(HandshakeMessagePayload {
        typ: HandshakeType::ClientHello,
        payload: HandshakePayload::ClientHello(ClientHelloPayload {
            client_version: client_version.clone(),
            random: random.clone(),
            session_id: session_id.clone(),
            cipher_suites: cipher_suites.clone(),
            compression_methods: compression_methods.clone(),
            extensions: extensions.clone(),
        }),
    });
    Message {
        typ: RecordHandshake,
        version: ProtocolVersion::TLSv1_2,
        payload,
    }
}

pub fn op_alert_description(message: &Message) -> Option<AlertDescription> {
    if let MessagePayload::Alert(payload) = &message.payload {
        Some(payload.description)
    } else {
        None
    }
}

pub fn op_alert_payload(message: &Message) -> Option<AlertMessagePayload> {
    // todo expensive clone action here
    let mut out: Vec<u8> = Vec::new();
    message.encode(&mut out);
    let cloned = Message::read_bytes(out.as_slice()).unwrap();

    if let MessagePayload::Alert(payload) = cloned.payload {
        Some(payload)
    } else {
        None
    }
}

pub fn op_random_cipher_suite() -> CipherSuite {
    *vec![
        CipherSuite::TLS13_AES_128_CCM_SHA256,
        CipherSuite::TLS13_AES_128_CCM_8_SHA256,
        CipherSuite::TLS13_AES_128_GCM_SHA256,
        CipherSuite::TLS13_AES_256_GCM_SHA384,
        CipherSuite::TLS_DHE_RSA_WITH_AES_128_CBC_SHA,
    ]
    .choose(&mut rand::thread_rng())
    .unwrap()
}

pub fn op_random_session_id() -> SessionID {
    let random_data: [u8; 32] = random();
    SessionID::new(&random_data)
}

pub fn op_random_protocol_version() -> ProtocolVersion {
    ProtocolVersion::TLSv1_3
}

pub fn op_random_random_data() -> Random {
    let random_data: [u8; 32] = random();
    Random::from_slice(&random_data)
}

pub fn on_random_cipher_suite() -> CipherSuite {
    *vec![
        CipherSuite::TLS13_AES_128_CCM_SHA256,
        CipherSuite::TLS13_AES_128_CCM_8_SHA256,
        CipherSuite::TLS13_AES_128_GCM_SHA256,
        CipherSuite::TLS13_AES_256_GCM_SHA384,
        CipherSuite::TLS_DHE_RSA_WITH_AES_128_CBC_SHA,
    ]
    .choose(&mut rand::thread_rng())
    .unwrap()
}

pub fn on_compression() -> Compression {
    *vec![Compression::Null, Compression::Deflate, Compression::LSZ]
        .choose(&mut rand::thread_rng())
        .unwrap()
}

pub fn op_server_name_extension(dns_name: &str) -> ClientExtension {
    ClientExtension::ServerName(vec![ServerName {
        typ: ServerNameType::HostName,
        payload: ServerNamePayload::HostName(
            webpki::DNSNameRef::try_from_ascii_str(dns_name)
                .unwrap()
                .to_owned(),
        ),
    }])
}

pub fn op_support_group_extension() -> ClientExtension {
    ClientExtension::NamedGroups(vec![NamedGroup::X25519])
}

pub fn op_signature_algorithm_extension() -> ClientExtension {
    ClientExtension::SignatureAlgorithms(vec![
        SignatureScheme::RSA_PKCS1_SHA256,
        SignatureScheme::RSA_PSS_SHA256,
    ])
}

pub fn op_random_key_share_extension() -> ClientExtension {
    let key = Vec::from(rand::random::<[u8; 32]>()); // 32 byte public key
    ClientExtension::KeyShare(vec![KeyShareEntry {
        group: NamedGroup::X25519,
        payload: PayloadU16::new(key),
    }])
}

pub fn op_supported_versions_extension() -> ClientExtension {
    ClientExtension::SupportedVersions(vec![ProtocolVersion::TLSv1_3])
}

pub fn op_random_extensions() -> Vec<ClientExtension> {
    let server_name: ClientExtension = op_server_name_extension("maxammann.org");

    let supported_groups: ClientExtension = op_support_group_extension();
    let signature_algorithms: ClientExtension = op_signature_algorithm_extension();
    let key_share: ClientExtension = op_random_key_share_extension();
    let supported_versions: ClientExtension = op_supported_versions_extension();


    vec![
        server_name,
        supported_groups,
        signature_algorithms,
        key_share,
        supported_versions,
    ]
}