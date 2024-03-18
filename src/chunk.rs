use std::fmt::Display;

use crate::chunk_type::ChunkType;

use crc::{Crc, CRC_32_ISO_HDLC};

#[derive(Clone)]
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    chunk_data: Vec<u8>,
    crc: u32,
}

impl TryFrom<&[u8]> for Chunk {
    type Error = super::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut iter = value.iter();

        let length = u32::from_be_bytes(
            iter.by_ref()
                .take(4)
                .copied()
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        // println!("{:?}", &value[0..4]);

        let chunk_type_bytes: [u8; 4] =
            iter.by_ref().take(4).copied().collect::<Vec<u8>>()[..].try_into()?;
        let chunk_type: ChunkType = chunk_type_bytes.try_into()?;
        let data = iter
            .by_ref()
            .take(length as usize)
            .copied()
            .collect::<Vec<u8>>();
        // eprintln!("{} <|< {} <|< {}", value.len(), length, 4 + 4 + data.len());
        let remaining_bytes = iter.copied().take(4) .collect::<Vec<u8>>();
        let original_crc =
            u32::from_be_bytes(remaining_bytes.try_into().unwrap());
        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let mut digest = crc.digest();
        digest.update(&chunk_type_bytes);
        digest.update(&data);
        let crc = digest.finalize();
        if crc == original_crc {
            Ok(Self {
                length,
                chunk_type,
                chunk_data: data,
                crc,
            })
        } else {
            Err("Invalid chunk CRC".into())
        }
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Chunk:",)?;
        writeln!(f, "    length: {}", self.length)?;
        writeln!(f, "    chunk_type: {}", self.chunk_type)?;
        writeln!(f, "    chunk_data: {:?}", self.chunk_data)?;
        writeln!(f, "    crc: {}", self.crc)
    }
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        let length = data.len().try_into().unwrap();
        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let mut digest = crc.digest();
        digest.update(&chunk_type.bytes());
        digest.update(&data);
        let crc = digest.finalize();
        Self {
            length,
            chunk_type,
            chunk_data: data,
            crc,
        }
    }
    pub fn length(&self) -> u32 {
        self.length
    }
    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }
    pub fn data(&self) -> &[u8] {
        &self.chunk_data
    }
    pub fn crc(&self) -> u32 {
        self.crc
    }
    pub fn data_as_string(&self) -> crate::Result<String> {
        let data = self
            .chunk_data
            .iter()
            .map(|byte| *byte as char)
            .collect::<String>();
        Ok(data)
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let length = self.length.to_be_bytes();
        let chunk_type = self.chunk_type.bytes();
        let data = self.chunk_data.clone().into_iter();
        let crc = self.crc.to_be_bytes();
        length
            .into_iter()
            .chain(chunk_type)
            .chain(data)
            .chain(crc)
            .collect()
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
