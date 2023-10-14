use hmac::{Hmac, Mac};
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::google_authenticator_converter;

// Create aliases
type HmacSha1 = Hmac<sha1::Sha1>;
type HmacSha256 = Hmac<sha2::Sha256>;
type HmacSha512 = Hmac<sha2::Sha512>;

// enum adopted/modified from https://github.com/constantoine/totp-rs/
enum Algorithm {
    SHA1,
    SHA256,
    SHA512,
}
impl Algorithm {
    fn hash<D: Mac>(mut digest: D, data: &[u8]) -> Vec<u8> {
        digest.update(data);
        digest.finalize().into_bytes().to_vec()
    }

    fn sign(&self, key: &[u8], data: &[u8]) -> Vec<u8> {
        match self {
            Self::SHA1 => Self::hash(HmacSha1::new_from_slice(key).unwrap(), data),
            Self::SHA256 => Self::hash(HmacSha256::new_from_slice(key).unwrap(), data),
            Self::SHA512 => Self::hash(HmacSha512::new_from_slice(key).unwrap(), data),
        }
    }
}

pub fn display_token(otpauth: &str) -> Result<(), Box<dyn Error>> {
    if otpauth.contains("otpauth-migration://offline") {
        let accounts = google_authenticator_converter::process_data(otpauth)?;
        for account in accounts {
            let otpauth = format!("secret={}", account.secret);
            let token = totp(&otpauth)?;
            println!("{account:?}\ntotp = {token}");
        }
    } else {
        let token = totp(otpauth)?;
        println!("totp = {token}");
        // Output 20 ~
        println!("{:~^20}", "");
    }
    Ok(())
}

/// Will generate a token given the provided Base-32 secret and Algorithm for the current time
fn time_token(secret_b32: &str, algorithm: Algorithm) -> Result<String, Box<dyn Error>> {
    let alphabet = base32::Alphabet::RFC4648 { padding: false };

    // TODO:
    // What's the spec say?
    // I have 16-byte Base-32 secret data I need decoded so I zero filled when short
    let mut secret_bytes = base32::decode(alphabet, secret_b32).expect("Base-32 secret");
    if secret_bytes.len() < 20 {
        secret_bytes.resize(20, 0);
    }

    // current time
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // digits=6, period=30
    let bytes = algorithm.sign(&secret_bytes, &(now / 30).to_be_bytes());
    match bytes.last() {
        Some(n) => {
            let offset = (n & 0xf) as usize;
            let result = u32::from_be_bytes(bytes[offset..offset + 4].try_into()?);
            let token = (result & 0x7fff_ffff) % 1000000;
            Ok(format!("{token:0>6}"))
        }
        None => Err("time_token() failed".into()),
    }
}

fn totp(otpauth: &str) -> Result<String, Box<dyn Error>> {
    // Extract the Algorithm, default to SHA1
    let algorithm = match otpauth.to_lowercase() {
        s if s.contains("algorithm=sha256") => Algorithm::SHA256,
        s if s.contains("algorithm=sha512") => Algorithm::SHA512,
        _ => Algorithm::SHA1,
    };

    // Extract the Secret and compute the 6 digit token
    match otpauth.split("secret=").nth(1) {
        Some(s) => time_token(s.split('&').next().unwrap(), algorithm),
        _ => Err("totp failed".into()),
    }
}
