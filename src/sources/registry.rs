//! Registry for managing research source plugins.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::{Source, SourceError};
use crate::config::SourceConfig;

// Conditionally import source types based on feature flags
#[cfg(feature = "source-acm")]
use super::acm::AcmSource;
#[cfg(feature = "source-arxiv")]
use super::arxiv::ArxivSource;
#[cfg(feature = "source-base")]
use super::base::BaseSource;
#[cfg(feature = "source-biorxiv")]
use super::biorxiv::BiorxivSource;
#[cfg(feature = "source-connected_papers")]
use super::connected_papers::ConnectedPapersSource;
#[cfg(feature = "source-core-repo")]
use super::core::CoreSource;
#[cfg(feature = "source-crossref")]
use super::crossref::CrossRefSource;
#[cfg(feature = "source-dblp")]
use super::dblp::DblpSource;
#[cfg(feature = "source-dimensions")]
use super::dimensions::DimensionsSource;
#[cfg(feature = "source-doaj")]
use super::doaj::DoajSource;
#[cfg(feature = "source-europe_pmc")]
use super::europe_pmc::EuropePmcSource;
#[cfg(feature = "source-google_scholar")]
use super::google_scholar::GoogleScholarSource;
#[cfg(feature = "source-hal")]
use super::hal::HalSource;
#[cfg(feature = "source-iacr")]
use super::iacr::IacrSource;
#[cfg(feature = "source-ieee_xplore")]
use super::ieee_xplore::IeeeXploreSource;
#[cfg(feature = "source-jstor")]
use super::jstor::JstorSource;
#[cfg(feature = "source-mdpi")]
use super::mdpi::MdpiSource;
#[cfg(feature = "source-openalex")]
use super::openalex::OpenAlexSource;
#[cfg(feature = "source-osf")]
use super::osf::OsfSource;
#[cfg(feature = "source-pmc")]
use super::pmc::PmcSource;
#[cfg(feature = "source-pubmed")]
use super::pubmed::PubMedSource;
#[cfg(feature = "source-scispace")]
use super::scispace::ScispaceSource;
#[cfg(feature = "source-semantic")]
use super::semantic::SemanticScholarSource;
#[cfg(feature = "source-springer")]
use super::springer::SpringerSource;
#[cfg(feature = "source-ssrn")]
use super::ssrn::SsrnSource;
#[cfg(feature = "source-unpaywall")]
use super::unpaywall::UnpaywallSource;
#[cfg(feature = "source-worldwidescience")]
use super::worldwidescience::WorldWideScienceSource;
#[cfg(feature = "source-zenodo")]
use super::zenodo::ZenodoSource;

/// Result of source filtering from config/environment
#[derive(Debug, Clone, Default)]
struct SourceFilter {
    /// Set of explicitly enabled sources (None means all are enabled)
    enabled: Option<HashSet<String>>,
    /// Set of explicitly disabled sources (None means none are disabled)
    disabled: Option<HashSet<String>>,
}

impl SourceFilter {
    /// Create a new filter from config (which may include env vars)
    fn from_config(config: &SourceConfig) -> Self {
        let enabled = config
            .enabled_sources
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|value| {
                value
                    .split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect::<HashSet<_>>()
            })
            .filter(|set| !set.is_empty());

        let disabled = config
            .disabled_sources
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|value| {
                value
                    .split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect::<HashSet<_>>()
            })
            .filter(|set| !set.is_empty());

        Self { enabled, disabled }
    }

    /// Check if a source should be enabled based on the filter
    ///
    /// Logic:
    /// - If ENABLE is set and DISABLE is not: only enabled sources
    /// - If DISABLE is set and ENABLE is not: all except disabled sources
    /// - If both are set: enabled sources minus disabled sources
    /// - If neither is set: all sources enabled
    fn is_enabled(&self, source_id: &str) -> bool {
        let id_lower = source_id.to_lowercase();

        match (&self.enabled, &self.disabled) {
            // Both specified: enabled minus disabled
            (Some(enabled), Some(disabled)) => {
                enabled.contains(&id_lower) && !disabled.contains(&id_lower)
            }
            // Only enabled specified: must be in enabled set
            (Some(enabled), None) => enabled.contains(&id_lower),
            // Only disabled specified: must not be in disabled set
            (None, Some(disabled)) => !disabled.contains(&id_lower),
            // Neither specified: all enabled
            (None, None) => true,
        }
    }
}

bitflags::bitflags! {
    /// Capabilities that a source can support
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SourceCapabilities: u32 {
        const SEARCH = 1 << 0;
        const DOWNLOAD = 1 << 1;
        const READ = 1 << 2;
        const CITATIONS = 1 << 3;
        const DOI_LOOKUP = 1 << 4;
        const AUTHOR_SEARCH = 1 << 5;
    }
}

/// Registry for all available research sources
///
/// The SourceRegistry manages all available source plugins and provides
/// methods to query and use them.
#[derive(Debug, Clone)]
pub struct SourceRegistry {
    sources: HashMap<String, Arc<dyn Source>>,
}

impl SourceRegistry {
    /// Create a new registry with all available sources
    pub fn new() -> Self {
        Self::try_new().expect("Failed to initialize any sources")
    }

    /// Try to create a new registry with all available sources
    ///
    /// This will:
    /// 1. Filter sources based on config file or environment variables
    /// 2. Skip sources that fail to initialize (e.g., missing API keys)
    /// 3. Return an error only if no sources could be initialized
    pub fn try_new() -> Result<Self, SourceError> {
        let source_config = crate::config::get_config().sources;
        let filter = SourceFilter::from_config(&source_config);
        let mut registry = Self {
            sources: HashMap::new(),
        };

        // Helper macro to register a source with error handling
        macro_rules! try_register {
            ($source:expr) => {
                if let Ok(source) = $source {
                    let source_id = source.id().to_string();
                    if filter.is_enabled(&source_id) {
                        registry.register(Arc::new(source));
                        tracing::info!("Registered source: {}", source_id);
                    } else {
                        tracing::debug!("Source '{}' filtered out by source filter", source_id);
                    }
                } else {
                    tracing::warn!("Skipping source: initialization failed");
                }
            };
        }

        // Register all available sources (will skip any that fail to initialize)
        // Each source is conditionally compiled based on feature flags
        #[cfg(feature = "source-arxiv")]
        try_register!(ArxivSource::new());

        #[cfg(feature = "source-pubmed")]
        try_register!(PubMedSource::new());

        #[cfg(feature = "source-biorxiv")]
        try_register!(BiorxivSource::new());

        #[cfg(feature = "source-semantic")]
        try_register!(SemanticScholarSource::new());

        #[cfg(feature = "source-openalex")]
        try_register!(OpenAlexSource::new());

        #[cfg(feature = "source-crossref")]
        try_register!(CrossRefSource::new());

        #[cfg(feature = "source-iacr")]
        try_register!(IacrSource::new());

        #[cfg(feature = "source-pmc")]
        try_register!(PmcSource::new());

        #[cfg(feature = "source-hal")]
        try_register!(HalSource::new());

        #[cfg(feature = "source-dblp")]
        try_register!(DblpSource::new());

        #[cfg(feature = "source-dimensions")]
        try_register!(DimensionsSource::new());

        #[cfg(feature = "source-ieee_xplore")]
        try_register!(IeeeXploreSource::new());

        #[cfg(feature = "source-core-repo")]
        try_register!(CoreSource::new());

        #[cfg(feature = "source-zenodo")]
        try_register!(ZenodoSource::new());

        #[cfg(feature = "source-unpaywall")]
        try_register!(UnpaywallSource::new());

        #[cfg(feature = "source-mdpi")]
        try_register!(MdpiSource::new());

        #[cfg(feature = "source-ssrn")]
        try_register!(SsrnSource::new());

        #[cfg(feature = "source-jstor")]
        try_register!(JstorSource::new());

        #[cfg(feature = "source-scispace")]
        try_register!(ScispaceSource::new());

        #[cfg(feature = "source-acm")]
        try_register!(AcmSource::new());

        #[cfg(feature = "source-connected_papers")]
        try_register!(ConnectedPapersSource::new());

        #[cfg(feature = "source-doaj")]
        try_register!(DoajSource::new());

        #[cfg(feature = "source-europe_pmc")]
        try_register!(EuropePmcSource::new());

        #[cfg(feature = "source-worldwidescience")]
        try_register!(WorldWideScienceSource::new());

        #[cfg(feature = "source-osf")]
        try_register!(OsfSource::new());

        #[cfg(feature = "source-base")]
        try_register!(BaseSource::new());

        #[cfg(feature = "source-springer")]
        try_register!(SpringerSource::new());

        #[cfg(feature = "source-google_scholar")]
        try_register!(GoogleScholarSource::new());

        if registry.is_empty() {
            return Err(SourceError::Other(
                "No sources could be initialized. Check configuration and API keys.".to_string(),
            ));
        }

        tracing::info!("Initialized {} sources", registry.len());

        Ok(registry)
    }

    /// Register a new source
    pub fn register(&mut self, source: Arc<dyn Source>) {
        self.sources.insert(source.id().to_string(), source);
    }

    /// Get a source by ID
    pub fn get(&self, id: &str) -> Option<&Arc<dyn Source>> {
        self.sources.get(id)
    }

    /// Get a source by ID, returning an error if not found
    pub fn get_required(&self, id: &str) -> Result<&Arc<dyn Source>, SourceError> {
        self.get(id)
            .ok_or_else(|| SourceError::NotFound(format!("Source '{}' not found", id)))
    }

    /// Get all registered sources
    pub fn all(&self) -> impl Iterator<Item = &Arc<dyn Source>> {
        self.sources.values()
    }

    /// Get all source IDs
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.sources.keys().map(|s| s.as_str())
    }

    /// Get sources that support a specific capability
    pub fn with_capability(&self, capability: SourceCapabilities) -> Vec<&Arc<dyn Source>> {
        self.all()
            .filter(|s| s.capabilities().contains(capability))
            .collect()
    }

    /// Get sources that support search
    pub fn searchable(&self) -> Vec<&Arc<dyn Source>> {
        self.with_capability(SourceCapabilities::SEARCH)
    }

    /// Get sources that support download
    pub fn downloadable(&self) -> Vec<&Arc<dyn Source>> {
        self.with_capability(SourceCapabilities::DOWNLOAD)
    }

    /// Get sources that support citations
    pub fn with_citations(&self) -> Vec<&Arc<dyn Source>> {
        self.with_capability(SourceCapabilities::CITATIONS)
    }

    /// Check if a source exists
    pub fn has(&self, id: &str) -> bool {
        self.sources.contains_key(id)
    }

    /// Get the number of registered sources
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_basic() {
        let registry = SourceRegistry::new();

        // Should have some sources (at least one)
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_get_source() {
        let registry = SourceRegistry::new();

        let arxiv = registry.get("arxiv");
        // arxiv should be available if not filtered
        if let Some(arxiv) = arxiv {
            assert_eq!(arxiv.id(), "arxiv");
        }

        let missing = registry.get("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_source_filter_only_enabled() {
        // Test: ENABLE only - only enabled sources
        std::env::set_var("RESEARCH_MASTER_ENABLED_SOURCES", "arxiv,pubmed");
        std::env::remove_var("RESEARCH_MASTER_DISABLED_SOURCES");

        let config = crate::config::get_config().sources;
        let filter = SourceFilter::from_config(&config);
        assert!(filter.is_enabled("arxiv"));
        assert!(filter.is_enabled("pubmed"));
        assert!(!filter.is_enabled("semantic"));
        assert!(filter.is_enabled("ARXIV")); // Case insensitive - ARXIV should be enabled

        std::env::remove_var("RESEARCH_MASTER_ENABLED_SOURCES");
        std::env::remove_var("RESEARCH_MASTER_DISABLED_SOURCES");
    }

    #[test]
    fn test_source_filter_only_disabled() {
        // Test: DISABLE only - all except disabled
        std::env::remove_var("RESEARCH_MASTER_ENABLED_SOURCES");
        std::env::set_var("RESEARCH_MASTER_DISABLED_SOURCES", "dblp,jstor");

        let config = crate::config::get_config().sources;
        let filter = SourceFilter::from_config(&config);
        assert!(filter.is_enabled("arxiv"));
        assert!(filter.is_enabled("pubmed"));
        assert!(!filter.is_enabled("dblp"));
        assert!(!filter.is_enabled("jstor"));
        assert!(!filter.is_enabled("DBLP")); // Case insensitive

        std::env::remove_var("RESEARCH_MASTER_ENABLED_SOURCES");
        std::env::remove_var("RESEARCH_MASTER_DISABLED_SOURCES");
    }

    #[test]
    fn test_source_filter_both_enabled_and_disabled() {
        // Test: Both ENABLE and DISABLE - enabled minus disabled
        std::env::set_var(
            "RESEARCH_MASTER_ENABLED_SOURCES",
            "arxiv,pubmed,semantic,dblp",
        );
        std::env::set_var("RESEARCH_MASTER_DISABLED_SOURCES", "dblp");

        let config = crate::config::get_config().sources;
        let filter = SourceFilter::from_config(&config);
        assert!(filter.is_enabled("arxiv"));
        assert!(filter.is_enabled("pubmed"));
        assert!(filter.is_enabled("semantic"));
        assert!(!filter.is_enabled("dblp")); // In enabled but also in disabled

        std::env::remove_var("RESEARCH_MASTER_ENABLED_SOURCES");
        std::env::remove_var("RESEARCH_MASTER_DISABLED_SOURCES");
    }

    #[test]
    fn test_source_filter_neither() {
        // Test: Neither set - all enabled
        std::env::remove_var("RESEARCH_MASTER_ENABLED_SOURCES");
        std::env::remove_var("RESEARCH_MASTER_DISABLED_SOURCES");

        let config = crate::config::get_config().sources;
        let filter = SourceFilter::from_config(&config);
        assert!(filter.is_enabled("arxiv"));
        assert!(filter.is_enabled("pubmed"));
        assert!(filter.is_enabled("semantic"));
        assert!(filter.is_enabled("dblp"));

        std::env::remove_var("RESEARCH_MASTER_ENABLED_SOURCES");
        std::env::remove_var("RESEARCH_MASTER_DISABLED_SOURCES");
    }

    #[test]
    fn test_source_filter_empty_values() {
        // Test: Empty values should be treated as not set
        std::env::set_var("RESEARCH_MASTER_ENABLED_SOURCES", "");
        std::env::set_var("RESEARCH_MASTER_DISABLED_SOURCES", "");

        let config = crate::config::get_config().sources;
        let filter = SourceFilter::from_config(&config);
        // Empty strings should result in all sources enabled
        assert!(filter.is_enabled("arxiv"));
        assert!(filter.is_enabled("pubmed"));

        std::env::remove_var("RESEARCH_MASTER_ENABLED_SOURCES");
        std::env::remove_var("RESEARCH_MASTER_DISABLED_SOURCES");
    }

    #[test]
    fn test_searchable_sources() {
        let registry = SourceRegistry::new();

        let searchable = registry.searchable();
        // Should have at least some searchable sources
        assert!(!searchable.is_empty());
    }

    #[test]
    fn test_capabilities() {
        let registry = SourceRegistry::new();

        // arXiv should support search, download, read (if available)
        if let Some(arxiv) = registry.get("arxiv") {
            assert!(arxiv.capabilities().contains(SourceCapabilities::SEARCH));
            assert!(arxiv.capabilities().contains(SourceCapabilities::DOWNLOAD));
            assert!(arxiv.capabilities().contains(SourceCapabilities::READ));
        }

        // Semantic Scholar should support citations (if available)
        if let Some(semantic) = registry.get("semantic") {
            assert!(semantic
                .capabilities()
                .contains(SourceCapabilities::CITATIONS));
            assert!(semantic
                .capabilities()
                .contains(SourceCapabilities::AUTHOR_SEARCH));
        }

        // DBLP should only support search (if available)
        if let Some(dblp) = registry.get("dblp") {
            assert!(dblp.capabilities().contains(SourceCapabilities::SEARCH));
            assert!(!dblp.capabilities().contains(SourceCapabilities::DOWNLOAD));
        }
    }
}
