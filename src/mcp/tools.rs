//! Tool registry for MCP tools.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use crate::sources::SourceRegistry;

/// An MCP tool that can be called by the client
#[derive(Clone)]
pub struct Tool {
    /// Tool name (e.g., "search_arxiv")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// JSON Schema for input parameters
    pub input_schema: serde_json::Value,

    /// Handler function to execute the tool
    pub handler: Arc<dyn ToolHandler>,
}

impl std::fmt::Debug for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("input_schema", &self.input_schema)
            .finish()
    }
}

/// Handler for executing a tool
#[async_trait::async_trait]
pub trait ToolHandler: Send + Sync + std::fmt::Debug {
    /// Execute the tool with the given arguments
    async fn execute(&self, args: Value) -> Result<Value, String>;
}

/// Registry for all MCP tools
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    /// Create a new tool registry and register all tools from the source registry
    pub fn from_sources(sources: &SourceRegistry) -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // Register tools for each source
        for source in sources.all() {
            registry.register_source_tools(source);
        }

        // Register utility tools
        registry.register_utility_tools();

        registry
    }

    /// Register all tools for a specific source
    fn register_source_tools(&mut self, source: &Arc<dyn crate::sources::Source>) {
        let source_id = source.id();

        // Search tool (if supported)
        if source.supports_search() {
            self.register(Tool {
                name: format!("search_{}", source_id),
                description: format!("Search {} for papers", source.name()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "Maximum number of results",
                            "default": 10
                        },
                        "year": {
                            "type": "string",
                            "description": "Year filter (e.g., '2020', '2018-2022', '2010-', '-2015')"
                        },
                        "category": {
                            "type": "string",
                            "description": "Category/subject filter"
                        }
                    },
                    "required": ["query"]
                }),
                handler: Arc::new(SearchToolHandler {
                    source: source.clone(),
                }),
            });
        }

        // Download tool (if supported)
        if source.supports_download() {
            self.register(Tool {
                name: format!("download_{}", source_id),
                description: format!("Download a paper from {}", source.name()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "paper_id": {
                            "type": "string",
                            "description": "Paper ID"
                        },
                        "save_path": {
                            "type": "string",
                            "description": "Directory to save the PDF",
                            "default": "./downloads"
                        }
                    },
                    "required": ["paper_id"]
                }),
                handler: Arc::new(DownloadToolHandler {
                    source: source.clone(),
                }),
            });
        }

        // Read tool (if supported)
        if source.supports_read() {
            self.register(Tool {
                name: format!("read_{}_paper", source_id),
                description: format!("Read and extract text from a {} paper", source.name()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "paper_id": {
                            "type": "string",
                            "description": "Paper ID"
                        },
                        "save_path": {
                            "type": "string",
                            "description": "Directory for downloaded PDFs",
                            "default": "./downloads"
                        }
                    },
                    "required": ["paper_id"]
                }),
                handler: Arc::new(ReadToolHandler {
                    source: source.clone(),
                }),
            });
        }

        // Citation tools (if supported)
        if source.supports_citations() {
            for (tool_suffix, desc) in [
                ("citations", "papers that cite this paper"),
                ("references", "papers referenced by this paper"),
                ("related", "related papers"),
            ] {
                self.register(Tool {
                    name: format!("get_{}_{}", source_id, tool_suffix),
                    description: format!("Get {} from {}", desc, source.name()),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "paper_id": {
                                "type": "string",
                                "description": "Paper ID"
                            },
                            "max_results": {
                                "type": "integer",
                                "description": "Maximum number of results",
                                "default": 20
                            }
                        },
                        "required": ["paper_id"]
                    }),
                    handler: Arc::new(CitationToolHandler {
                        source: source.clone(),
                        citation_type: tool_suffix.to_string(),
                    }),
                });
            }
        }

        // Author search (if supported)
        if source.supports_author_search() {
            self.register(Tool {
                name: format!("search_{}_by_author", source_id),
                description: format!("Search {} by author name", source.name()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "author_name": {
                            "type": "string",
                            "description": "Author name"
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "Maximum number of results",
                            "default": 20
                        }
                    },
                    "required": ["author_name"]
                }),
                handler: Arc::new(AuthorSearchToolHandler {
                    source: source.clone(),
                }),
            });
        }

        // DOI lookup (if supported)
        if source.supports_doi_lookup() {
            self.register(Tool {
                name: format!("get_{}_by_doi", source_id),
                description: format!("Get a paper from {} by DOI", source.name()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "doi": {
                            "type": "string",
                            "description": "Digital Object Identifier"
                        }
                    },
                    "required": ["doi"]
                }),
                handler: Arc::new(DoiLookupHandler {
                    source: source.clone(),
                }),
            });
        }
    }

    /// Register utility tools (deduplication, etc.)
    fn register_utility_tools(&mut self) {
        self.register(Tool {
            name: "deduplicate_papers".to_string(),
            description: "Remove duplicate papers from a list".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "papers": {
                        "type": "array",
                        "description": "Array of paper objects"
                    },
                    "keep": {
                        "type": "string",
                        "description": "Which papers to keep ('first' or 'last')",
                        "default": "first"
                    }
                },
                "required": ["papers"]
            }),
            handler: Arc::new(DeduplicateToolHandler),
        });
    }

    /// Register a tool
    pub fn register(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Get all tools
    pub fn all(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    /// Execute a tool by name
    pub async fn execute(&self, name: &str, args: Value) -> Result<Value, String> {
        let tool = self
            .get(name)
            .ok_or_else(|| format!("Tool '{}' not found", name))?;

        tool.handler.execute(args).await
    }
}

// ===== Tool Handlers =====

#[derive(Debug)]
struct SearchToolHandler {
    source: Arc<dyn crate::sources::Source>,
}

#[async_trait::async_trait]
impl ToolHandler for SearchToolHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'query' parameter")?;

        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let year = args.get("year").and_then(|v| v.as_str()).map(|s| s.to_string());

        let category = args
            .get("category")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut search_query = crate::models::SearchQuery::new(query).max_results(max_results);

        if let Some(year) = year {
            search_query = search_query.year(year);
        }
        if let Some(cat) = category {
            search_query = search_query.category(cat);
        }

        let response = self.source.search(&search_query).await.map_err(|e| e.to_string())?;

        serde_json::to_value(response).map_err(|e| e.to_string())
    }
}

#[derive(Debug)]
struct DownloadToolHandler {
    source: Arc<dyn crate::sources::Source>,
}

#[async_trait::async_trait]
impl ToolHandler for DownloadToolHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let paper_id = args
            .get("paper_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'paper_id' parameter")?;

        let save_path = args
            .get("save_path")
            .and_then(|v| v.as_str())
            .unwrap_or("./downloads");

        let request = crate::models::DownloadRequest::new(paper_id, save_path);

        let result = self.source.download(&request).await.map_err(|e| e.to_string())?;

        serde_json::to_value(result).map_err(|e| e.to_string())
    }
}

#[derive(Debug)]
struct ReadToolHandler {
    source: Arc<dyn crate::sources::Source>,
}

#[async_trait::async_trait]
impl ToolHandler for ReadToolHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let paper_id = args
            .get("paper_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'paper_id' parameter")?;

        let save_path = args
            .get("save_path")
            .and_then(|v| v.as_str())
            .unwrap_or("./downloads");

        let request = crate::models::ReadRequest::new(paper_id, save_path);

        let result = self.source.read(&request).await.map_err(|e| e.to_string())?;

        serde_json::to_value(result).map_err(|e| e.to_string())
    }
}

#[derive(Debug)]
struct CitationToolHandler {
    source: Arc<dyn crate::sources::Source>,
    citation_type: String,
}

#[async_trait::async_trait]
impl ToolHandler for CitationToolHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let paper_id = args
            .get("paper_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'paper_id' parameter")?;

        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize;

        let request = crate::models::CitationRequest::new(paper_id).max_results(max_results);

        let response = match self.citation_type.as_str() {
            "citations" => self.source.get_citations(&request).await,
            "references" => self.source.get_references(&request).await,
            "related" => self.source.get_related(&request).await,
            _ => Err(crate::sources::SourceError::InvalidRequest(
                "Unknown citation type".to_string(),
            )),
        }
        .map_err(|e| e.to_string())?;

        serde_json::to_value(response).map_err(|e| e.to_string())
    }
}

#[derive(Debug)]
struct AuthorSearchToolHandler {
    source: Arc<dyn crate::sources::Source>,
}

#[async_trait::async_trait]
impl ToolHandler for AuthorSearchToolHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let author_name = args
            .get("author_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'author_name' parameter")?;

        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize;

        let response = self
            .source
            .search_by_author(author_name, max_results)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_value(response).map_err(|e| e.to_string())
    }
}

#[derive(Debug)]
struct DoiLookupHandler {
    source: Arc<dyn crate::sources::Source>,
}

#[async_trait::async_trait]
impl ToolHandler for DoiLookupHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let doi = args
            .get("doi")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'doi' parameter")?;

        let paper = self.source.get_by_doi(doi).await.map_err(|e| e.to_string())?;

        serde_json::to_value(paper).map_err(|e| e.to_string())
    }
}

#[derive(Debug)]
struct DeduplicateToolHandler;

#[async_trait::async_trait]
impl ToolHandler for DeduplicateToolHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let papers: Vec<crate::models::Paper> = serde_json::from_value(
            args.get("papers")
                .ok_or("Missing 'papers' parameter")?
                .clone(),
        )
        .map_err(|e| format!("Invalid papers array: {}", e))?;

        let keep = args
            .get("keep")
            .and_then(|v| v.as_str())
            .unwrap_or("first");

        let strategy = match keep {
            "last" => crate::utils::DuplicateStrategy::Last,
            _ => crate::utils::DuplicateStrategy::First,
        };

        let deduped = crate::utils::deduplicate_papers(papers, strategy);

        serde_json::to_value(deduped).map_err(|e| e.to_string())
    }
}
