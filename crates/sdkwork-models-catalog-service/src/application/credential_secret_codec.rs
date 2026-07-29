use crate::domain::DomainResult;

pub trait CredentialSecretCodec {
    fn encode_secret(&self, secret: &str) -> DomainResult<String>;
    fn decode_secret(&self, encoded_secret: &str) -> DomainResult<String>;
}
