use crate::crc::checksum;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DecodeError {
    // TODO: Add meaningful errors for each problem that can occur.
    Invalid,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum Strkey {
    PublicKey(PublicKey),
}

impl Strkey {
    pub fn to_string(&self) -> String {
        match self {
            Self::PublicKey(x) => x.to_string(),
        }
    }

    pub fn from_string(s: &str) -> Result<Self, DecodeError> {
        let (ver, payload) = decode(s)?;
        match ver {
            Version::PublicKeyEd25519 => {
                match <[u8; 32]>::try_from(payload) {
                    Ok(x) => Ok(Self::PublicKey(PublicKey(x))),
                    Err(_) => Err(DecodeError::Invalid),
                }
            },
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct PublicKey(pub [u8; 32]);

impl PublicKey {
    pub fn to_string(&self) -> String {
        encode(Version::PublicKeyEd25519, &self.0)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Version {
    PublicKeyEd25519 = version::PUBLIC_KEY_ED25519,
}

impl Version {
    fn try_from(b: u8) -> Result<Self, DecodeError> {
        match b {
            version::PUBLIC_KEY_ED25519 => Ok(Version::PublicKeyEd25519),
            _ => Err(DecodeError::Invalid),
        }
    }
}

mod version {
    use super::public_key_alg::*;
    use super::typ::*;

    pub const PUBLIC_KEY_ED25519: u8 = PUBLIC_KEY | ED25519;
}

mod typ {
    pub const PUBLIC_KEY: u8 = 6 << 3;
}

mod public_key_alg {
    pub const ED25519: u8 = 0;
}

// TODO: Could encode and decode, and the functions upstream that call them, be
// const fn's?

fn encode(v: Version, payload: &[u8]) -> String {
    let mut d: Vec<u8> = Vec::with_capacity(1 + payload.len() + 2);
    d.push(v as u8);
    d.extend_from_slice(&payload);
    d.extend_from_slice(&checksum(&d));
    base32::encode(base32::Alphabet::RFC4648 { padding: false }, &d)
}

fn decode(s: &str) -> Result<(Version, Vec<u8>), DecodeError> {
    // TODO: Look at what other base32 implementations are available, because
    // this one allows for decoding of non-canonical base32 strings, and doesn't
    // come with helpful methods for validating the length is canonical.
    let data = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &s);
    if let Some(data) = data {
        let s_canonical_len = (data.len() * 8 + 4) / 5;
        if s.len() != s_canonical_len {
            return Err(DecodeError::Invalid);
        }
        if data.len() < 3 {
            return Err(DecodeError::Invalid);
        }
        let ver = Version::try_from(data[0])?;
        let (data_without_crc, crc_actual) = data.split_at(data.len() - 2);
        let crc_expect = checksum(&data_without_crc);
        if crc_actual != crc_expect {
            return Err(DecodeError::Invalid);
        }
        let payload = &data_without_crc[1..];
        Ok((ver, payload.to_vec()))
    } else {
        Err(DecodeError::Invalid)
    }
}
