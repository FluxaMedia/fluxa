use aes::cipher::block_padding::{NoPadding, Pkcs7};
use aes::cipher::{BlockModeDecrypt, BlockModeEncrypt, KeyInit, KeyIvInit};
use aes::{Aes128, Aes192, Aes256};
use hmac::{Hmac, Mac};
use md5::Md5;
use pbkdf2::pbkdf2_hmac;
use rand::Rng;
use rsa::pkcs1v15::{SigningKey as RsaSigningKey, VerifyingKey as RsaVerifyingKey};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::signature::{SignatureEncoding, Signer, Verifier};
use rsa::{RsaPrivateKey, RsaPublicKey};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};

pub fn digest(algorithm: &str, data: &[u8]) -> Result<Vec<u8>, String> {
    Ok(match algorithm {
        "MD5" => Md5::digest(data).to_vec(),
        "SHA1" => Sha1::digest(data).to_vec(),
        "SHA256" => Sha256::digest(data).to_vec(),
        "SHA384" => Sha384::digest(data).to_vec(),
        "SHA512" => Sha512::digest(data).to_vec(),
        other => return Err(format!("unsupported digest algorithm: {other}")),
    })
}

pub fn hmac(algorithm: &str, key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    macro_rules! hmac_with {
        ($hash:ty) => {{
            let mut mac = Hmac::<$hash>::new_from_slice(key).map_err(|e| e.to_string())?;
            mac.update(data);
            mac.finalize().into_bytes().to_vec()
        }};
    }
    Ok(match algorithm {
        "MD5" => hmac_with!(Md5),
        "SHA1" => hmac_with!(Sha1),
        "SHA256" => hmac_with!(Sha256),
        "SHA384" => hmac_with!(Sha384),
        "SHA512" => hmac_with!(Sha512),
        other => return Err(format!("unsupported HMAC algorithm: {other}")),
    })
}

pub fn pbkdf2(
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    key_size_bits: u32,
    algorithm: &str,
) -> Result<Vec<u8>, String> {
    let out_len = (key_size_bits as usize).div_ceil(8);
    let mut out = vec![0u8; out_len];
    macro_rules! pbkdf2_with {
        ($hash:ty) => {
            pbkdf2_hmac::<$hash>(password, salt, iterations, &mut out)
        };
    }
    match algorithm {
        "MD5" => pbkdf2_with!(Md5),
        "SHA1" => pbkdf2_with!(Sha1),
        "SHA256" => pbkdf2_with!(Sha256),
        "SHA384" => pbkdf2_with!(Sha384),
        "SHA512" => pbkdf2_with!(Sha512),
        other => return Err(format!("unsupported PBKDF2 algorithm: {other}")),
    }
    Ok(out)
}

pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    rand::rng().fill_bytes(&mut bytes);
    bytes
}

pub fn aes_encrypt(mode: &str, key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    let (base_mode, no_padding) = split_padding(mode);
    match base_mode {
        "AES-CBC" => cbc_encrypt(key, iv, data, no_padding),
        "AES-ECB" => ecb_encrypt(key, data, no_padding),
        "AES-GCM" => gcm_encrypt(key, iv, data),
        other => Err(format!("unsupported AES mode: {other}")),
    }
}

pub fn aes_decrypt(mode: &str, key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    let (base_mode, no_padding) = split_padding(mode);
    match base_mode {
        "AES-CBC" => cbc_decrypt(key, iv, data, no_padding),
        "AES-ECB" => ecb_decrypt(key, data, no_padding),
        "AES-GCM" => gcm_decrypt(key, iv, data),
        other => Err(format!("unsupported AES mode: {other}")),
    }
}

fn split_padding(mode: &str) -> (&str, bool) {
    match mode.strip_suffix("-NoPadding") {
        Some(base) => (base, true),
        None => (mode, false),
    }
}

fn cbc_encrypt(key: &[u8], iv: &[u8], data: &[u8], no_padding: bool) -> Result<Vec<u8>, String> {
    if iv.len() != 16 {
        return Err("AES-CBC requires a 16-byte IV".to_string());
    }
    match key.len() {
        16 => cbc_encrypt_with::<Aes128>(key, iv, data, no_padding),
        24 => cbc_encrypt_with::<Aes192>(key, iv, data, no_padding),
        32 => cbc_encrypt_with::<Aes256>(key, iv, data, no_padding),
        other => Err(format!("unsupported AES key length: {other} bytes")),
    }
}

fn cbc_decrypt(key: &[u8], iv: &[u8], data: &[u8], no_padding: bool) -> Result<Vec<u8>, String> {
    if iv.len() != 16 {
        return Err("AES-CBC requires a 16-byte IV".to_string());
    }
    match key.len() {
        16 => cbc_decrypt_with::<Aes128>(key, iv, data, no_padding),
        24 => cbc_decrypt_with::<Aes192>(key, iv, data, no_padding),
        32 => cbc_decrypt_with::<Aes256>(key, iv, data, no_padding),
        other => Err(format!("unsupported AES key length: {other} bytes")),
    }
}

fn cbc_encrypt_with<C>(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
    no_padding: bool,
) -> Result<Vec<u8>, String>
where
    cbc::Encryptor<C>: KeyIvInit + BlockModeEncrypt,
    C: aes::cipher::BlockCipherEncrypt,
{
    let encryptor = cbc::Encryptor::<C>::new_from_slices(key, iv).map_err(|e| e.to_string())?;
    let mut buf = data.to_vec();
    buf.resize(data.len() + 16, 0);
    let result = if no_padding {
        encryptor
            .encrypt_padded::<NoPadding>(&mut buf, data.len())
            .map_err(|e| e.to_string())?
    } else {
        encryptor
            .encrypt_padded::<Pkcs7>(&mut buf, data.len())
            .map_err(|e| e.to_string())?
    };
    Ok(result.to_vec())
}

fn cbc_decrypt_with<C>(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
    no_padding: bool,
) -> Result<Vec<u8>, String>
where
    cbc::Decryptor<C>: KeyIvInit + BlockModeDecrypt,
    C: aes::cipher::BlockCipherDecrypt,
{
    let decryptor = cbc::Decryptor::<C>::new_from_slices(key, iv).map_err(|e| e.to_string())?;
    let mut buf = data.to_vec();
    let result = if no_padding {
        decryptor
            .decrypt_padded::<NoPadding>(&mut buf)
            .map_err(|e| e.to_string())?
    } else {
        decryptor
            .decrypt_padded::<Pkcs7>(&mut buf)
            .map_err(|e| e.to_string())?
    };
    Ok(result.to_vec())
}

fn ecb_encrypt(key: &[u8], data: &[u8], no_padding: bool) -> Result<Vec<u8>, String> {
    match key.len() {
        16 => ecb_encrypt_with::<Aes128>(key, data, no_padding),
        24 => ecb_encrypt_with::<Aes192>(key, data, no_padding),
        32 => ecb_encrypt_with::<Aes256>(key, data, no_padding),
        other => Err(format!("unsupported AES key length: {other} bytes")),
    }
}

fn ecb_decrypt(key: &[u8], data: &[u8], no_padding: bool) -> Result<Vec<u8>, String> {
    match key.len() {
        16 => ecb_decrypt_with::<Aes128>(key, data, no_padding),
        24 => ecb_decrypt_with::<Aes192>(key, data, no_padding),
        32 => ecb_decrypt_with::<Aes256>(key, data, no_padding),
        other => Err(format!("unsupported AES key length: {other} bytes")),
    }
}

fn ecb_encrypt_with<C>(key: &[u8], data: &[u8], no_padding: bool) -> Result<Vec<u8>, String>
where
    ecb::Encryptor<C>: aes::cipher::KeyInit + BlockModeEncrypt,
    C: aes::cipher::BlockCipherEncrypt,
{
    let encryptor = ecb::Encryptor::<C>::new_from_slice(key).map_err(|e| e.to_string())?;
    let mut buf = data.to_vec();
    buf.resize(data.len() + 16, 0);
    let result = if no_padding {
        encryptor
            .encrypt_padded::<NoPadding>(&mut buf, data.len())
            .map_err(|e| e.to_string())?
    } else {
        encryptor
            .encrypt_padded::<Pkcs7>(&mut buf, data.len())
            .map_err(|e| e.to_string())?
    };
    Ok(result.to_vec())
}

fn ecb_decrypt_with<C>(key: &[u8], data: &[u8], no_padding: bool) -> Result<Vec<u8>, String>
where
    ecb::Decryptor<C>: aes::cipher::KeyInit + BlockModeDecrypt,
    C: aes::cipher::BlockCipherDecrypt,
{
    let decryptor = ecb::Decryptor::<C>::new_from_slice(key).map_err(|e| e.to_string())?;
    let mut buf = data.to_vec();
    let result = if no_padding {
        decryptor
            .decrypt_padded::<NoPadding>(&mut buf)
            .map_err(|e| e.to_string())?
    } else {
        decryptor
            .decrypt_padded::<Pkcs7>(&mut buf)
            .map_err(|e| e.to_string())?
    };
    Ok(result.to_vec())
}

fn gcm_encrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes128Gcm, Aes256Gcm, Nonce};

    if iv.len() != 12 {
        return Err("AES-GCM requires a 12-byte IV".to_string());
    }
    let nonce = Nonce::from_slice(iv);
    match key.len() {
        16 => Aes128Gcm::new_from_slice(key)
            .map_err(|e| e.to_string())?
            .encrypt(nonce, data)
            .map_err(|e| e.to_string()),
        32 => Aes256Gcm::new_from_slice(key)
            .map_err(|e| e.to_string())?
            .encrypt(nonce, data)
            .map_err(|e| e.to_string()),
        other => Err(format!("unsupported AES-GCM key length: {other} bytes")),
    }
}

fn gcm_decrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes128Gcm, Aes256Gcm, Nonce};

    if iv.len() != 12 {
        return Err("AES-GCM requires a 12-byte IV".to_string());
    }
    let nonce = Nonce::from_slice(iv);
    match key.len() {
        16 => Aes128Gcm::new_from_slice(key)
            .map_err(|e| e.to_string())?
            .decrypt(nonce, data)
            .map_err(|e| e.to_string()),
        32 => Aes256Gcm::new_from_slice(key)
            .map_err(|e| e.to_string())?
            .decrypt(nonce, data)
            .map_err(|e| e.to_string()),
        other => Err(format!("unsupported AES-GCM key length: {other} bytes")),
    }
}

pub fn sign(algorithm: &str, private_key_der: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    if let Some(hash) = algorithm.strip_prefix("RSASSA-PKCS1-V1_5-") {
        return rsa_sign(hash, private_key_der, data);
    }
    if algorithm.starts_with("ECDSA") {
        return ecdsa_sign(private_key_der, data);
    }
    Err(format!("unsupported signature algorithm: {algorithm}"))
}

pub fn verify(
    algorithm: &str,
    public_key_der: &[u8],
    signature: &[u8],
    data: &[u8],
) -> Result<bool, String> {
    if let Some(hash) = algorithm.strip_prefix("RSASSA-PKCS1-V1_5-") {
        return rsa_verify(hash, public_key_der, signature, data);
    }
    if algorithm.starts_with("ECDSA") {
        return ecdsa_verify(public_key_der, signature, data);
    }
    Err(format!("unsupported signature algorithm: {algorithm}"))
}

fn rsa_sign(hash: &str, private_key_der: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    use rsa::sha2::{Sha256, Sha384, Sha512};

    let key = RsaPrivateKey::from_pkcs8_der(private_key_der).map_err(|e| e.to_string())?;
    macro_rules! sign_with {
        ($d:ty) => {{
            let signing_key = RsaSigningKey::<$d>::new(key);
            signing_key.sign(data).to_vec()
        }};
    }
    Ok(match hash {
        "SHA256" => sign_with!(Sha256),
        "SHA384" => sign_with!(Sha384),
        "SHA512" => sign_with!(Sha512),
        other => return Err(format!("unsupported RSA signature hash: {other}")),
    })
}

fn rsa_verify(
    hash: &str,
    public_key_der: &[u8],
    signature: &[u8],
    data: &[u8],
) -> Result<bool, String> {
    use rsa::sha2::{Sha256, Sha384, Sha512};

    let key = RsaPublicKey::from_public_key_der(public_key_der).map_err(|e| e.to_string())?;
    macro_rules! verify_with {
        ($d:ty) => {{
            let verifying_key = RsaVerifyingKey::<$d>::new(key);
            let sig = rsa::pkcs1v15::Signature::try_from(signature).map_err(|e| e.to_string())?;
            verifying_key.verify(data, &sig).is_ok()
        }};
    }
    Ok(match hash {
        "SHA256" => verify_with!(Sha256),
        "SHA384" => verify_with!(Sha384),
        "SHA512" => verify_with!(Sha512),
        other => return Err(format!("unsupported RSA signature hash: {other}")),
    })
}

fn ecdsa_sign(private_key_der: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    use p256::ecdsa::signature::Signer as EcdsaSigner;
    use p256::ecdsa::{Signature, SigningKey};
    use p256::pkcs8::DecodePrivateKey as EcdsaDecodePrivateKey;

    let signing_key = SigningKey::from_pkcs8_der(private_key_der).map_err(|e| e.to_string())?;
    let signature: Signature = signing_key.sign(data);
    Ok(signature.to_bytes().to_vec())
}

fn ecdsa_verify(public_key_der: &[u8], signature: &[u8], data: &[u8]) -> Result<bool, String> {
    use p256::ecdsa::signature::Verifier as EcdsaVerifier;
    use p256::ecdsa::{Signature, VerifyingKey};
    use p256::pkcs8::DecodePublicKey as EcdsaDecodePublicKey;

    let verifying_key =
        VerifyingKey::from_public_key_der(public_key_der).map_err(|e| e.to_string())?;
    let sig = Signature::from_slice(signature).map_err(|e| e.to_string())?;
    Ok(verifying_key.verify(data, &sig).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmac_sha256_output_is_32_bytes() {
        let mac = hmac(
            "SHA256",
            b"key",
            b"The quick brown fox jumps over the lazy dog",
        )
        .unwrap();
        assert_eq!(
            mac.len(),
            32,
            "HMAC-SHA256 must be 32 bytes, got {}",
            mac.len()
        );
        let hex: String = mac.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex,
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }

    fn hex_decode(s: &str) -> Vec<u8> {
        (0..s.len() / 2)
            .map(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap())
            .collect()
    }

    const RSA_PRIV_DER_HEX: &str = "308204bd020100300d06092a864886f70d0101010500048204a7308204a30201000282010100bddba8f4e0cb99a11283571767832821a23220b11ec92160e3a99185a7c11342abc002d1b981f2fa73beb88b682e95a06d51cf0ecc82bd6a33ab0d778d1063d3229711313c5350d984b3bce669ddbebabfe34e261588cb6c560edb6d7f83a403e492ef387d8119cbc2f2f695dca535fed5df1fc54001ddb8712b895d90fbd9bd43d3e4b2cf2512f6c6e88d99e711883039d108bfb80e1882b52d0c869740f0c2106c706bd2fb003a50a18414c135042b97ba70874bc5d4100437be9a375ac85c48c768cdfbdc958c0f7a7c3b18cd2de9ef2d5241e1406e0f672f17199fd4c94d6179991f1b1860fa1d13a9a4adcd1ef1a261a4241e5f0e78feaf0ebb54408b9702030100010282010018f74bae77e90e18ba407f315ab14a93b1aa79d610c04e1d5992b0ca2c971d1b454e88905f9e90ff123d603324ce0b6aff3524cf436165db135448ff1193b82cc97da80b1b6db152093b4179e32dffa8bb16feedcf231feced1c230ab11bf473769943cfdfe11c2d49437ddd93183e937638530b33318ba077714c2cfba5c2aa00b2b92c6c6c0d845d689aa3d0745d38b799a6ee65c9c2b601466d350faa03dc089ecd282eca2300bcaebd8ca90fb56136d526dc4e88381e137d07db846b099a3960f418440a2ba90bdbb57bdba68cec424d25524f6d4b3e6fb2b0afe60bc04b6b94e5eb5c94b4b0f17a2a068cabf852af771968958e69bb57d6b12b92d11f4102818100e898d69b8b669ce41491374ae51a4d7ed8d411424b58b0dffe7e10603070782c7600fa69cd86ddaf4bffa4aa84e53f8776c298f3ae246c2072e089688b550757f6129842be9a1fde12d880c8185bf8321ed0175bd3a148e4f7bf29b6e58f2d28003a5e8d9084c9258128591ac99f5a4e5aa8bf3911606b036f344ab8790cce3102818100d0f5f6e2fed04943dc6f37ff2fd34ba6f525a5ae1eb2a838c8f81ae1666d082d9ed9cafec15c146fd141fa2ae2c4da6ad9c88f77e399b905bd54bb62db0f23e5e02fedcc0d5c9bf699cf829efdc1b1af0b512e51dc7105ce15565e8b948e7f1e6eb945d94b5c99bdf7f8fe697deaf60c43cebb38abf2996d13d85de066061c4702818100bf07afd266062bbac30d952ce4c6da78e09a2366a03d425f6543a22d6dfb2127daeee8bb76ba5a5dbb7c45806acf6182df4de992d12646cbcbad900d15ca0ad2fa5dc141a80b2b3b179d4aff8bca15290cd77927c8c340fc1461a35ffc5fc0058752690fb96ee548f12ff326a60b437ef0c480f0dccda47658a1c8f08332ba3102818038589ae24ae3cf5dc52b18666f7702875ac37411a7a575471aaa0c558b510b4b81f22892d98102e1c4038e79d1fc53094d008fef3c284f8404b1bc75c3ac2979eb2201756c84f46962c059f0956c8d852e000e0574f3e524d50c1d22764496729658cf44f0d55f7fa2463a1428281b80e327f3c42c5996959b2c54e4159ec0310281800bbc5ab68bdd62ebe027a1e86e46b9cfdd888c0e6e733e9c8e41a5dd4bb743583671cab340e0372b3d21a59d6dd903ec8a2a21473317e4bc99c54ac1647f1169f624da78dfc23eff30971c0f0ca3e108ea7a93027e9721ed9d8631c76a73aed28592e593b47bfd39c6f3cf5278dcbf7e127ed2024427f4836dbd0a64839dd223";
    const RSA_PUB_DER_HEX: &str = "30820122300d06092a864886f70d01010105000382010f003082010a0282010100bddba8f4e0cb99a11283571767832821a23220b11ec92160e3a99185a7c11342abc002d1b981f2fa73beb88b682e95a06d51cf0ecc82bd6a33ab0d778d1063d3229711313c5350d984b3bce669ddbebabfe34e261588cb6c560edb6d7f83a403e492ef387d8119cbc2f2f695dca535fed5df1fc54001ddb8712b895d90fbd9bd43d3e4b2cf2512f6c6e88d99e711883039d108bfb80e1882b52d0c869740f0c2106c706bd2fb003a50a18414c135042b97ba70874bc5d4100437be9a375ac85c48c768cdfbdc958c0f7a7c3b18cd2de9ef2d5241e1406e0f672f17199fd4c94d6179991f1b1860fa1d13a9a4adcd1ef1a261a4241e5f0e78feaf0ebb54408b970203010001";
    const RSA_SIG_HEX: &str = "1f6e08bb7841498b47f553cb6a3c4a09a177169bdeb6152634720f63ec7348e04c1e07b2915c1999bcb4bdbfbc85f970b641b4a6295e2befda427b51ab2cbd6c2fceaf57e0854c0e1c943802539c050026a89e1fcefe35fd4a6d8ab5cf8955d4f9da1e8713686d038ebda8a9623b1c18cfc2f5d30a3846c6c133fe50930ca03c3b3645999d0eb89056406c5746dd6635dd45ebbcdecb6e48e6853d1594bd41bda6017c6cc2b053df7a791d8c8822528b42a6e554e21ad86bb800c9b298dff248ad807233bc0c95e6db63d98d92926e328f78a5f9a2495d319ba6d43c594107e2829d3fe2b09da6fbbd5e398f57fdc5f0975b78e8aceeb9f63a474bad6fc89198";
    const RSA_SIGNED_MESSAGE: &[u8] = b"hello plugin signature test";

    const EC_PRIV_DER_HEX: &str = "308187020100301306072a8648ce3d020106082a8648ce3d030107046d306b020101042042f8c39da6ffb2b063b1fc24f521c8588064773e0f46c0c1c515759ca3a80f3aa144034200049e6a723242258b9d87c8362fd321140b80e16d1671b5cb9b2dba3da7ccc42b82380102e3fd415ed40dfc8c4b4b218218995327daedc7eb35493f4f5b419aeaf8";
    const EC_PUB_DER_HEX: &str = "3059301306072a8648ce3d020106082a8648ce3d030107034200049e6a723242258b9d87c8362fd321140b80e16d1671b5cb9b2dba3da7ccc42b82380102e3fd415ed40dfc8c4b4b218218995327daedc7eb35493f4f5b419aeaf8";
    const EC_SIG_HEX: &str = "f982b6fd52964b591329f5b503627dc1dc5b7f74ff0cf9acc840ab160636a99a526a7f11ee179e77d176827ab0035ee92653e3e7408c6c8fea3f566ec79e8c8f";
    const EC_SIGNED_MESSAGE: &[u8] = b"hello plugin signature test";

    #[test]
    fn rsa_verify_accepts_an_openssl_produced_signature() {
        let key = hex_decode(RSA_PUB_DER_HEX);
        let sig = hex_decode(RSA_SIG_HEX);
        let ok = verify("RSASSA-PKCS1-V1_5-SHA256", &key, &sig, RSA_SIGNED_MESSAGE).unwrap();
        assert!(ok, "should verify a real openssl-produced RSA signature");
    }

    #[test]
    fn rsa_verify_rejects_a_tampered_message() {
        let key = hex_decode(RSA_PUB_DER_HEX);
        let sig = hex_decode(RSA_SIG_HEX);
        let ok = verify("RSASSA-PKCS1-V1_5-SHA256", &key, &sig, b"tampered message").unwrap();
        assert!(!ok);
    }

    #[test]
    fn rsa_sign_then_openssl_style_verify_roundtrips() {
        let priv_key = hex_decode(RSA_PRIV_DER_HEX);
        let pub_key = hex_decode(RSA_PUB_DER_HEX);
        let sig = sign("RSASSA-PKCS1-V1_5-SHA256", &priv_key, RSA_SIGNED_MESSAGE).unwrap();
        let ok = verify(
            "RSASSA-PKCS1-V1_5-SHA256",
            &pub_key,
            &sig,
            RSA_SIGNED_MESSAGE,
        )
        .unwrap();
        assert!(ok);
    }

    #[test]
    fn ecdsa_verify_accepts_a_python_cryptography_produced_signature() {
        let key = hex_decode(EC_PUB_DER_HEX);
        let sig = hex_decode(EC_SIG_HEX);
        let ok = verify("ECDSA-SHA256", &key, &sig, EC_SIGNED_MESSAGE).unwrap();
        assert!(
            ok,
            "should verify a real python `cryptography`-produced P-256 signature"
        );
    }

    #[test]
    fn ecdsa_sign_then_verify_roundtrips() {
        let priv_key = hex_decode(EC_PRIV_DER_HEX);
        let pub_key = hex_decode(EC_PUB_DER_HEX);
        let sig = sign("ECDSA-SHA256", &priv_key, EC_SIGNED_MESSAGE).unwrap();
        let ok = verify("ECDSA-SHA256", &pub_key, &sig, EC_SIGNED_MESSAGE).unwrap();
        assert!(ok);
    }
}
