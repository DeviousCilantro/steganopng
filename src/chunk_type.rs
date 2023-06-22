use std::str::FromStr;
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkType {
    pub ancillary: char,
    pub private: char,
    pub reserved: char,
    pub safe_to_copy: char,
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = &'static str;

    fn try_from(c: [u8; 4]) -> Result<Self, Self::Error>  {
        let ancillary    = c[0_usize];
        let private      = c[1_usize];
        let reserved     = c[2_usize];
        let safe_to_copy = c[3_usize];

        let valid  = |ascii:  u8| (65..=90).contains(&ascii) || (97..=122).contains(&ascii);

        if !(valid(ancillary) && valid(private) && valid(reserved) && valid(safe_to_copy)) {
            return Err("ChunkType codes are restricted to consist of uppercase or lowercase ASCII letters.");
        };

        Ok(Self {
            ancillary:    char::from(ancillary),
            private:      char::from(private),
            reserved:     char::from(reserved),
            safe_to_copy: char::from(safe_to_copy),
        })
    }
}

impl FromStr for ChunkType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ancillary    = s.chars().next().unwrap();
        let private      = s.chars().nth(1).unwrap();
        let reserved     = s.chars().nth(2).unwrap();
        let safe_to_copy = s.chars().nth(3).unwrap();

        let valid  = |ascii:  u8| (65..=90).contains(&ascii) || (97..=122).contains(&ascii);

        if !(valid(ancillary as u8) && valid(private as u8) && valid(reserved as u8) && valid(safe_to_copy as u8)) {
            return Err("ChunkType codes are restricted to consist of uppercase or lowercase ASCII letters.");
        };

        if s.len() != 4  {
            return Err("ChunkType codes are 4 bytes long.");
        }

        Ok(Self {
            ancillary,
            private,
            reserved,
            safe_to_copy,
        })
    }
}

impl fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}{}", self.ancillary, self.private, self.reserved, self.safe_to_copy)
    }
}

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        let mut bytes: [u8; 4] = [0; 4];
        bytes[0] = self.ancillary  as u8;
        bytes[1] = self.private    as u8;
        bytes[2] = self.reserved   as u8;
        bytes[3] = self.safe_to_copy as u8;

        bytes
    }

    pub fn is_valid(&self) -> bool {
        let ancillary  = self.ancillary; 
        let private    = self.private; 
        let reserved   = self.reserved; 
        let safe_to_copy = self.safe_to_copy; 

        let valid  = |ascii:  u8| (65..=90).contains(&ascii) || (97..=122).contains(&ascii);

        valid(ancillary as u8) && valid(private as u8) && (reserved as u8 >= 65 && reserved as u8 <= 90) && valid(safe_to_copy as u8)
    }

    #[cfg(test)]
    pub fn is_critical(&self) -> bool {
        self.ancillary as u8 >= 65 && self.ancillary as u8 <= 90
    }

    #[cfg(test)]
    pub fn is_public(&self) -> bool {
        self.private as u8 >= 65 && self.private as u8 <= 90
    }

    #[cfg(test)]
    pub fn is_reserved_bit_valid(&self) -> bool {
        self.reserved as u8 >= 65 && self.reserved as u8 <= 90
    }

    #[cfg(test)]
    pub fn is_safe_to_copy(&self) -> bool {
        self.safe_to_copy as u8 >= 97 && self.safe_to_copy as u8 <= 122
    }

    pub fn convert_to_fixed_slice(v: &[u8]) -> [u8; 4] {
        v.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
