use core::fmt;
use std::fmt::Display;

use crate::chunk_type::ChunkType;
use crate::Error;
use crate::Result;

use crc32fast::Hasher;
#[derive(Debug, PartialEq)]
pub struct Chunk {
    chunk_type: ChunkType,
    data: Vec<u8>,
}

impl Chunk {
    pub const DATA_LENGTH_BYTES: usize = 4;
    pub const CHUNK_TYPE_BYTES: usize = 4;
    pub const CRC_BYTES: usize = 4;

    /// Total size of the of the metadata making up a chunk
    pub const METADATA_BYTES: usize =
        Chunk::DATA_LENGTH_BYTES + Chunk::CHUNK_TYPE_BYTES + Chunk::CRC_BYTES;

    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        Chunk {
            chunk_type: chunk_type,
            data: data,
        }
    }
    pub fn length(&self) -> u32 {
        self.data.len().try_into().unwrap()
    }
    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
    pub fn crc(&self) -> u32 {
        let bytes: Vec<u8> = self
            .chunk_type
            .bytes()
            .iter()
            .chain(self.data.iter())
            .copied()
            .collect();

        let mut hasher = Hasher::new();
        hasher.update(&bytes);
        hasher.finalize()
    }
    pub fn data_as_string(&self) -> Result<String> {
        let ret_string = std::str::from_utf8(&self.data)?;
        Ok(ret_string.to_string())
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        self.length()
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type.bytes().iter())
            .chain(self.data.iter())
            .chain(self.crc().to_be_bytes().iter())
            .copied()
            .collect()
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        if value.len() < 4 {
            return Err(Box::from(ChunkError::InputTooSmall));
        }

        // consume first 4 bytes as data length
        let (data_length, value) = value.split_at(4);
        let data_length = u32::from_be_bytes(data_length.try_into()?) as usize;

        // consume next 4 bytes as chunk type
        let (chunk_type_bytes, value) = value.split_at(4);

        let chunk_type_bytes: [u8; 4] = chunk_type_bytes.try_into()?;
        let chunk_type: ChunkType = ChunkType::try_from(chunk_type_bytes)?;

        if !chunk_type.is_valid() {
            return Err(Box::from(ChunkError::InvalidChunkType));
        }

        let (data, value) = value.split_at(data_length);
        let (crc_bytes, _) = value.split_at(4);

        // validate CRC
        let new = Self {
            chunk_type,
            data: data.into(),
        };

        let actual_crc = new.crc();
        let expected_crc = u32::from_be_bytes(crc_bytes.try_into()?);

        if expected_crc != actual_crc {
            return Err(Box::from(ChunkError::InvalidCrc(expected_crc, actual_crc)));
        }

        Ok(new)
    }
}

#[derive(Debug)]
pub enum ChunkError {
    InputTooSmall,
    InvalidCrc(u32, u32),
    InvalidChunkType,
}

impl std::error::Error for ChunkError {}

impl Display for ChunkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ChunkError::InputTooSmall => {
                write!(f, "At least 12 bytes must be supplied to construct a chunk")
            }
            ChunkError::InvalidCrc(expected, actual) => write!(
                f,
                "Invalid CRC when constructing chunk. Expected {} but found {}",
                expected, actual
            ),
            ChunkError::InvalidChunkType => write!(f, "Invalid chunk type"),
        }
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Chunk {{",)?;
        writeln!(f, "  Length: {}", self.length())?;
        writeln!(f, "  Type: {}", self.chunk_type())?;
        writeln!(f, "  Data: {} bytes", self.data().len())?;
        writeln!(f, "  Crc: {}", self.crc())?;
        writeln!(f, "}}",)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
