//! Streaming encryption for large files.
//!
//! This module provides chunk-based encryption that allows processing files
//! of any size without loading the entire file into memory.
//!
//! # Security Design
//!
//! - Each chunk is encrypted independently with a unique nonce
//! - Nonces are derived deterministically from a base nonce + chunk counter
//! - Each chunk maintains AEAD properties (authentication + encryption)
//! - Chunk size is configurable (default: 1 MB)
//! - Maximum file size: 2^64 bytes (nonce counter is u64)
//!
//! # File Format V2
//!
//! ```text
//! [Header]
//!   - Magic bytes: "SCRYPTv2" (8 bytes)
//!   - Version: 0x02 (1 byte)
//!   - Header size: u32 (4 bytes)
//!   - Salt length: u16 (2 bytes)
//!   - Salt: variable
//!   - Base nonce: 12 bytes
//!   - Chunk size: u32 (4 bytes)
//!   - Total chunks: u64 (8 bytes)
//!   - Original file size: u64 (8 bytes)
//!   - Metadata size: u16 (2 bytes)
//!   - Metadata: JSON (compressed, optional)
//!
//! [Chunk 0]
//!   - Chunk index: u64 (8 bytes)
//!   - Data size: u32 (4 bytes)
//!   - Encrypted data + auth tag
//!
//! [Chunk 1]
//!   - Chunk index: u64 (8 bytes)
//!   - Data size: u32 (4 bytes)
//!   - Encrypted data + auth tag
//!
//! ...
//! ```

use crate::config::NONCE_LEN;
use crate::error::{CryptorError, Result};
use std::io::{Read, Write};

/// Default chunk size: 1 MB
pub const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024;

/// Minimum chunk size: 4 KB
pub const MIN_CHUNK_SIZE: usize = 4 * 1024;

/// Maximum chunk size: 16 MB
pub const MAX_CHUNK_SIZE: usize = 16 * 1024 * 1024;

/// Magic bytes for streaming file format v2
pub const MAGIC_BYTES_V2: &[u8] = b"SCRYPTv2";

/// File format version
pub const FORMAT_VERSION: u8 = 0x02;

/// Configuration for streaming encryption/decryption.
#[derive(Debug, Clone, Copy)]
pub struct StreamConfig {
    /// Size of each chunk in bytes
    pub chunk_size: usize,
    /// Whether to enable compression before encryption
    pub compress: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            compress: false,
        }
    }
}

impl StreamConfig {
    /// Creates a new stream configuration with custom chunk size.
    ///
    /// # Arguments
    ///
    /// * `chunk_size` - Size of each chunk in bytes (must be between MIN and MAX)
    ///
    /// # Errors
    ///
    /// Returns an error if chunk size is out of valid range.
    pub fn new(chunk_size: usize) -> Result<Self> {
        if chunk_size < MIN_CHUNK_SIZE || chunk_size > MAX_CHUNK_SIZE {
            return Err(CryptorError::Cryptography(format!(
                "Chunk size {} is out of range [{}, {}]",
                chunk_size, MIN_CHUNK_SIZE, MAX_CHUNK_SIZE
            )));
        }

        Ok(Self {
            chunk_size,
            compress: false,
        })
    }

    /// Enables or disables compression.
    pub fn with_compression(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }

    /// Creates a configuration optimized for fast processing.
    pub fn fast() -> Self {
        Self {
            chunk_size: 4 * 1024 * 1024, // 4 MB chunks
            compress: false,
        }
    }

    /// Creates a configuration optimized for small memory footprint.
    pub fn low_memory() -> Self {
        Self {
            chunk_size: 64 * 1024, // 64 KB chunks
            compress: true,
        }
    }
}

/// Derives a unique nonce for a specific chunk.
///
/// # Security
///
/// This function generates a unique nonce for each chunk by combining:
/// - A base nonce (12 bytes, randomly generated once per file)
/// - A chunk counter (u64, incremented for each chunk)
///
/// The nonce is constructed as: base_nonce[0..8] || (base_nonce[8..12] XOR chunk_counter_bytes)
///
/// This ensures:
/// - Each chunk gets a unique nonce
/// - Nonces are deterministic (same chunk = same nonce)
/// - No nonce reuse within a single file
/// - Maximum file size: 2^64 chunks
///
/// # Arguments
///
/// * `base_nonce` - 12-byte base nonce (randomly generated per file)
/// * `chunk_index` - Index of the chunk (0-based)
///
/// # Returns
///
/// A 12-byte nonce unique to this chunk.
pub fn derive_chunk_nonce(base_nonce: &[u8; NONCE_LEN], chunk_index: u64) -> [u8; NONCE_LEN] {
    let mut nonce = *base_nonce;

    // XOR the last 8 bytes with the chunk counter
    // This ensures each chunk has a unique nonce
    let counter_bytes = chunk_index.to_le_bytes();
    for i in 0..8 {
        nonce[i + 4] ^= counter_bytes[i];
    }

    nonce
}

/// Header information for a streaming encrypted file.
#[derive(Debug, Clone)]
pub struct StreamHeader {
    /// Salt string for key derivation
    pub salt: String,
    /// Base nonce for deriving chunk nonces
    pub base_nonce: [u8; NONCE_LEN],
    /// Size of each chunk in bytes
    pub chunk_size: u32,
    /// Total number of chunks in the file
    pub total_chunks: u64,
    /// Original file size in bytes
    pub original_size: u64,
    /// Optional metadata (JSON)
    pub metadata: Option<String>,
}

impl StreamHeader {
    /// Calculates the number of chunks needed for a given file size.
    pub fn calculate_chunks(file_size: u64, chunk_size: u32) -> u64 {
        let chunk_size = chunk_size as u64;
        (file_size + chunk_size - 1) / chunk_size
    }

    /// Writes the header to a writer.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Magic bytes
        writer.write_all(MAGIC_BYTES_V2)?;

        // Version
        writer.write_all(&[FORMAT_VERSION])?;

        // Calculate and write header size (placeholder for now)
        let header_size_pos = 0u32; // We'll calculate this
        writer.write_all(&header_size_pos.to_le_bytes())?;

        // Salt
        let salt_bytes = self.salt.as_bytes();
        let salt_len = salt_bytes.len() as u16;
        writer.write_all(&salt_len.to_le_bytes())?;
        writer.write_all(salt_bytes)?;

        // Base nonce
        writer.write_all(&self.base_nonce)?;

        // Chunk size
        writer.write_all(&self.chunk_size.to_le_bytes())?;

        // Total chunks
        writer.write_all(&self.total_chunks.to_le_bytes())?;

        // Original file size
        writer.write_all(&self.original_size.to_le_bytes())?;

        // Metadata
        let metadata_bytes = self.metadata.as_ref().map(|s| s.as_bytes()).unwrap_or(&[]);
        let metadata_len = metadata_bytes.len() as u16;
        writer.write_all(&metadata_len.to_le_bytes())?;
        writer.write_all(metadata_bytes)?;

        Ok(())
    }

    /// Reads the header from a reader.
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        // Magic bytes
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;
        if magic != MAGIC_BYTES_V2 {
            return Err(CryptorError::InvalidFormat);
        }

        // Version
        let mut version = [0u8; 1];
        reader.read_exact(&mut version)?;
        if version[0] != FORMAT_VERSION {
            return Err(CryptorError::InvalidFormat);
        }

        // Header size (skip for now)
        let mut header_size_bytes = [0u8; 4];
        reader.read_exact(&mut header_size_bytes)?;

        // Salt length and salt
        let mut salt_len_bytes = [0u8; 2];
        reader.read_exact(&mut salt_len_bytes)?;
        let salt_len = u16::from_le_bytes(salt_len_bytes) as usize;

        let mut salt_bytes = vec![0u8; salt_len];
        reader.read_exact(&mut salt_bytes)?;
        let salt = String::from_utf8(salt_bytes)
            .map_err(|_| CryptorError::InvalidFormat)?;

        // Base nonce
        let mut base_nonce = [0u8; NONCE_LEN];
        reader.read_exact(&mut base_nonce)?;

        // Chunk size
        let mut chunk_size_bytes = [0u8; 4];
        reader.read_exact(&mut chunk_size_bytes)?;
        let chunk_size = u32::from_le_bytes(chunk_size_bytes);

        // Total chunks
        let mut total_chunks_bytes = [0u8; 8];
        reader.read_exact(&mut total_chunks_bytes)?;
        let total_chunks = u64::from_le_bytes(total_chunks_bytes);

        // Original size
        let mut original_size_bytes = [0u8; 8];
        reader.read_exact(&mut original_size_bytes)?;
        let original_size = u64::from_le_bytes(original_size_bytes);

        // Metadata length and metadata
        let mut metadata_len_bytes = [0u8; 2];
        reader.read_exact(&mut metadata_len_bytes)?;
        let metadata_len = u16::from_le_bytes(metadata_len_bytes) as usize;

        let metadata = if metadata_len > 0 {
            let mut metadata_bytes = vec![0u8; metadata_len];
            reader.read_exact(&mut metadata_bytes)?;
            Some(String::from_utf8(metadata_bytes)
                .map_err(|_| CryptorError::InvalidFormat)?)
        } else {
            None
        };

        Ok(Self {
            salt,
            base_nonce,
            chunk_size,
            total_chunks,
            original_size,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_config_validation() {
        assert!(StreamConfig::new(1024).is_err()); // Too small
        assert!(StreamConfig::new(MIN_CHUNK_SIZE).is_ok());
        assert!(StreamConfig::new(DEFAULT_CHUNK_SIZE).is_ok());
        assert!(StreamConfig::new(MAX_CHUNK_SIZE).is_ok());
        assert!(StreamConfig::new(20 * 1024 * 1024).is_err()); // Too large
    }

    #[test]
    fn test_derive_chunk_nonce_uniqueness() {
        let base_nonce = [1u8; NONCE_LEN];

        let nonce0 = derive_chunk_nonce(&base_nonce, 0);
        let nonce1 = derive_chunk_nonce(&base_nonce, 1);
        let nonce2 = derive_chunk_nonce(&base_nonce, 2);

        assert_ne!(nonce0, nonce1);
        assert_ne!(nonce1, nonce2);
        assert_ne!(nonce0, nonce2);
    }

    #[test]
    fn test_derive_chunk_nonce_deterministic() {
        let base_nonce = [42u8; NONCE_LEN];

        let nonce1_a = derive_chunk_nonce(&base_nonce, 100);
        let nonce1_b = derive_chunk_nonce(&base_nonce, 100);

        assert_eq!(nonce1_a, nonce1_b);
    }

    #[test]
    fn test_calculate_chunks() {
        assert_eq!(StreamHeader::calculate_chunks(0, 1024), 0);
        assert_eq!(StreamHeader::calculate_chunks(1024, 1024), 1);
        assert_eq!(StreamHeader::calculate_chunks(1025, 1024), 2);
        assert_eq!(StreamHeader::calculate_chunks(2048, 1024), 2);
        assert_eq!(StreamHeader::calculate_chunks(2049, 1024), 3);
    }

    #[test]
    fn test_stream_header_roundtrip() {
        let header = StreamHeader {
            salt: "test_salt_string".to_string(),
            base_nonce: [42u8; NONCE_LEN],
            chunk_size: 1024 * 1024,
            total_chunks: 100,
            original_size: 100 * 1024 * 1024,
            metadata: Some("{\"compressed\":true}".to_string()),
        };

        let mut buffer = Vec::new();
        header.write_to(&mut buffer).unwrap();

        let mut cursor = std::io::Cursor::new(buffer);
        let decoded = StreamHeader::read_from(&mut cursor).unwrap();

        assert_eq!(header.salt, decoded.salt);
        assert_eq!(header.base_nonce, decoded.base_nonce);
        assert_eq!(header.chunk_size, decoded.chunk_size);
        assert_eq!(header.total_chunks, decoded.total_chunks);
        assert_eq!(header.original_size, decoded.original_size);
        assert_eq!(header.metadata, decoded.metadata);
    }

    #[test]
    fn test_config_presets() {
        let fast = StreamConfig::fast();
        assert_eq!(fast.chunk_size, 4 * 1024 * 1024);
        assert!(!fast.compress);

        let low_mem = StreamConfig::low_memory();
        assert_eq!(low_mem.chunk_size, 64 * 1024);
        assert!(low_mem.compress);
    }
}
