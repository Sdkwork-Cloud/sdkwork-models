use crate::domain::{DomainError, DomainResult, GatewayApiKey};
use crate::ports::PricingCatalog;

pub trait ApiKeySecretHasher {
    fn hash_secret(&self, secret: &str) -> DomainResult<String>;
}

pub trait ApiKeySecretCodec {
    fn encode_secret(&self, secret: &str) -> DomainResult<String>;
    fn decode_secret(&self, encoded_secret: &str) -> DomainResult<String>;
}

pub struct ApiKeyAuthenticator<'a, C, H>
where
    C: PricingCatalog,
    H: ApiKeySecretHasher + ?Sized,
{
    catalog: &'a C,
    hasher: &'a H,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthenticateApiKeyQuery<'a> {
    pub credential_secret: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedApiKeyContext {
    pub api_key_id: i64,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub user_id: i64,
    pub api_key_name_snapshot: String,
    pub group_id: i64,
    pub group_code: String,
    pub pricing_plan_code: String,
}

impl<'a, C, H> ApiKeyAuthenticator<'a, C, H>
where
    C: PricingCatalog,
    H: ApiKeySecretHasher + ?Sized,
{
    pub fn new(catalog: &'a C, hasher: &'a H) -> Self {
        Self { catalog, hasher }
    }

    pub fn authenticate(
        &self,
        query: AuthenticateApiKeyQuery<'_>,
    ) -> DomainResult<AuthenticatedApiKeyContext> {
        let key_hash = self.hasher.hash_secret(query.credential_secret)?;
        let api_key = self.find_api_key(&key_hash)?;
        let group = self
            .catalog
            .find_channel_group(api_key.group_id)
            .ok_or_else(|| DomainError::new("channel group is not available"))?;

        Ok(AuthenticatedApiKeyContext {
            api_key_id: api_key.id,
            tenant_id: api_key.tenant_id,
            organization_id: api_key.organization_id,
            user_id: api_key.user_id,
            api_key_name_snapshot: api_key.display_name(),
            group_id: group.id,
            group_code: group.code,
            pricing_plan_code: group.pricing_plan_code,
        })
    }

    fn find_api_key(&self, key_hash: &str) -> DomainResult<GatewayApiKey> {
        self.catalog
            .find_api_key_by_hash(key_hash)
            .ok_or_else(|| DomainError::new("api key credential is invalid"))
    }
}
