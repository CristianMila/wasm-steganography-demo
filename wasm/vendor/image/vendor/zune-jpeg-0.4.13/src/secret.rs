const U64_BYTES: u64 = (u64::BITS as u64) / 8;

#[derive(Debug)]
pub struct Secret {
    bytes: Vec<u8>,
    expected_secret_len: Option<u64>,
    completed: bool
}

impl Secret {
    pub fn new() -> Secret {
        Secret {
            bytes: vec![],
            expected_secret_len: None,
            completed: false
        }
    }

    pub fn from_array(arr: &[u8]) -> Secret {
        let expected_secret_len = get_expected_str_len(arr).ok();

        Secret {
            bytes: arr.to_vec(),
            expected_secret_len,
            completed: match expected_secret_len {
                Some(expected_len) => arr.len() as u64 == expected_len + U64_BYTES,
                None => false
            }
        }
    }

    pub fn push_byte(&mut self, byte: u8) -> Result<(), SecretErrors> {
        if self.is_complete() {
            return Err(SecretErrors::Overflow);
        }

        self.bytes.push(byte);

        match self.expected_secret_len {
            None => {
                if self.bytes.len() >= U64_BYTES as usize {
                    self.expected_secret_len = get_expected_str_len(self.bytes.as_slice()).ok();
                }
            },
            Some(len) => {
                self.completed = len + U64_BYTES == self.bytes.len() as u64;
            }
        }

        Ok(())
    }

    pub fn get_as_string(self) -> Result<String, SecretErrors> {
        if !self.is_complete() {
            return Err(SecretErrors::Incomplete);
        }

        let secret = str::from_utf8(&self.bytes[U64_BYTES as usize..]).map_err(|_| SecretErrors::Utf8Error)?;

        Ok(secret.to_string())
    }

    pub fn is_complete(&self) -> bool {
        self.completed
    }
}

pub fn get_expected_str_len(bytes: &[u8]) -> Result<u64, SecretErrors> {
    if bytes.len() < U64_BYTES as usize {
        return Err(SecretErrors::Underflow);
    }

    let len_bytes: [u8; 8] = bytes[..U64_BYTES as usize].try_into().unwrap();

    Ok(u64::from_le_bytes(len_bytes))
}

#[derive(Debug)]
pub enum SecretErrors {
    Overflow,
    Underflow,
    Incomplete,
    Utf8Error
}

#[cfg(test)]
mod tests {
    use super::Secret;

    #[test]
    fn push_enough_bytes_to_get_valid_len() {
        let mut secret = Secret::new();
        let _ = secret.push_byte(0x3);
        let _ = secret.push_byte(0x0);
        let _ = secret.push_byte(0x0);
        let _ = secret.push_byte(0x0);
        let _ = secret.push_byte(0x0);
        let _ = secret.push_byte(0x0);
        let _ = secret.push_byte(0x0);
        let _ = secret.push_byte(0x0);
        let _ = secret.push_byte(0x0);

        let expected_len = super::get_expected_str_len(&secret.bytes).expect("error getting str len from first 8 bytes");

        assert_eq!(3, expected_len);
    }
}
