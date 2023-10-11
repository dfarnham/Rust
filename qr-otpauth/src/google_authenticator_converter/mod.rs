//! # Google Authenticator Converter
//!
//! -   Extract name, secret and issuer from a Google Authenticator migration QR code
//!
//! ### Example
//!
//! ```rust
//! use google_authenticator_converter::{extract_data_from_uri, process_data, Account};
//!
//! let qr_code = "otpauth-migration://offline?data=CjMKCkhlbGxvId6tvu8SGFRlc3QxOnRlc3QxQGV4YW1wbGUxLmNvbRoFVGVzdDEgASgBMAIKMwoKSGVsbG8h3q2%2B8BIYVGVzdDI6dGVzdDJAZXhhbXBsZTIuY29tGgVUZXN0MiABKAEwAgozCgpIZWxsbyHerb7xEhhUZXN0Mzp0ZXN0M0BleGFtcGxlMy5jb20aBVRlc3QzIAEoATACEAEYASAAKI3orYEE";
//!
//! let accounts = process_data(&qr_code)?;
//!
//! for account in accounts {
//!     println!("{0} {1} {2}", account.name, account.secret, account.issuer);
//! }
//!

use base64::{engine::general_purpose, Engine};
use protobuf::Message;

mod proto;

#[derive(Debug)]
pub struct Account {
    pub name: String,
    pub secret: String,
    pub issuer: String,
}

/// Convert a Google Authenticator migration QR code string to a list of accounts
pub fn process_data(string: &str) -> Result<Vec<Account>, Box<dyn std::error::Error>> {
    let alphabet = base32::Alphabet::RFC4648 { padding: false };

    let encoded_data = extract_data_from_uri(string)?;
    let decoded_data = general_purpose::STANDARD.decode(encoded_data)?;
    let migration_payload = proto::google_auth::MigrationPayload::parse_from_bytes(&decoded_data)?;

    Ok(migration_payload
        .otp_parameters
        .into_iter()
        .map(|a| Account {
            name: a.name,
            secret: base32::encode(alphabet, &a.secret),
            issuer: a.issuer,
        })
        .collect())
}

pub fn extract_data_from_uri(uri: &str) -> Result<String, Box<dyn std::error::Error>> {
    match uri.split("data=").nth(1) {
        Some(encoded_data) => Ok(urlencoding::decode(encoded_data)?.into()),
        _ => Err("No data found in URI".into()),
    }
}
