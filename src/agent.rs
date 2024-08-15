//! Module to build recipients/signers for the various types of COSE messages.
//!
//! This structure is also used to build counter signatures that can be present in any type of COSE
//! message.
//!
//! # Example
//!
//! This example shows a cose-sign1 message with 2 counter signatures present in it, one of them is
//! counter signed externally to the crate.
//!
//! ## Encoding the message
//!
//! ```
//! use cose::message::CoseMessage;
//! use cose::agent::CoseAgent;
//! use cose::keys;
//! use cose::algs;
//! use openssl::bn::BigNum;
//! use openssl::bn::BigNumContext;
//! use openssl::ec::EcPoint;
//! use openssl::ec::{EcGroup, EcKey};
//! use openssl::hash::MessageDigest;
//! use openssl::pkey::PKey;
//! use openssl::sign::{Signer, Verifier};
//! use openssl::nid::Nid;
//! use hex;
//!
//! fn main() {
//!     let msg = b"This is the content.".to_vec();
//!     let kid = b"kid2".to_vec();
//!
//!     // Prepare cose-key to encode the message
//!     let mut key = keys::CoseKey::new();
//!     key.kty(keys::OKP);
//!     key.alg(algs::EDDSA);
//!     key.crv(keys::ED25519);
//!     key.x(hex::decode("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a").unwrap());
//!     key.d(hex::decode("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60").unwrap());
//!     key.key_ops(vec![keys::KEY_OPS_SIGN]);
//!
//!     // Prepare cose_sign1 message
//!     let mut sign1 = CoseMessage::new_sign();
//!     sign1.header.alg(algs::EDDSA, true, false);
//!     sign1.payload(msg);
//!
//!     // Add key and generate the signature without AAD
//!     sign1.key(&key).unwrap();
//!     sign1.secure_content(None).unwrap();
//!
//!
//!     // Prepare counter signature 1 key
//!     let mut counter1_key = keys::CoseKey::new();
//!     counter1_key.kty(keys::EC2);
//!     counter1_key.alg(algs::ES256);
//!     counter1_key.crv(keys::P_256);
//!     counter1_key.x(hex::decode("bac5b11cad8f99f9c72b05cf4b9e26d244dc189f745228255a219a86d6a09eff").unwrap());
//!     counter1_key.y(hex::decode("20138bf82dc1b6d562be0fa54ab7804a3a64b6d72ccfed6b6fb6ed28bbfc117e").unwrap());
//!     counter1_key.d(hex::decode("57c92077664146e876760c9520d054aa93c3afb04e306705db6090308507b4d3").unwrap());
//!     counter1_key.key_ops(vec![keys::KEY_OPS_SIGN]);
//!
//!     // Prepare counter signature 1
//!     let mut counter1 = CoseAgent::new_counter_sig();
//!     counter1.header.kid(vec![0], true, false);
//!     counter1.header.alg(algs::ES256, true, false);
//!
//!     // Add counter signature 1 key, counter sign and add to the cose-sign1 message
//!     counter1.key(&counter1_key).unwrap();
//!     sign1.counter_sig(None, &mut counter1).unwrap();
//!     sign1.add_counter_sig(counter1).unwrap();
//!
//!
//!     // Prepare counter signature 2
//!     let mut counter2 = CoseAgent::new_counter_sig();
//!     counter2.header.alg(algs::ES256, true, false);
//!     counter2.header.kid([3].to_vec(), true, false);
//!
//!     // Get content to counter sign externally
//!     let to_sign = sign1.get_to_sign(None, &mut counter2).unwrap();
//!
//!     // Prepare private key to sign the content
//!     let counter2_priv_key = hex::decode("02d1f7e6f26c43d4868d87ceb2353161740aacf1f7163647984b522a848df1c3").unwrap();
//!
//!     // Sign the content externally
//!     let number = BigNum::from_slice(counter2_priv_key.as_slice()).unwrap();
//!     let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
//!     let ec_key = EcKey::from_private_components(&group, &number, &EcPoint::new(&group).unwrap()).unwrap();
//!     let final_key = PKey::from_ec_key(ec_key).unwrap();
//!     let mut signer = Signer::new(MessageDigest::sha256(), &final_key).unwrap();
//!     signer.update(to_sign.as_slice()).unwrap();
//!
//!     // Add counter signature to the cose-sign1 message
//!     counter2.add_signature(signer.sign_to_vec().unwrap()).unwrap();
//!     sign1.add_counter_sig(counter2).unwrap();
//!
//!     // Encode cose-sign1 message
//!     sign1.encode(true).unwrap();
//! }
//!
//! ```
//!
//! ## Decoding the message
//!
//! ```
//! use cose::message::CoseMessage;
//! use cose::agent::CoseAgent;
//! use cose::keys;
//! use cose::algs;
//! use openssl::bn::BigNum;
//! use openssl::bn::BigNumContext;
//! use openssl::ec::EcPoint;
//! use openssl::ec::{EcGroup, EcKey};
//! use openssl::hash::MessageDigest;
//! use openssl::pkey::PKey;
//! use openssl::sign::{Signer, Verifier};
//! use openssl::nid::Nid;
//! use hex;
//!
//! fn main() {
//!
//!     // Prepare cose-key to encode the message
//!     let mut key = keys::CoseKey::new();
//!     key.kty(keys::OKP);
//!     key.alg(algs::EDDSA);
//!     key.crv(keys::ED25519);
//!     key.x(hex::decode("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a").unwrap());
//!     key.key_ops(vec![keys::KEY_OPS_VERIFY]);
//!
//!     // Prepare CoseMessage with the cose-sign1 message to decode
//!     let mut verify = CoseMessage::new_sign();
//!     verify.bytes = hex::decode("d28443a10127a107828346a20441000126a058402d9410644cfe376119fcb4ff9c48ee97a14fec58e6b02fc10b61d3dc667b2fcb74ab7cce6b2303066e5440e3d5cdd5a164c4483972eba8a9d85e924384365d4d8346a20126044103a058473045022100ce59168dd9e23b229c7f9362deb0efb9c1210ae0ed3e224c968210493ed96d97022024da25640eabd57e0f306abcc9e46973d530c34d7f112eea8ab27f0464312c0c54546869732069732074686520636f6e74656e742e58406354488f9f290e36cd80e23762e664a5cb03e4267c66a8cffaef7c66d89a40bf2cbb8222432a08e5ee410d8b540c6931d26fb6af673f7e2100655d8bae765c04").unwrap();
//!     verify.init_decoder(None).unwrap();
//!
//!     // Add key and decode the message
//!     verify.key(&key).unwrap();
//!     verify.decode(None, None).unwrap();
//!
//!
//!     // Prepare counter signature 1 cose-key
//!     let mut c1_key = keys::CoseKey::new();
//!     c1_key.kty(keys::EC2);
//!     c1_key.alg(algs::ES256);
//!     c1_key.crv(keys::P_256);
//!     c1_key.x(hex::decode("bac5b11cad8f99f9c72b05cf4b9e26d244dc189f745228255a219a86d6a09eff").unwrap());
//!     c1_key.y(hex::decode("20138bf82dc1b6d562be0fa54ab7804a3a64b6d72ccfed6b6fb6ed28bbfc117e").unwrap());
//!     c1_key.key_ops(vec![keys::KEY_OPS_VERIFY]);
//!
//!     // Prepare counter signature 2 public key
//!     let counter2_pub_key =
//!     hex::decode("0398f50a4ff6c05861c8860d13a638ea56c3f5ad7590bbfbf054e1c7b4d91d6280").unwrap();
//!
//!     // Get counter signature 1 index
//!     let mut c1 = verify.header.get_counter(&vec![0]).unwrap()[0];
//!
//!     // Add counter signature 1 key and verify
//!     verify.header.counters[c1].key(&c1_key).unwrap();
//!     verify.counters_verify(None, c1).unwrap();
//!
//!     // Get counter signature 2
//!     let mut c2 = verify.header.get_counter(&vec![3]).unwrap()[0];
//!
//!     // Get content to verify the counter signature externally
//!     let to_verify = verify.get_to_verify(None, &c2).unwrap();
//!
//!     // Verify signture externally
//!     let mut ctx = BigNumContext::new().unwrap();
//!     let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
//!     let point = EcPoint::from_bytes(&group, &counter2_pub_key, &mut ctx).unwrap();
//!     let ec_key = EcKey::from_public_key(&group, &point).unwrap();
//!     let final_key = PKey::from_ec_key(ec_key).unwrap();
//!     let mut verifier = Verifier::new(MessageDigest::sha256(), &final_key).unwrap();
//!     verifier.update(&to_verify).unwrap();
//!     assert!(verifier.verify(&verify.header.counters[c2].payload).unwrap());
//! }
//! ```

use crate::algs;
use crate::cose_struct;
use crate::errors::{CoseError, CoseResult, CoseResultWithRet};
use crate::headers::{CoseHeader, COUNTER_SIG};
use crate::keys;
use cbor::{Decoder, Encoder};
use std::io::Cursor;

/// COSE recipient, signer or counter-signature structure.
#[derive(Clone)]
pub struct CoseAgent {
    /// Header of the CoseAgent (recipient, signer or counter-signature).
    pub header: CoseHeader,
    /// Payload (signature, ciphertext or MAC).
    pub payload: Vec<u8>,
    pub(crate) ph_bstr: Vec<u8>,
    /// Public key.
    pub pub_key: Vec<u8>,
    /// Private/Symmetric key.
    pub s_key: Vec<u8>,
    pub(crate) context: String,
    pub(crate) crv: Option<i32>,
    pub(crate) key_ops: Vec<i32>,
    pub(crate) base_iv: Option<Vec<u8>>,
    pub(crate) enc: bool,
}
const KEY_OPS_SKEY: [i32; 8] = [
    keys::KEY_OPS_DERIVE_BITS,
    keys::KEY_OPS_DERIVE,
    keys::KEY_OPS_DECRYPT,
    keys::KEY_OPS_ENCRYPT,
    keys::KEY_OPS_WRAP,
    keys::KEY_OPS_UNWRAP,
    keys::KEY_OPS_MAC_VERIFY,
    keys::KEY_OPS_MAC,
];

const SIZE: usize = 3;

impl CoseAgent {
    /// Creates an empty CoseAgent structure.
    pub fn new() -> CoseAgent {
        CoseAgent {
            header: CoseHeader::new(),
            payload: Vec::new(),
            ph_bstr: Vec::new(),
            pub_key: Vec::new(),
            key_ops: Vec::new(),
            s_key: Vec::new(),
            crv: None,
            base_iv: None,
            context: "".to_string(),
            enc: false,
        }
    }

    /// Creates an empty CoseAgent structure for counter signatures.
    pub fn new_counter_sig() -> CoseAgent {
        CoseAgent {
            header: CoseHeader::new(),
            payload: Vec::new(),
            ph_bstr: Vec::new(),
            pub_key: Vec::new(),
            key_ops: Vec::new(),
            s_key: Vec::new(),
            crv: None,
            base_iv: None,
            context: cose_struct::COUNTER_SIGNATURE.to_string(),
            enc: false,
        }
    }

    /// Adds an [header](../headers/struct.CoseHeader.html).
    pub fn add_header(&mut self, header: CoseHeader) {
        self.header = header;
    }

    /// Adds a [cose-key](../keys/struct.CoseKey.html).
    pub fn key(&mut self, key: &keys::CoseKey) -> CoseResult {
        let alg = self.header.alg.ok_or(CoseError::MissingAlg())?;
        key.verify_kty()?;
        if algs::ECDH_ALGS.contains(&alg) {
            if !keys::ECDH_KTY.contains(key.kty.as_ref().ok_or(CoseError::MissingKTY())?) {
                return Err(CoseError::InvalidKTY());
            }
            if key.alg != None {
                if key.alg.ok_or(CoseError::MissingAlg())? != alg {
                    return Err(CoseError::AlgsDontMatch());
                }
            }
        } else if (alg != algs::DIRECT
            && !algs::A_KW.contains(&alg)
            && !algs::RSA_OAEP.contains(&alg))
            && key.alg.ok_or(CoseError::MissingAlg())? != alg
        {
            return Err(CoseError::AlgsDontMatch());
        }
        if algs::SIGNING_ALGS.contains(&alg) {
            if key.key_ops.contains(&keys::KEY_OPS_SIGN) {
                self.s_key = key.get_s_key()?;
            }
            if key.key_ops.contains(&keys::KEY_OPS_VERIFY) {
                self.pub_key = key.get_pub_key()?;
            }
        } else if algs::KEY_DISTRIBUTION_ALGS.contains(&alg) || algs::ENCRYPT_ALGS.contains(&alg) {
            if KEY_OPS_SKEY.iter().any(|i| key.key_ops.contains(i)) {
                self.s_key = key.get_s_key()?;
            }
            if algs::ECDH_ALGS.contains(&alg) {
                if key.key_ops.len() == 0 {
                    self.pub_key = key.get_pub_key()?;
                }
            }
        }
        self.crv = key.crv;
        self.base_iv = key.base_iv.clone();
        self.key_ops = key.key_ops.clone();
        Ok(())
    }

    pub(crate) fn enc(
        &mut self,
        content: &Vec<u8>,
        external_aad: &Vec<u8>,
        body_protected: &Vec<u8>,
        alg: &i32,
        iv: &Vec<u8>,
    ) -> CoseResultWithRet<Vec<u8>> {
        if !self.key_ops.contains(&keys::KEY_OPS_ENCRYPT) {
            return Err(CoseError::KeyOpNotSupported());
        }
        Ok(cose_struct::gen_cipher(
            &self.s_key,
            alg,
            iv,
            &external_aad,
            &self.context,
            &body_protected,
            &content,
        )?)
    }

    pub(crate) fn sign(
        &mut self,
        content: &Vec<u8>,
        external_aad: &Vec<u8>,
        body_protected: &Vec<u8>,
    ) -> CoseResult {
        self.ph_bstr = self.header.get_protected_bstr(false)?;
        if !self.key_ops.contains(&keys::KEY_OPS_SIGN) {
            return Err(CoseError::KeyOpNotSupported());
        }
        self.payload = cose_struct::gen_sig(
            &self.s_key,
            &self.header.alg.ok_or(CoseError::MissingAlg())?,
            &self.crv,
            &external_aad,
            &self.context,
            &body_protected,
            &self.ph_bstr,
            &content,
        )?;
        Ok(())
    }
    pub(crate) fn verify(
        &self,
        content: &Vec<u8>,
        external_aad: &Vec<u8>,
        body_protected: &Vec<u8>,
    ) -> CoseResultWithRet<bool> {
        if !self.key_ops.contains(&keys::KEY_OPS_VERIFY) {
            return Err(CoseError::KeyOpNotSupported());
        }
        Ok(cose_struct::verify_sig(
            &self.pub_key,
            &self.header.alg.ok_or(CoseError::MissingAlg())?,
            &self.crv,
            &external_aad,
            &self.context,
            &body_protected,
            &self.ph_bstr,
            &content,
            &self.payload,
        )?)
    }

    pub(crate) fn mac(
        &mut self,
        content: &Vec<u8>,
        external_aad: &Vec<u8>,
        body_protected: &Vec<u8>,
        alg: &i32,
    ) -> CoseResultWithRet<Vec<u8>> {
        self.ph_bstr = self.header.get_protected_bstr(false)?;
        if !self.key_ops.contains(&keys::KEY_OPS_MAC) {
            return Err(CoseError::KeyOpNotSupported());
        }
        Ok(cose_struct::gen_mac(
            &self.s_key,
            &alg,
            &external_aad,
            &self.context,
            &body_protected,
            &content,
        )?)
    }

    /// Adds the counter signature to the CoseAgent.
    ///
    /// Function to use when signature was produce externally to the module.
    /// This function is to use only in the context of counter signatures, not message
    /// recipients/signers.
    pub fn add_signature(&mut self, signature: Vec<u8>) -> CoseResult {
        if self.context != cose_struct::COUNTER_SIGNATURE {
            return Err(CoseError::InvalidContext());
        }
        self.payload = signature;
        Ok(())
    }

    pub(crate) fn get_sign_content(
        &mut self,
        content: &Vec<u8>,
        external_aad: &Vec<u8>,
        body_protected: &Vec<u8>,
    ) -> CoseResultWithRet<Vec<u8>> {
        if self.context != cose_struct::COUNTER_SIGNATURE {
            return Err(CoseError::InvalidContext());
        }
        self.ph_bstr = self.header.get_protected_bstr(false)?;
        cose_struct::get_to_sign(
            &external_aad,
            cose_struct::COUNTER_SIGNATURE,
            &body_protected,
            &self.ph_bstr,
            &content,
        )
    }

    /// Adds a counter signature to the signer/recipient.
    pub fn counter_sig(
        &self,
        external_aad: Option<Vec<u8>>,
        counter: &mut CoseAgent,
    ) -> CoseResult {
        if !self.enc {
            Err(CoseError::MissingPayload())
        } else {
            let aead = match external_aad {
                None => Vec::new(),
                Some(v) => v,
            };
            counter.sign(&self.payload, &aead, &self.ph_bstr)?;
            Ok(())
        }
    }

    /// Function to get the content to sign by the counter signature.
    ///
    /// This function is meant to be called if the counter signature process needs to be external
    /// to this crate, like a timestamp authority.
    pub fn get_to_sign(
        &self,
        external_aad: Option<Vec<u8>>,
        counter: &mut CoseAgent,
    ) -> CoseResultWithRet<Vec<u8>> {
        if !self.enc {
            Err(CoseError::MissingPayload())
        } else {
            let aead = match external_aad {
                None => Vec::new(),
                Some(v) => v,
            };
            counter.get_sign_content(&self.payload, &aead, &self.ph_bstr)
        }
    }

    /// Function to get the content to verify with the counter signature.
    ///
    /// This function is meant to be called if the counter signature process needs to be external
    /// to this crate, like a timestamp authority.
    pub fn get_to_verify(
        &mut self,
        external_aad: Option<Vec<u8>>,
        counter: &usize,
    ) -> CoseResultWithRet<Vec<u8>> {
        if !self.enc {
            Err(CoseError::MissingPayload())
        } else {
            let aead = match external_aad {
                None => Vec::new(),
                Some(v) => v,
            };
            self.header.counters[*counter].get_sign_content(&self.payload, &aead, &self.ph_bstr)
        }
    }

    /// Function that verifies a given counter signature on the respective signer/recipient.
    pub fn counters_verify(&mut self, external_aad: Option<Vec<u8>>, counter: usize) -> CoseResult {
        if !self.enc {
            Err(CoseError::MissingPayload())
        } else {
            let aead = match external_aad {
                None => Vec::new(),
                Some(v) => v,
            };
            if self.header.counters[counter].verify(&self.payload, &aead, &self.ph_bstr)? {
                Ok(())
            } else {
                Err(CoseError::InvalidCounterSignature())
            }
        }
    }

    /// Function that adds a counter signature which was signed externally with the use of
    /// [get_to_sign](#method.get_to_sign)
    pub fn add_counter_sig(&mut self, counter: CoseAgent) -> CoseResult {
        if !algs::SIGNING_ALGS.contains(&counter.header.alg.ok_or(CoseError::MissingAlg())?) {
            return Err(CoseError::InvalidAlg());
        }
        if counter.context != cose_struct::COUNTER_SIGNATURE {
            return Err(CoseError::InvalidContext());
        }
        if self.header.unprotected.contains(&COUNTER_SIG) {
            self.header.counters.push(counter);
            Ok(())
        } else {
            self.header.counters.push(counter);
            self.header.remove_label(COUNTER_SIG);
            self.header.unprotected.push(COUNTER_SIG);
            Ok(())
        }
    }

    pub(crate) fn derive_key(
        &mut self,
        cek: &Vec<u8>,
        size: usize,
        sender: bool,
        true_alg: &i32,
    ) -> CoseResultWithRet<Vec<u8>> {
        if self.ph_bstr.len() <= 0 {
            self.ph_bstr = self.header.get_protected_bstr(false)?;
        }
        let alg = self.header.alg.as_ref().ok_or(CoseError::MissingAlg())?;
        if algs::A_KW.contains(alg) {
            if sender {
                self.payload = algs::aes_key_wrap(&self.s_key, size, &cek)?;
            } else {
                return Ok(algs::aes_key_unwrap(&self.s_key, size, &cek)?);
            }
            return Ok(cek.to_vec());
        } else if algs::RSA_OAEP.contains(alg) {
            if sender {
                self.payload = algs::rsa_oaep_enc(&self.s_key, size, &cek, alg)?;
            } else {
                return Ok(algs::rsa_oaep_dec(&self.s_key, size, &cek, alg)?);
            }
            return Ok(cek.to_vec());
        } else if algs::D_HA.contains(alg) {
            let mut kdf_context = cose_struct::gen_kdf(
                true_alg,
                &self.header.party_u_identity,
                &self.header.party_u_nonce,
                &self.header.party_u_other,
                &self.header.party_v_identity,
                &self.header.party_v_nonce,
                &self.header.party_v_other,
                size as u16 * 8,
                &self.ph_bstr,
                &self.header.pub_other,
                &self.header.priv_info,
            )?;
            return Ok(algs::hkdf(
                size,
                &self.s_key,
                self.header.salt.as_ref(),
                &mut kdf_context,
                self.header.alg.unwrap(),
            )?);
        } else if algs::D_HS.contains(alg) {
            let mut kdf_context = cose_struct::gen_kdf(
                true_alg,
                &self.header.party_u_identity,
                &self.header.party_u_nonce,
                &self.header.party_u_other,
                &self.header.party_v_identity,
                &self.header.party_v_nonce,
                &self.header.party_v_other,
                size as u16 * 8,
                &self.ph_bstr,
                &self.header.pub_other,
                &self.header.priv_info,
            )?;
            return Ok(algs::hkdf(
                size,
                &self.s_key,
                self.header.salt.as_ref(),
                &mut kdf_context,
                self.header.alg.unwrap(),
            )?);
        } else if algs::ECDH_H.contains(alg) || algs::ECDH_A.contains(alg) {
            let (receiver_key, sender_key, crv_rec, crv_send);
            if sender {
                if self.pub_key.len() == 0 {
                    return Err(CoseError::MissingKey());
                }
                receiver_key = self.pub_key.clone();
                if self.header.x5_private.len() > 0 {
                    sender_key = self.header.x5_private.clone();
                    crv_send = None;
                } else {
                    sender_key = self.header.ecdh_key.get_s_key()?;
                    crv_send = Some(self.header.ecdh_key.crv.unwrap());
                }
                crv_rec = Some(self.crv.unwrap());
            } else {
                if self.s_key.len() == 0 {
                    return Err(CoseError::MissingKey());
                }
                if self.header.x5chain_sender != None {
                    algs::verify_chain(self.header.x5chain_sender.as_ref().unwrap())?;
                    receiver_key = self.header.x5chain_sender.as_ref().unwrap()[0].clone();
                    crv_rec = None;
                } else {
                    receiver_key = self.header.ecdh_key.get_pub_key()?;
                    crv_rec = Some(self.crv.unwrap());
                }
                sender_key = self.s_key.clone();
                crv_send = Some(self.crv.unwrap());
            }
            let shared = algs::ecdh_derive_key(crv_rec, crv_send, &receiver_key, &sender_key)?;

            if algs::ECDH_H.contains(alg) {
                let mut kdf_context = cose_struct::gen_kdf(
                    true_alg,
                    &self.header.party_u_identity,
                    &self.header.party_u_nonce,
                    &self.header.party_u_other,
                    &self.header.party_v_identity,
                    &self.header.party_v_nonce,
                    &self.header.party_v_other,
                    size as u16 * 8,
                    &self.ph_bstr,
                    &self.header.pub_other,
                    &self.header.priv_info,
                )?;
                return Ok(algs::hkdf(
                    size,
                    &shared,
                    self.header.salt.as_ref(),
                    &mut kdf_context,
                    self.header.alg.unwrap(),
                )?);
            } else {
                let size_akw = algs::get_cek_size(&alg)?;

                let alg_akw;
                if [algs::ECDH_ES_A128KW, algs::ECDH_SS_A128KW].contains(alg) {
                    alg_akw = algs::A128KW;
                } else if [algs::ECDH_ES_A192KW, algs::ECDH_SS_A192KW].contains(alg) {
                    alg_akw = algs::A192KW;
                } else {
                    alg_akw = algs::A256KW;
                }

                let mut kdf_context = cose_struct::gen_kdf(
                    &alg_akw,
                    &self.header.party_u_identity,
                    &self.header.party_u_nonce,
                    &self.header.party_u_other,
                    &self.header.party_v_identity,
                    &self.header.party_v_nonce,
                    &self.header.party_v_other,
                    size_akw as u16 * 8,
                    &self.ph_bstr,
                    &self.header.pub_other,
                    &self.header.priv_info,
                )?;
                let kek = algs::hkdf(
                    size_akw,
                    &shared,
                    self.header.salt.as_ref(),
                    &mut kdf_context,
                    self.header.alg.unwrap(),
                )?;
                if sender {
                    self.payload = algs::aes_key_wrap(&kek, size, &cek)?;
                } else {
                    return Ok(algs::aes_key_unwrap(&kek, size, &cek)?);
                }
                return Ok(cek.to_vec());
            }
        } else {
            return Err(CoseError::InvalidAlg());
        }
    }

    pub(crate) fn decode(&mut self, d: &mut Decoder<Cursor<Vec<u8>>>) -> CoseResult {
        if self.ph_bstr.len() > 0 {
            self.header.decode_protected_bstr(&self.ph_bstr)?;
        }
        self.header
            .decode_unprotected(d, self.context == cose_struct::COUNTER_SIGNATURE)?;
        self.payload = d.bytes()?;
        Ok(())
    }

    pub(crate) fn encode(&mut self, e: &mut Encoder<Vec<u8>>) -> CoseResult {
        e.array(SIZE)?;
        e.bytes(&self.ph_bstr)?;
        self.header.encode_unprotected(e)?;
        e.bytes(&self.payload)?;
        Ok(())
    }
}
