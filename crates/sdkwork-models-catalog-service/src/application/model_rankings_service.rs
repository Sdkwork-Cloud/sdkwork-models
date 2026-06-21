use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::domain::DomainResult;
use crate::ports::{
    normalize_model_ranking_filter_value, normalize_rank_scope, normalize_scope_ids,
    ModelRankingRefreshJobHistoryPage, ModelRankingRefreshJobHistoryQuery,
    ModelRankingRefreshJobHistoryReadFuture, ModelRankingRefreshJobHistoryReadStore,
    ModelRankingRefreshStatus, ModelRankingRefreshStatusQuery, ModelRankingRefreshStatusReadFuture,
    ModelRankingRefreshStatusReadStore, ModelRankingsCacheInvalidation,
    ModelRankingsCacheInvalidator, ModelRankingsQuery, ModelRankingsReadFuture,
    ModelRankingsReadModelStore, ModelRankingsReadStore, ModelRankingsSnapshot,
    ModelRankingsSubject,
};

const MIN_CACHE_TTL_SECONDS: i64 = 1;
const DEFAULT_CACHE_TTL_SECONDS: i64 = 60;
const MAX_CACHE_ENTRIES: usize = 64;
const MIN_RANKING_LIMIT: i64 = 1;
const MAX_RANKING_LIMIT: i64 = 200;
const MIN_JOB_HISTORY_LIMIT: i64 = 1;
const MAX_JOB_HISTORY_LIMIT: i64 = 100;

type MonotonicNow = Arc<dyn Fn() -> Instant + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModelRankingsCacheKey {
    tenant_id: i64,
    organization_id: i64,
    rank_scope: Option<String>,
    vendor_code: Option<String>,
    modality: Option<String>,
    search_query: Option<String>,
    limit: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModelRankingRefreshStatusCacheKey {
    tenant_id: i64,
    organization_id: i64,
    rank_scope: Option<String>,
}

#[derive(Debug, Clone)]
struct ModelRankingsCacheEntry {
    key: ModelRankingsCacheKey,
    expires_at: Instant,
    snapshot: ModelRankingsSnapshot,
}

#[derive(Debug, Clone)]
struct ModelRankingRefreshStatusCacheEntry {
    key: ModelRankingRefreshStatusCacheKey,
    expires_at: Instant,
    status: ModelRankingRefreshStatus,
}

#[derive(Debug, Default)]
struct ModelRankingsCache {
    entries: VecDeque<ModelRankingsCacheEntry>,
    status_entries: VecDeque<ModelRankingRefreshStatusCacheEntry>,
}

#[derive(Clone)]
pub struct ModelRankingsService {
    read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync>,
    cache: Arc<Mutex<ModelRankingsCache>>,
    fallback_ttl_seconds: i64,
    now: MonotonicNow,
}

impl ModelRankingsService {
    pub fn new(read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync>) -> Self {
        Self::with_fallback_ttl_seconds_and_clock(
            read_store,
            DEFAULT_CACHE_TTL_SECONDS,
            Instant::now,
        )
    }

    pub fn with_fallback_ttl_seconds(
        read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync>,
        fallback_ttl_seconds: i64,
    ) -> Self {
        Self::with_fallback_ttl_seconds_and_clock(read_store, fallback_ttl_seconds, Instant::now)
    }

    pub fn with_fallback_ttl_seconds_and_clock(
        read_store: Arc<dyn ModelRankingsReadModelStore + Send + Sync>,
        fallback_ttl_seconds: i64,
        now: impl Fn() -> Instant + Send + Sync + 'static,
    ) -> Self {
        Self {
            read_store,
            cache: Arc::new(Mutex::new(ModelRankingsCache::default())),
            fallback_ttl_seconds: fallback_ttl_seconds.max(MIN_CACHE_TTL_SECONDS),
            now: Arc::new(now),
        }
    }
}

impl ModelRankingsReadStore for ModelRankingsService {
    fn load_model_rankings<'a>(
        &'a self,
        query: ModelRankingsQuery,
        subject: Option<ModelRankingsSubject>,
    ) -> ModelRankingsReadFuture<'a> {
        Box::pin(async move {
            let query = normalize_rankings_query(query)?;
            let subject = normalize_subject_option(subject);
            let key = cache_key(&query, subject);
            if let Some(snapshot) = self.load_cached(&key) {
                return Ok(snapshot);
            }

            let snapshot = self.read_store.load_model_rankings(query, subject).await?;
            self.store_cached(key, snapshot.clone());
            DomainResult::Ok(snapshot)
        })
    }
}

impl ModelRankingRefreshStatusReadStore for ModelRankingsService {
    fn load_model_ranking_refresh_status<'a>(
        &'a self,
        query: ModelRankingRefreshStatusQuery,
        subject: Option<ModelRankingsSubject>,
    ) -> ModelRankingRefreshStatusReadFuture<'a> {
        Box::pin(async move {
            let query = normalize_status_query(query);
            let subject = normalize_subject_option(subject);
            let key = status_cache_key(&query, subject);
            if let Some(status) = self.load_status_cached(&key) {
                return Ok(status);
            }

            let status = self
                .read_store
                .load_model_ranking_refresh_status(query, subject)
                .await?;
            self.store_status_cached(key, status.clone());
            DomainResult::Ok(status)
        })
    }
}

impl ModelRankingRefreshJobHistoryReadStore for ModelRankingsService {
    fn load_model_ranking_refresh_jobs<'a>(
        &'a self,
        query: ModelRankingRefreshJobHistoryQuery,
        subject: Option<ModelRankingsSubject>,
    ) -> ModelRankingRefreshJobHistoryReadFuture<'a> {
        Box::pin(async move {
            let query = normalize_job_history_query(query)?;
            let subject = normalize_subject_option(subject);
            let page: ModelRankingRefreshJobHistoryPage = self
                .read_store
                .load_model_ranking_refresh_jobs(query, subject)
                .await?;
            DomainResult::Ok(page)
        })
    }
}

impl ModelRankingsCacheInvalidator for ModelRankingsService {
    fn invalidate_model_rankings_cache(&self, invalidation: ModelRankingsCacheInvalidation) {
        let (invalidation_tenant_id, invalidation_organization_id) =
            normalize_scope_ids(invalidation.tenant_id, invalidation.organization_id);
        let invalidation = ModelRankingsCacheInvalidation {
            tenant_id: invalidation_tenant_id,
            organization_id: invalidation_organization_id,
            rank_scope: invalidation.rank_scope,
        };
        let rank_scope = invalidation
            .rank_scope
            .as_deref()
            .map(|rank_scope| normalize_rank_scope(Some(rank_scope)));
        let Ok(mut cache) = self.cache.lock() else {
            return;
        };
        cache.entries.retain(|entry| {
            !same_scope(
                entry.key.tenant_id,
                entry.key.organization_id,
                entry.key.rank_scope.as_deref(),
                &invalidation,
                rank_scope.as_deref(),
            )
        });
        cache.status_entries.retain(|entry| {
            !same_scope(
                entry.key.tenant_id,
                entry.key.organization_id,
                entry.key.rank_scope.as_deref(),
                &invalidation,
                rank_scope.as_deref(),
            )
        });
    }
}

impl ModelRankingsService {
    fn load_cached(&self, key: &ModelRankingsCacheKey) -> Option<ModelRankingsSnapshot> {
        let now = (self.now.as_ref())();
        let mut cache = self.cache.lock().ok()?;
        cache.entries.retain(|entry| entry.expires_at > now);
        cache
            .entries
            .iter()
            .find(|entry| &entry.key == key)
            .map(|entry| entry.snapshot.clone())
    }

    fn store_cached(&self, key: ModelRankingsCacheKey, snapshot: ModelRankingsSnapshot) {
        let ttl_seconds = if snapshot.source.cache_max_age_seconds >= MIN_CACHE_TTL_SECONDS {
            snapshot.source.cache_max_age_seconds
        } else {
            self.fallback_ttl_seconds
        };
        let Ok(mut cache) = self.cache.lock() else {
            return;
        };
        cache.entries.retain(|entry| entry.key != key);
        while cache.entries.len() >= MAX_CACHE_ENTRIES {
            cache.entries.pop_front();
        }
        cache.entries.push_back(ModelRankingsCacheEntry {
            key,
            expires_at: (self.now.as_ref())() + Duration::from_secs(ttl_seconds as u64),
            snapshot,
        });
    }

    fn load_status_cached(
        &self,
        key: &ModelRankingRefreshStatusCacheKey,
    ) -> Option<ModelRankingRefreshStatus> {
        let now = (self.now.as_ref())();
        let mut cache = self.cache.lock().ok()?;
        cache.status_entries.retain(|entry| entry.expires_at > now);
        cache
            .status_entries
            .iter()
            .find(|entry| &entry.key == key)
            .map(|entry| entry.status.clone())
    }

    fn store_status_cached(
        &self,
        key: ModelRankingRefreshStatusCacheKey,
        status: ModelRankingRefreshStatus,
    ) {
        let ttl_seconds = if status.cache_max_age_seconds >= MIN_CACHE_TTL_SECONDS {
            status.cache_max_age_seconds
        } else {
            self.fallback_ttl_seconds
        };
        let Ok(mut cache) = self.cache.lock() else {
            return;
        };
        cache.status_entries.retain(|entry| entry.key != key);
        while cache.status_entries.len() >= MAX_CACHE_ENTRIES {
            cache.status_entries.pop_front();
        }
        cache
            .status_entries
            .push_back(ModelRankingRefreshStatusCacheEntry {
                key,
                expires_at: (self.now.as_ref())() + Duration::from_secs(ttl_seconds as u64),
                status,
            });
    }
}

fn same_scope(
    tenant_id: i64,
    organization_id: i64,
    entry_rank_scope: Option<&str>,
    invalidation: &ModelRankingsCacheInvalidation,
    invalidated_rank_scope: Option<&str>,
) -> bool {
    if !same_tenant_scope(tenant_id, organization_id, invalidation) {
        return false;
    }
    match invalidated_rank_scope {
        Some(rank_scope) => entry_rank_scope
            .map(|value| value.eq_ignore_ascii_case(rank_scope))
            .unwrap_or(false),
        None => true,
    }
}

fn same_tenant_scope(
    tenant_id: i64,
    organization_id: i64,
    invalidation: &ModelRankingsCacheInvalidation,
) -> bool {
    if invalidation.tenant_id <= 0 && invalidation.organization_id <= 0 {
        return true;
    }
    if invalidation.organization_id <= 0 {
        return tenant_id == invalidation.tenant_id;
    }
    tenant_id == invalidation.tenant_id && organization_id == invalidation.organization_id
}

fn cache_key(
    query: &ModelRankingsQuery,
    subject: Option<ModelRankingsSubject>,
) -> ModelRankingsCacheKey {
    let subject = subject.unwrap_or(ModelRankingsSubject {
        tenant_id: 0,
        organization_id: 0,
        user_id: 0,
    });
    let (tenant_id, organization_id) =
        normalize_scope_ids(subject.tenant_id, subject.organization_id);
    ModelRankingsCacheKey {
        tenant_id,
        organization_id,
        rank_scope: normalized_rank_scope(query.rank_scope.as_deref()),
        vendor_code: query.vendor_code.clone(),
        modality: query.modality.clone(),
        search_query: query.search_query.clone(),
        limit: query.limit,
    }
}

fn normalize_subject_option(subject: Option<ModelRankingsSubject>) -> Option<ModelRankingsSubject> {
    subject.map(|subject| {
        let (tenant_id, organization_id) =
            normalize_scope_ids(subject.tenant_id, subject.organization_id);
        ModelRankingsSubject {
            tenant_id,
            organization_id,
            user_id: subject.user_id.max(0),
        }
    })
}

fn normalize_rankings_query(query: ModelRankingsQuery) -> DomainResult<ModelRankingsQuery> {
    if !(MIN_RANKING_LIMIT..=MAX_RANKING_LIMIT).contains(&query.limit) {
        return Err(crate::domain::DomainError::new(format!(
            "model rankings limit must be between {MIN_RANKING_LIMIT} and {MAX_RANKING_LIMIT}"
        )));
    }

    Ok(ModelRankingsQuery {
        rank_scope: normalized_rank_scope(query.rank_scope.as_deref()),
        vendor_code: normalized_filter_value(query.vendor_code.as_deref()),
        modality: normalized_filter_value(query.modality.as_deref()),
        search_query: normalized_filter_value(query.search_query.as_deref()),
        limit: query.limit,
    })
}

fn normalize_status_query(query: ModelRankingRefreshStatusQuery) -> ModelRankingRefreshStatusQuery {
    ModelRankingRefreshStatusQuery {
        rank_scope: normalized_rank_scope(query.rank_scope.as_deref()),
    }
}

fn normalize_job_history_query(
    query: ModelRankingRefreshJobHistoryQuery,
) -> DomainResult<ModelRankingRefreshJobHistoryQuery> {
    if !(MIN_JOB_HISTORY_LIMIT..=MAX_JOB_HISTORY_LIMIT).contains(&query.limit) {
        return Err(crate::domain::DomainError::new(format!(
            "model ranking refresh job history limit must be between {MIN_JOB_HISTORY_LIMIT} and {MAX_JOB_HISTORY_LIMIT}"
        )));
    }

    Ok(ModelRankingRefreshJobHistoryQuery {
        rank_scope: normalized_rank_scope(query.rank_scope.as_deref()),
        limit: query.limit,
    })
}

fn status_cache_key(
    query: &ModelRankingRefreshStatusQuery,
    subject: Option<ModelRankingsSubject>,
) -> ModelRankingRefreshStatusCacheKey {
    let subject = subject.unwrap_or(ModelRankingsSubject {
        tenant_id: 0,
        organization_id: 0,
        user_id: 0,
    });
    let (tenant_id, organization_id) =
        normalize_scope_ids(subject.tenant_id, subject.organization_id);
    ModelRankingRefreshStatusCacheKey {
        tenant_id,
        organization_id,
        rank_scope: normalized_rank_scope(query.rank_scope.as_deref()),
    }
}

fn normalized_rank_scope(value: Option<&str>) -> Option<String> {
    Some(normalize_rank_scope(value))
}

fn normalized_filter_value(value: Option<&str>) -> Option<String> {
    normalize_model_ranking_filter_value(value)
}
