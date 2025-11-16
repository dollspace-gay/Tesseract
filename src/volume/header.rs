/// Volume header format for encrypted volumes
///
/// The header contains all metadata necessary to mount and decrypt an encrypted volume.
/// It is stored at the beginning of the volume file.

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::io::{self, Read, Write};
use thiserror::Error;

/// Magic bytes to identify Secure Cryptor volume files
/// "SECVOL01" in ASCII
const MAGIC: [u8; 8] = [0x53, 0x45, 0x43, 0x56, 0x4F, 0x4C, 0x30, 0x31];

/// Current volume format version
const VERSION: u32 = 1;

/// Size of the volume header in bytes (4KB aligned)
pub const HEADER_SIZE: usize = 4096;

/// Cipher algorithm identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CipherAlgorithm {
    /// AES-256-GCM (default)
    Aes256Gcm = 1,
}

/// Volume header containing all metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeHeader {
    /// Magic bytes for file identification
    magic: [u8; 8],

    /// Header format version
    version: u32,

    /// Cipher algorithm used for encryption
    cipher: CipherAlgorithm,

    /// Salt for key derivation (32 bytes for Argon2id)
    salt: [u8; 32],

    /// Initialization vector for header encryption (12 bytes for AES-GCM)
    header_iv: [u8; 12],

    /// Total volume size in bytes (excluding header)
    volume_size: u64,

    /// Sector size in bytes (typically 512 or 4096)
    sector_size: u32,

    /// Creation timestamp (Unix epoch seconds)
    created_at: u64,

    /// Last modification timestamp (Unix epoch seconds)
    modified_at: u64,

    /// Reserved space for future use (256 bytes)
    #[serde(with = "BigArray")]
    reserved: [u8; 256],
}

/// Errors that can occur when working with volume headers
#[derive(Debug, Error)]
pub enum HeaderError {
    /// Invalid magic bytes (not a Secure Cryptor volume)
    #[error("Invalid magic bytes: not a Secure Cryptor volume file")]
    InvalidMagic,

    /// Unsupported volume format version
    #[error("Unsupported volume version: {0}")]
    UnsupportedVersion(u32),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    /// Header size mismatch
    #[error("Header size mismatch: expected {expected}, got {actual}")]
    SizeMismatch { expected: usize, actual: usize },
}

impl VolumeHeader {
    /// Creates a new volume header with the given parameters
    ///
    /// # Arguments
    ///
    /// * `volume_size` - Total size of the encrypted volume in bytes
    /// * `sector_size` - Size of each sector in bytes (typically 512 or 4096)
    /// * `salt` - 32-byte salt for key derivation
    /// * `header_iv` - 12-byte IV for header encryption
    ///
    /// # Returns
    ///
    /// A new `VolumeHeader` instance
    pub fn new(
        volume_size: u64,
        sector_size: u32,
        salt: [u8; 32],
        header_iv: [u8; 12],
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time before Unix epoch")
            .as_secs();

        Self {
            magic: MAGIC,
            version: VERSION,
            cipher: CipherAlgorithm::Aes256Gcm,
            salt,
            header_iv,
            volume_size,
            sector_size,
            created_at: now,
            modified_at: now,
            reserved: [0u8; 256],
        }
    }

    /// Serializes the header to bytes
    ///
    /// The header is serialized to exactly HEADER_SIZE bytes, with
    /// padding added if necessary.
    ///
    /// # Returns
    ///
    /// A byte vector of exactly HEADER_SIZE bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, HeaderError> {
        let mut serialized = bincode::serialize(self)?;

        // Ensure the header is exactly HEADER_SIZE bytes
        if serialized.len() > HEADER_SIZE {
            return Err(HeaderError::SizeMismatch {
                expected: HEADER_SIZE,
                actual: serialized.len(),
            });
        }

        // Pad with zeros to reach HEADER_SIZE
        serialized.resize(HEADER_SIZE, 0);

        Ok(serialized)
    }

    /// Deserializes a header from bytes
    ///
    /// # Arguments
    ///
    /// * `bytes` - Byte slice containing the serialized header
    ///
    /// # Returns
    ///
    /// A deserialized `VolumeHeader` instance
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The bytes are not exactly HEADER_SIZE long
    /// - The magic bytes are invalid
    /// - The version is unsupported
    /// - Deserialization fails
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, HeaderError> {
        if bytes.len() != HEADER_SIZE {
            return Err(HeaderError::SizeMismatch {
                expected: HEADER_SIZE,
                actual: bytes.len(),
            });
        }

        let header: Self = bincode::deserialize(bytes)?;

        // Validate magic bytes
        if header.magic != MAGIC {
            return Err(HeaderError::InvalidMagic);
        }

        // Check version compatibility
        if header.version != VERSION {
            return Err(HeaderError::UnsupportedVersion(header.version));
        }

        Ok(header)
    }

    /// Writes the header to a writer
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer to write the header to
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or writing fails
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), HeaderError> {
        let bytes = self.to_bytes()?;
        writer.write_all(&bytes)?;
        Ok(())
    }

    /// Reads a header from a reader
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader to read the header from
    ///
    /// # Returns
    ///
    /// A deserialized `VolumeHeader` instance
    ///
    /// # Errors
    ///
    /// Returns an error if reading or deserialization fails
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, HeaderError> {
        let mut bytes = vec![0u8; HEADER_SIZE];
        reader.read_exact(&mut bytes)?;
        Self::from_bytes(&bytes)
    }

    /// Updates the modification timestamp to the current time
    pub fn touch(&mut self) {
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time before Unix epoch")
            .as_secs();
    }

    /// Returns the salt used for key derivation
    pub fn salt(&self) -> &[u8; 32] {
        &self.salt
    }

    /// Returns the header IV
    pub fn header_iv(&self) -> &[u8; 12] {
        &self.header_iv
    }

    /// Returns the total volume size in bytes
    pub fn volume_size(&self) -> u64 {
        self.volume_size
    }

    /// Returns the sector size in bytes
    pub fn sector_size(&self) -> u32 {
        self.sector_size
    }

    /// Returns the creation timestamp (Unix epoch seconds)
    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    /// Returns the last modification timestamp (Unix epoch seconds)
    pub fn modified_at(&self) -> u64 {
        self.modified_at
    }

    /// Returns the cipher algorithm
    pub fn cipher(&self) -> CipherAlgorithm {
        self.cipher
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_header_creation() {
        let salt = [1u8; 32];
        let iv = [2u8; 12];
        let header = VolumeHeader::new(1024 * 1024 * 1024, 4096, salt, iv);

        assert_eq!(header.magic, MAGIC);
        assert_eq!(header.version, VERSION);
        assert_eq!(header.cipher, CipherAlgorithm::Aes256Gcm);
        assert_eq!(header.salt, salt);
        assert_eq!(header.header_iv, iv);
        assert_eq!(header.volume_size, 1024 * 1024 * 1024);
        assert_eq!(header.sector_size, 4096);
    }

    #[test]
    fn test_header_serialization() {
        let salt = [1u8; 32];
        let iv = [2u8; 12];
        let header = VolumeHeader::new(1024 * 1024 * 1024, 4096, salt, iv);

        let bytes = header.to_bytes().unwrap();
        assert_eq!(bytes.len(), HEADER_SIZE);

        let deserialized = VolumeHeader::from_bytes(&bytes).unwrap();
        assert_eq!(deserialized.salt, header.salt);
        assert_eq!(deserialized.header_iv, header.header_iv);
        assert_eq!(deserialized.volume_size, header.volume_size);
        assert_eq!(deserialized.sector_size, header.sector_size);
    }

    #[test]
    fn test_header_write_read() {
        let salt = [3u8; 32];
        let iv = [4u8; 12];
        let header = VolumeHeader::new(2 * 1024 * 1024 * 1024, 512, salt, iv);

        let mut buffer = Vec::new();
        header.write_to(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let read_header = VolumeHeader::read_from(&mut cursor).unwrap();

        assert_eq!(read_header.salt, salt);
        assert_eq!(read_header.header_iv, iv);
        assert_eq!(read_header.volume_size, 2 * 1024 * 1024 * 1024);
        assert_eq!(read_header.sector_size, 512);
    }

    #[test]
    fn test_invalid_magic() {
        let mut bytes = vec![0u8; HEADER_SIZE];
        bytes[0..8].copy_from_slice(b"INVALID!");

        let result = VolumeHeader::from_bytes(&bytes);
        assert!(matches!(result, Err(HeaderError::InvalidMagic)));
    }

    #[test]
    fn test_touch() {
        let salt = [1u8; 32];
        let iv = [2u8; 12];
        let mut header = VolumeHeader::new(1024, 512, salt, iv);

        let original_modified = header.modified_at;
        std::thread::sleep(std::time::Duration::from_secs(1));
        header.touch();

        assert!(header.modified_at > original_modified);
        assert_eq!(header.created_at, original_modified);
    }
}
