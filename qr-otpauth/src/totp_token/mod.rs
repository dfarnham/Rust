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

/// Returns a list of (token, issuer) tuples
/// otpauth can be of form "otpauth-migration://offline" or "otpauth://totp"
pub fn generate_tokens(otpauth: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let mut token_issuer = vec![];

    // otpauth-migration contains a base-64 payload encoding multiple accounts
    if otpauth.contains("otpauth-migration://offline") {
        // retreive the list of accounts
        let accounts = google_authenticator_converter::process_data(otpauth)?;

        // build and issue totp() queries from the account secrets
        for account in accounts {
            let token = totp(&format!("secret={}", account.secret))?;
            token_issuer.push((token, account.issuer));
        }
    } else {
        let token = totp(otpauth)?;
        let issuer = uri_param(otpauth, "issuer=").unwrap_or_default();
        token_issuer.push((token, issuer));
    }

    Ok(token_issuer)
}

/// Return the named parameter value in the otpauth string
fn uri_param(otpauth: &str, name: &str) -> Option<String> {
    otpauth.split(name).nth(1)?.split('&').next().map(str::to_string)
}

/// Extract the Base-32 'secret=' and optional 'algorithm={SHA1, SHA512, SHA256}'
/// to generate a token at SystemTime::now()
fn totp(otpauth: &str) -> Result<String, Box<dyn Error>> {
    // Time now
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // Extract the Secret
    match uri_param(otpauth, "secret=") {
        // Compute the token given the secret and SHA algorithm
        Some(secret) => time_token(
            now,
            &secret,
            // supply the algorith, default to SHA1
            match uri_param(otpauth, "algorithm=") {
                Some(sha) if sha.to_lowercase().contains("sha256") => Algorithm::SHA256,
                Some(sha) if sha.to_lowercase().contains("sha512") => Algorithm::SHA512,
                _ => Algorithm::SHA1,
            },
        ),
        _ => Err("totp() no secret".into()),
    }
}

/// Generate a time based token from the Base-32 secret and Algorithm
fn time_token(time: u64, secret_b32: &str, algorithm: Algorithm) -> Result<String, Box<dyn Error>> {
    let alphabet = base32::Alphabet::RFC4648 { padding: false };
    let secret_bytes = base32::decode(alphabet, secret_b32).expect("Base-32 secret");

    // digits=6, period=30
    let bytes = algorithm.sign(&secret_bytes, &(time / 30).to_be_bytes());
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
