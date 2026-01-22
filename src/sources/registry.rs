//! Registry for managing research source plugins.

use std::collections::HashMap;
use std::sync::Arc;

use super::{
    arxiv::ArxivSource, biorxiv::BiorxivSource, crossref::CrossRefSource, dblp::DblpSource,
    hal::HalSource, iacr::IacrSource, openalex::OpenAlexSource,
    pmc::PmcSource, pubmed::PubMedSource, semantic::SemanticScholarSource, ssrn::SsrnSource,
    Source, SourceError,
};

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
        let mut registry = Self {
            sources: HashMap::new(),
        };

        // Register all available sources
        registry.register(Arc::new(ArxivSource::new()));
        registry.register(Arc::new(PubMedSource::new()));
        registry.register(Arc::new(BiorxivSource::new()));
        registry.register(Arc::new(SemanticScholarSource::new()));
        registry.register(Arc::new(OpenAlexSource::new()));
        registry.register(Arc::new(CrossRefSource::new()));
        registry.register(Arc::new(IacrSource::new()));
        registry.register(Arc::new(PmcSource::new()));
        registry.register(Arc::new(HalSource::new()));
        registry.register(Arc::new(DblpSource::new()));
        registry.register(Arc::new(SsrnSource::new()));

        registry
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

        // Should have all 11 sources
        assert_eq!(registry.len(), 11);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_get_source() {
        let registry = SourceRegistry::new();

        let arxiv = registry.get("arxiv");
        assert!(arxiv.is_some());
        assert_eq!(arxiv.unwrap().id(), "arxiv");

        let missing = registry.get("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_all_sources_registered() {
        let registry = SourceRegistry::new();

        // Check all expected sources are registered
        let expected_sources = [
            "arxiv", "pubmed", "biorxiv", "semantic", "openalex",
            "crossref", "iacr", "pmc", "hal", "dblp", "ssrn",
        ];

        for source_id in expected_sources {
            assert!(
                registry.has(source_id),
                "Source '{}' should be registered",
                source_id
            );
        }
    }

    #[test]
    fn test_searchable_sources() {
        let registry = SourceRegistry::new();

        let searchable = registry.searchable();
        // All sources should be searchable
        assert!(searchable.len() >= 11);
    }

    #[test]
    fn test_capabilities() {
        let registry = SourceRegistry::new();

        // arXiv should support search, download, read
        let arxiv = registry.get("arxiv").unwrap();
        assert!(arxiv.capabilities().contains(SourceCapabilities::SEARCH));
        assert!(arxiv.capabilities().contains(SourceCapabilities::DOWNLOAD));
        assert!(arxiv.capabilities().contains(SourceCapabilities::READ));

        // Semantic Scholar should support citations
        let semantic = registry.get("semantic").unwrap();
        assert!(semantic.capabilities().contains(SourceCapabilities::CITATIONS));
        assert!(semantic.capabilities().contains(SourceCapabilities::AUTHOR_SEARCH));

        // DBLP should only support search
        let dblp = registry.get("dblp").unwrap();
        assert!(dblp.capabilities().contains(SourceCapabilities::SEARCH));
        assert!(!dblp.capabilities().contains(SourceCapabilities::DOWNLOAD));
    }
}
