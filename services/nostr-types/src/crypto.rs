use crate::error::NostrError;
use secp256k1::{Secp256k1, Message, Signature as Secp256k1Signature};
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature(String);

impl PublicKey {
    pub fn new(hex: String) -> Result<Self, NostrError> {
        if hex.len() != 64 {
            return Err(NostrError::InvalidPublicKey(
                "Public key must be 64 hex characters".to_string()
            ));
        }
        
        // Validate hex encoding
        hex::decode(&hex).map_err(|_| {
            NostrError::InvalidPublicKey("Invalid hex encoding".to_string())
        })?;
        
        Ok(PublicKey(hex))
    }
    
    pub fn as_hex(&self) -> &str {
        &self.0
    }
    
    pub fn as_bytes(&self) -> Result<Vec<u8>, NostrError> {
        hex::decode(&self.0).map_err(|_| {
            NostrError::InvalidPublicKey("Invalid hex encoding".to_string())
        })
    }
}

impl FromStr for PublicKey {
    type Err = NostrError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl Signature {
    pub fn new(hex: String) -> Result<Self, NostrError> {
        if hex.len() != 128 {
            return Err(NostrError::InvalidSignature);
        }
        
        // Validate hex encoding
        hex::decode(&hex).map_err(|_| NostrError::InvalidSignature)?;
        
        Ok(Signature(hex))
    }
    
    pub fn as_hex(&self) -> &str {
        &self.0
    }
    
    pub fn as_bytes(&self) -> Result<Vec<u8>, NostrError> {
        hex::decode(&self.0).map_err(|_| NostrError::InvalidSignature)
    }
}

impl FromStr for Signature {
    type Err = NostrError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

/// Verify a Schnorr signature for a message
pub fn verify_signature(
    message_hash: &[u8],
    public_key: &PublicKey,
    signature: &Signature,
) -> Result<bool, NostrError> {
    let secp = Secp256k1::verification_only();
    
    // Parse public key
    let pubkey_bytes = public_key.as_bytes()?;
    let pubkey = secp256k1::XOnlyPublicKey::from_slice(&pubkey_bytes)
        .map_err(|e| NostrError::CryptoError(format!("Invalid public key: {}", e)))?;
    
    // Parse signature
    let sig_bytes = signature.as_bytes()?;
    let sig = Secp256k1Signature::from_slice(&sig_bytes)
        .map_err(|e| NostrError::CryptoError(format!("Invalid signature: {}", e)))?;
    
    // Create message
    let message = Message::from_slice(message_hash)
        .map_err(|e| NostrError::CryptoError(format!("Invalid message hash: {}", e)))?;
    
    // Verify signature
    Ok(secp.verify_schnorr(&sig, &message, &pubkey).is_ok())
}

/// Create SHA256 hash of the given data
pub fn sha256_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_public_key_validation() {
        // Valid public key
        let valid_pk = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert!(PublicKey::new(valid_pk.to_string()).is_ok());
        
        // Invalid length
        let invalid_pk = "1234567890abcdef";
        assert!(PublicKey::new(invalid_pk.to_string()).is_err());
        
        // Invalid hex
        let invalid_hex = "gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg";
        assert!(PublicKey::new(invalid_hex.to_string()).is_err());
    }
    
    #[test]
    fn test_signature_validation() {
        // Valid signature (128 hex chars)
        let valid_sig = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert!(Signature::new(valid_sig.to_string()).is_ok());
        
        // Invalid length
        let invalid_sig = "1234567890abcdef";
        assert!(Signature::new(invalid_sig.to_string()).is_err());
    }
}
