use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce, Key,
};
use rand_core::{OsRng, RngCore};
use base64::{engine::general_purpose, Engine as _};
use rsa::{
    pkcs1v15::Pkcs1v15Encrypt,
    RsaPublicKey,
    RsaPrivateKey,
};
use pkcs8::{DecodePublicKey, DecodePrivateKey};
use std::{fs, path::PathBuf};

#[derive(Debug)]
enum AppError {
    Crypto(aes_gcm::Error),
    Io(std::io::Error),
    Rsa(rsa::Error),
    Pkcs8(pkcs8::Error),
    Base64(base64::DecodeError),
    Other(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Crypto(err) => write!(f, "Cryptographic error: {}", err),
            AppError::Io(err) => write!(f, "IO error: {}", err),
            AppError::Rsa(err) => write!(f, "RSA error: {}", err),
            AppError::Pkcs8(err) => write!(f, "PKCS8 error: {}", err),
            AppError::Base64(err) => write!(f, "Base64 decode error: {}", err),
            AppError::Other(err) => write!(f, "Other error: {}", err),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Crypto(_) => None,
            AppError::Io(err) => Some(err),
            AppError::Rsa(err) => Some(err),
            AppError::Pkcs8(err) => Some(err),
            AppError::Base64(err) => Some(err),
            AppError::Other(_) => None,
        }
    }
}

impl From<aes_gcm::Error> for AppError {
    fn from(err: aes_gcm::Error) -> Self {
        AppError::Crypto(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<rsa::Error> for AppError {
    fn from(err: rsa::Error) -> Self {
        AppError::Rsa(err)
    }
}

impl From<pkcs8::Error> for AppError {
    fn from(err: pkcs8::Error) -> Self {
        AppError::Pkcs8(err)
    }
}

impl From<base64::DecodeError> for AppError {
    fn from(err: base64::DecodeError) -> Self {
        AppError::Base64(err)
    }
}

/// Asymmetric encryption function
/// to_encrypt: Data to be encrypted (usually a symmetric key)
/// public_key_path: Path to the public key file
fn encrypt_asymmetric(to_encrypt: &[u8], public_key_path: &PathBuf) -> Result<String, AppError> {
    let public_key_pem = fs::read_to_string(public_key_path)?;
    let public_key = RsaPublicKey::from_public_key_pem(&public_key_pem)
        .map_err(|e| AppError::Pkcs8(e.into()))?;

    let padding = Pkcs1v15Encrypt;
    let encrypted = public_key.encrypt(&mut OsRng, padding, to_encrypt)?;
    Ok(general_purpose::STANDARD.encode(&encrypted))
}

/// Symmetric encryption function (part of hybrid encryption)
/// text: Plaintext to be encrypted
/// public_key_path: Path to the public key file used to encrypt the symmetric key
fn encrypt_symmetric(text: &str, public_key_path: &PathBuf) -> Result<String, AppError> {
    let mut key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut key_bytes);

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, text.as_bytes())?;

    let encrypted_symmetric_key = encrypt_asymmetric(&key_bytes, public_key_path)?;

    Ok(format!(
        "{}:{}:{}",
        encrypted_symmetric_key,
        general_purpose::STANDARD.encode(&nonce_bytes),
        general_purpose::STANDARD.encode(&ciphertext)
    ))
}

/// Asymmetric decryption function
/// to_decrypt_base64: Base64 encoded symmetric key ciphertext
/// private_key_path: Path to the private key file
fn decrypt_asymmetric(to_decrypt_base64: &str, private_key_path: &PathBuf) -> Result<Vec<u8>, AppError> {
    let private_key_pem = fs::read_to_string(private_key_path)?;
    let private_key = RsaPrivateKey::from_pkcs8_pem(&private_key_pem)
        .map_err(|e| AppError::Pkcs8(e.into()))?;

    let encrypted_bytes = general_purpose::STANDARD.decode(to_decrypt_base64)?;

    let padding = Pkcs1v15Encrypt;
    let decrypted = private_key.decrypt(padding, &encrypted_bytes)?;
    Ok(decrypted)
}

/// Symmetric decryption function (part of hybrid decryption)
/// encrypted_string: Full encrypted string (format: encrypted_key:Nonce:ciphertext)
/// private_key_path: Path to the private key file used to decrypt the symmetric key
fn decrypt_symmetric(encrypted_string: &str, private_key_path: &PathBuf) -> Result<String, AppError> {
    let parts: Vec<&str> = encrypted_string.split(':').collect();
    if parts.len() != 3 {
        return Err(AppError::Other("Invalid encrypted string format".to_string()));
    }

    let encrypted_symmetric_key_base64 = parts[0];
    let nonce_base64 = parts[1];
    let ciphertext_base64 = parts[2];

    // 1. Decrypt the symmetric key using asymmetric decryption
    let decrypted_symmetric_key_bytes = decrypt_asymmetric(encrypted_symmetric_key_base64, private_key_path)?;
    let key = Key::<Aes256Gcm>::from_slice(&decrypted_symmetric_key_bytes);
    let cipher = Aes256Gcm::new(key);

    // 2. Decode Nonce and ciphertext
    let nonce_bytes = general_purpose::STANDARD.decode(nonce_base64)?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = general_purpose::STANDARD.decode(ciphertext_base64)?;

    // 3. Decrypt the ciphertext
    let decrypted_text_bytes = cipher.decrypt(nonce, ciphertext.as_ref())?;
    Ok(String::from_utf8(decrypted_text_bytes)
        .map_err(|e| AppError::Other(format!("UTF-8 decode error: {}", e)))?)
}


// Extracts the main logic for easier testing
fn run_encryption_example(public_key_path: &PathBuf) -> Result<String, AppError> {
    let plaintext = "This is a secret message that I hope will be securely encrypted.";
    println!("Original plaintext: {}", plaintext);

    let encrypted_string = encrypt_symmetric(plaintext, public_key_path)?;
    println!("Encrypted string: {}", encrypted_string);
    Ok(encrypted_string)
}

fn main() -> Result<(), AppError> {
    let public_key_path = PathBuf::from("public_key.pem");
    let private_key_path = PathBuf::from("private_key.pem");

    if !public_key_path.exists() || !private_key_path.exists() {
        eprintln!("Error: Public or private key files not found.");
        eprintln!("Please generate an RSA key pair and place them in the project root.");
        eprintln!("For example, using OpenSSL:");
        eprintln!("  openssl genrsa -out private_key.pem 2048");
        eprintln!("  openssl rsa -in private_key.pem -pubout -out public_key.pem");
        return Ok(());
    }

    // Run encryption example and get the encrypted string
    let encrypted_text = run_encryption_example(&public_key_path)?;

    // --- Add decryption example to use decrypt_symmetric and decrypt_asymmetric functions ---
    println!("\nStarting decryption...");
    match decrypt_symmetric(&encrypted_text, &private_key_path) {
        Ok(decrypted_string) => {
            println!("Decrypted plaintext: {}", decrypted_string);
        }
        Err(e) => {
            eprintln!("Decryption failed: {}", e);
            return Err(e);
        }
    }
    // --- End of decryption example ---

    Ok(())
}


// --- Test Module ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption_flow() -> Result<(), AppError> {
        let public_key_path = PathBuf::from("public_key.pem");
        let private_key_path = PathBuf::from("private_key.pem");

        if !public_key_path.exists() || !private_key_path.exists() {
            eprintln!("\nWarning: Key files not found. Please generate them with OpenSSL commands:");
            eprintln!("  openssl genrsa -out private_key.pem 2048");
            eprintln!("  openssl rsa -in private_key.pem -pubout -out public_key.pem");
            eprintln!("Skipping test.\n");
            return Ok(());
        }

        let original_plaintext = "This is a secret message used for testing encryption and decryption!";

        println!("\n[Test] Starting encryption...");
        let encrypted_string = encrypt_symmetric(original_plaintext, &public_key_path)?;
        println!("[Test] Encryption complete, encrypted string: {}", encrypted_string);

        println!("[Test] Starting decryption...");
        let decrypted_plaintext = decrypt_symmetric(&encrypted_string, &private_key_path)?;
        println!("[Test] Decryption complete, decrypted plaintext: {}", decrypted_plaintext);

        assert_eq!(original_plaintext, decrypted_plaintext, "Decrypted text does not match original text!");
        println!("[Test] Assertion successful: Original and decrypted text match.");

        Ok(())
    }
}
