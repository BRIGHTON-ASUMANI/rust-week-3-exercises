use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct CompactSize {
    pub value: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BitcoinError {
    InsufficientBytes,
    InvalidFormat,
}

impl CompactSize {
    pub fn new(value: u64) -> Self {
        // TODO: Construct a CompactSize from a u64 value
        CompactSize { value }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Encode according to Bitcoin's CompactSize format:
        // [0x00–0xFC] => 1 byte
        // [0xFDxxxx] => 0xFD + u16 (2 bytes)
        // [0xFExxxxxxxx] => 0xFE + u32 (4 bytes)
        // [0xFFxxxxxxxxxxxxxxxx] => 0xFF + u64 (8 bytes)
        let v = self.value;
        if v <= 0xFC {
            vec![v as u8]
        } else if v <= 0xFFFF {
            let mut bytes = vec![0xFD];
            bytes.extend_from_slice(&(v as u16).to_le_bytes());
            bytes
        } else if v <= 0xFFFF_FFFF {
            let mut bytes = vec![0xFE];
            bytes.extend_from_slice(&(v as u32).to_le_bytes());
            bytes
        } else {
            let mut bytes = vec![0xFF];
            bytes.extend_from_slice(&v.to_le_bytes());
            bytes
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Decode CompactSize, returning value and number of bytes consumed.
        // First check if bytes is empty.
        // Check that enough bytes are available based on prefix.
        if bytes.is_empty() {
            return Err(BitcoinError::InsufficientBytes);
        }

        match bytes[0] {
            n @ 0x00..=0xFC => Ok((CompactSize::new(n as u64), 1)),
            0xFD => {
                if bytes.len() < 3 {
                    Err(BitcoinError::InsufficientBytes)
                } else {
                    let val = u16::from_le_bytes([bytes[1], bytes[2]]) as u64;
                    Ok((CompactSize::new(val), 3))
                }
            }
            0xFE => {
                if bytes.len() < 5 {
                    Err(BitcoinError::InsufficientBytes)
                } else {
                    let val = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as u64;
                    Ok((CompactSize::new(val), 5))
                }
            }
            0xFF => {
                if bytes.len() < 9 {
                    Err(BitcoinError::InsufficientBytes)
                } else {
                    let val = u64::from_le_bytes([
                        bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                        bytes[8],
                    ]);
                    Ok((CompactSize::new(val), 9))
                }
            }
            _ => Err(BitcoinError::InvalidFormat),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Txid(pub [u8; 32]);

impl Serialize for Txid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // TODO: Serialize as a hex-encoded string (32 bytes => 64 hex characters)
        serializer.serialize_str(&hex::encode(&self.0))
    }
}

impl<'de> Deserialize<'de> for Txid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: Parse hex string into 32-byte array
        // Use `hex::decode`, validate length = 32
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Txid must be 32 bytes"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Txid(arr))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct OutPoint {
    pub txid: Txid,
    pub vout: u32,
}

impl OutPoint {
    pub fn new(txid: [u8; 32], vout: u32) -> Self {
        // TODO: Create an OutPoint from raw txid bytes and output index
        OutPoint {
            txid: Txid(txid),
            vout,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Serialize as: txid (32 bytes) + vout (4 bytes, little-endian)
        let mut res = self.txid.0.to_vec();
        res.extend(&self.vout.to_le_bytes());
        res
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Deserialize 36 bytes: txid[0..32], vout[32..36]
        // Return error if insufficient bytes
        if bytes.len() < 36 {
            return Err(BitcoinError::InsufficientBytes);
        }
        let mut txid = [0u8; 32];
        txid.copy_from_slice(&bytes[..32]);
        let vout = u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);
        Ok((OutPoint::new(txid, vout), 36))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Script {
    pub bytes: Vec<u8>,
}

impl Script {
    pub fn new(bytes: Vec<u8>) -> Self {
        // TODO: Simple constructor
        Script { bytes }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Prefix with CompactSize (length), then raw bytes
        let mut compact_len_prefix = CompactSize::new(self.bytes.len() as u64).to_bytes();
        compact_len_prefix.extend(&self.bytes);
        compact_len_prefix
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Parse CompactSize prefix, then read that many bytes
        // Return error if not enough bytes
        let (len, offset) = CompactSize::from_bytes(bytes)?;
        let total_compact_len = offset + len.value as usize;
        if bytes.len() < total_compact_len {
            return Err(BitcoinError::InsufficientBytes);
        }
        Ok((
            Script::new(bytes[offset..total_compact_len].to_vec()),
            total_compact_len,
        ))
    }
}

impl Deref for Script {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        // TODO: Allow &Script to be used as &[u8]
        &self.bytes
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    pub previous_output: OutPoint,
    pub script_sig: Script,
    pub sequence: u32,
}

impl TransactionInput {
    pub fn new(previous_output: OutPoint, script_sig: Script, sequence: u32) -> Self {
        // TODO: Basic constructor
        TransactionInput {
            previous_output,
            script_sig,
            sequence,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Serialize: OutPoint + Script (with CompactSize) + sequence (4 bytes LE)
        let mut result = self.previous_output.to_bytes();
        result.extend(self.script_sig.to_bytes());
        result.extend(&self.sequence.to_le_bytes());
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Deserialize in order:
        // - OutPoint (36 bytes)
        // - Script (with CompactSize)
        // - Sequence (4 bytes)
        let (outpoint, outpoint_consumed) = OutPoint::from_bytes(bytes)?;

        let script_slice = &bytes[outpoint_consumed..];
        let (script, script_consumed) = Script::from_bytes(script_slice)?;

        let total = outpoint_consumed + script_consumed;
        if bytes.len() < total + 4 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let sequence_bytes = &bytes[total..total + 4];
        let sequence = u32::from_le_bytes([
            sequence_bytes[0],
            sequence_bytes[1],
            sequence_bytes[2],
            sequence_bytes[3],
        ]);

        Ok((TransactionInput::new(outpoint, script, sequence), total + 4))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BitcoinTransaction {
    pub version: u32,
    pub inputs: Vec<TransactionInput>,
    pub lock_time: u32,
}

impl BitcoinTransaction {
    pub fn new(version: u32, inputs: Vec<TransactionInput>, lock_time: u32) -> Self {
        // TODO: Construct a transaction from parts
        BitcoinTransaction {
            version,
            inputs,
            lock_time,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Format:
        // - version (4 bytes LE)
        // - CompactSize (number of inputs)
        // - each input serialized
        // - lock_time (4 bytes LE)
        let mut result = self.version.to_le_bytes().to_vec();
        result.extend(CompactSize::new(self.inputs.len() as u64).to_bytes());
        for input in &self.inputs {
            result.extend(input.to_bytes());
        }
        result.extend(&self.lock_time.to_le_bytes());
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Read version, CompactSize for input count
        // Parse inputs one by one
        // Read final 4 bytes for lock_time
        if bytes.len() < 4 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let version = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let (input_count, mut offset) = CompactSize::from_bytes(&bytes[4..])?;
        offset += 4;

        let mut inputs = Vec::new();
        for _ in 0..input_count.value {
            let (input, consumed) = TransactionInput::from_bytes(&bytes[offset..])?;
            offset += consumed;
            inputs.push(input);
        }

        if bytes.len() < offset + 4 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let lock_time = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);

        Ok((
            BitcoinTransaction::new(version, inputs, lock_time),
            offset + 4,
        ))
    }
}

impl fmt::Display for BitcoinTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Format a user-friendly string showing version, inputs, lock_time
        // Display scriptSig length and bytes, and previous output info
        writeln!(f, "Version: {}", self.version)?;
        for input in &self.inputs {
            writeln!(
                f,
                "Previous Output Vout: {}",
                input.previous_output.vout
            )?;
        }
        writeln!(f, "Lock Time: {}", self.lock_time)
    }
}
