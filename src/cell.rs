#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Celula {
    pub hash: [u8; 32],
    pub salt: [u8; 16],
    pub genoma: u32,
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl Celula {
    pub const SIZE: usize = 64;

    pub fn new(hash: [u8; 32], salt: [u8; 16], genoma: u32, x: u32, y: u32, z: u32) -> Self {
        Self {
            hash,
            salt,
            genoma,
            x,
            y,
            z,
        }
    }

    pub fn serialize(&self) -> [u8; Self::SIZE] {
        let mut buffer = [0u8; Self::SIZE];
        buffer[..32].copy_from_slice(&self.hash);
        buffer[32..48].copy_from_slice(&self.salt);
        buffer[48..52].copy_from_slice(&self.genoma.to_le_bytes());
        buffer[52..56].copy_from_slice(&self.x.to_le_bytes());
        buffer[56..60].copy_from_slice(&self.y.to_le_bytes());
        buffer[60..64].copy_from_slice(&self.z.to_le_bytes());
        buffer
    }

    pub fn deserialize(buffer: [u8; Self::SIZE]) -> Self {
        let mut hash = [0u8; 32];
        let mut salt = [0u8; 16];
        hash.copy_from_slice(&buffer[..32]);
        salt.copy_from_slice(&buffer[32..48]);

        Self {
            hash,
            salt,
            genoma: u32::from_le_bytes(buffer[48..52].try_into().expect("fixed slice size")),
            x: u32::from_le_bytes(buffer[52..56].try_into().expect("fixed slice size")),
            y: u32::from_le_bytes(buffer[56..60].try_into().expect("fixed slice size")),
            z: u32::from_le_bytes(buffer[60..64].try_into().expect("fixed slice size")),
        }
    }

    pub fn with_secret(salt: [u8; 16], secret: &[u8], genoma: u32, x: u32, y: u32, z: u32) -> Self {
        Self::new(Self::hash_secret(&salt, secret), salt, genoma, x, y, z)
    }

    pub fn hash_secret(salt: &[u8; 16], secret: &[u8]) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(salt);
        hasher.update(secret);
        *hasher.finalize().as_bytes()
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::Celula;

    #[test]
    fn serialize_round_trip() {
        let cell = Celula::with_secret([7u8; 16], b"clave", 42, 10, 20, 30);
        let decoded = Celula::deserialize(cell.serialize());
        assert_eq!(decoded, cell);
    }

    #[test]
    fn explicit_constructor_round_trip() {
        let salt = [9u8; 16];
        let hash = Celula::hash_secret(&salt, b"clave");
        let cell = Celula::new(hash, salt, 7, 1, 2, 3);
        let decoded = Celula::deserialize(cell.serialize());
        assert_eq!(decoded, cell);
    }
}