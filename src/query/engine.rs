// Core query engine

use anyhow::{Context, Result};
use rustdoc_types::{Crate, Function, Id, Impl, Item, ItemEnum, Type};
use serde_json;
use std::collections::HashMap;
use std::fs;

use crate::cache::store::SerializableIndex;
use crate::query::format::TypeFormatter;
use crate::query::lookup::PathResolver;
use crate::types::doc::DocExtractor;
use crate::types::query::*;

#[derive(Debug, Clone)]
pub struct QueryOptions {
    kind: QueryKind,
    include_docs: bool,
    include_private: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryKind {
    Methods,
    Traits,
    Types,
    All,
}

impl QueryOptions {
    /// Create new query options
    pub fn new(kind: QueryKind) -> Self {
        Self {
            kind,
            include_docs: false,
            include_private: false,
        }
    }

    /// Set include_docs flag
    pub fn with_docs(mut self, include_docs: bool) -> Self {
        self.include_docs = include_docs;
        self
    }

    /// Set include_private flag
    pub fn with_private(mut self, include_private: bool) -> Self {
        self.include_private = include_private;
        self
    }
}

pub struct QueryEngine {
    index: SerializableIndex,
    crates: HashMap<String, Crate>,
}

impl QueryEngine {
    /// Create new query engine with index
    pub fn new(index: SerializableIndex) -> Self {
        Self {
            index,
            crates: HashMap::new(),
        }
    }

    /// Create query engine from current cache
    pub fn from_cache() -> Result<Self> {
        use crate::cache::store::CacheStore;

        let store = CacheStore::new()?;
        let index = store.load_current()?.ok_or_else(|| {
            anyhow::anyhow!("No cached index found. Run `cargo doc-query build` first.")
        })?;

        Ok(Self::new(index))
    }

    /// Load a crate's rustdoc JSON into memory
    fn load_crate(&mut self, crate_name: &str, crate_version: &str) -> Result<()> {
        // Check if already loaded
        let key = format!("{}::{}", crate_name, crate_version);
        if self.crates.contains_key(&key) {
            return Ok(());
        }

        // Find the crate node
        let crate_node = self
            .index
            .nodes
            .iter()
            .find(|n| n.name == crate_name && n.version == crate_version)
            .ok_or_else(|| {
                anyhow::anyhow!("Crate {} v{} not found in index", crate_name, crate_version)
            })?;

        // Load rustdoc JSON
        let json_path = &crate_node.json_path;
        let json_str = fs::read_to_string(json_path)
            .with_context(|| format!("Failed to read rustdoc JSON from {}", json_path))?;

        let krate: Crate = serde_json::from_str(&json_str)
            .with_context(|| format!("Failed to parse rustdoc JSON from {}", json_path))?;

        self.crates.insert(key, krate);
        Ok(())
    }

    /// Get a loaded crate
    fn get_crate(&self, crate_name: &str, crate_version: &str) -> Result<&Crate> {
        let key = format!("{}::{}", crate_name, crate_version);
        self.crates
            .get(&key)
            .ok_or_else(|| anyhow::anyhow!("Crate not loaded: {}", key))
    }

    /// Execute a query
    pub fn query(
        &mut self,
        path: &str,
        options: &QueryOptions,
        crate_filter: Option<&str>,
    ) -> Result<QueryResponse> {
        let mut matches = Vec::new();

        // Collect crate names to load first
        let mut crates_to_load: Vec<(String, String)> = Vec::new();
        for crate_node in &self.index.nodes {
            if let Some(filter) = crate_filter {
                if crate_node.name != filter {
                    continue;
                }
            }
            crates_to_load.push((crate_node.name.clone(), crate_node.version.clone()));
        }

        // Load all crates first (to avoid borrowing issues)
        for (name, version) in &crates_to_load {
            self.load_crate(name, version)?;
        }

        // Now execute queries on loaded crates
        for (crate_name, crate_version) in &crates_to_load {
            let krate = self.get_crate(crate_name, crate_version)?;

            let items = PathResolver::find_by_path(krate, path);

            if items.is_empty() {
                continue;
            }

            for (id, item) in items {
                let kind = Self::item_kind(item)?;
                let content = self.extract_content(krate, id, item, &kind, options)?;

                matches.push(QueryMatch::new(
                    crate_name.clone(),
                    crate_version.clone(),
                    self.get_qualified_path(krate, id)?,
                    kind,
                    content,
                ));
            }
        }

        if matches.is_empty() {
            return Err(anyhow::anyhow!("No items found matching path: {}", path));
        }

        let mut response = QueryResponse::new(path.to_string());
        for match_ in matches {
            response.add_match(match_);
        }

        Ok(response)
    }

    /// Get the item kind for output
    fn item_kind(item: &Item) -> Result<String> {
        Ok(match &item.inner {
            ItemEnum::Struct(_) => "type",
            ItemEnum::Enum(_) => "type",
            ItemEnum::Union(_) => "type",
            ItemEnum::Trait(_) => "trait",
            ItemEnum::TraitAlias(_) => "trait",
            ItemEnum::TypeAlias(_) => "type",
            ItemEnum::Function(_) => "function",
            ItemEnum::Module(_) => "module",
            ItemEnum::Impl(_) => "impl",
            ItemEnum::Constant { .. } => "constant",
            ItemEnum::Static(_) => "static",
            _ => "unknown",
        }
        .to_string())
    }

    /// Get fully qualified path for an item
    fn get_qualified_path(&self, krate: &Crate, id: Id) -> Result<String> {
        krate
            .paths
            .get(&id)
            .map(|summary| summary.path.join("::"))
            .ok_or_else(|| anyhow::anyhow!("No path found for item"))
    }

    /// Extract content based on item kind and query options
    fn extract_content(
        &self,
        krate: &Crate,
        id: Id,
        item: &Item,
        kind: &str,
        options: &QueryOptions,
    ) -> Result<QueryContent> {
        match kind {
            "type" => Ok(QueryContent::Type(
                self.extract_type_result(krate, id, item, options)?,
            )),
            "trait" => Ok(QueryContent::Trait(
                self.extract_trait_result(krate, item, options)?,
            )),
            _ => Err(anyhow::anyhow!("Unsupported item kind: {}", kind)),
        }
    }

    /// Extract type query result (methods + trait implementations)
    fn extract_type_result(
        &self,
        krate: &Crate,
        id: Id,
        item: &Item,
        _options: &QueryOptions,
    ) -> Result<TypeResult> {
        // Determine type kind
        let kind_str = match &item.inner {
            ItemEnum::Struct(_) => "struct",
            ItemEnum::Enum(_) => "enum",
            ItemEnum::Union(_) => "union",
            ItemEnum::TypeAlias(_) => "type alias",
            _ => "type",
        };

        // Extract inherent methods (impl without trait)
        let mut methods = Vec::new();
        let mut trait_impls = Vec::new();

        for item_ref in krate.index.values() {
            if let ItemEnum::Impl(impl_block) = &item_ref.inner {
                // Check if this impl is for our type
                if self.impl_is_for_type(impl_block, id) {
                    if let Some(trait_path) = &impl_block.trait_ {
                        // Trait implementation
                        let trait_methods = self.extract_impl_methods(krate, impl_block)?;
                        let mut impl_output =
                            TraitImplOutput::new(trait_path.path.clone(), trait_path.path.clone());
                        trait_methods
                            .into_iter()
                            .for_each(|m| impl_output.add_method(m));
                        trait_impls.push(impl_output);
                    } else {
                        // Inherent impl (methods)
                        let impl_methods = self.extract_impl_methods(krate, impl_block)?;
                        methods.extend(impl_methods);
                    }
                }
            }
        }

        let mut result = TypeResult::new(kind_str.to_string());
        methods.into_iter().for_each(|m| result.add_method(m));
        trait_impls
            .into_iter()
            .for_each(|ti| result.add_trait_impl(ti));

        Ok(result)
    }

    /// Check if impl block is for a specific type
    fn impl_is_for_type(&self, impl_block: &Impl, type_id: Id) -> bool {
        match &impl_block.for_ {
            Type::ResolvedPath(path) => path.id == type_id,
            _ => false,
        }
    }

    /// Extract methods from an impl block
    fn extract_impl_methods(&self, krate: &Crate, impl_block: &Impl) -> Result<Vec<MethodOutput>> {
        let mut methods = Vec::new();

        for item_id in &impl_block.items {
            if let Some(item) = krate.index.get(item_id) {
                if let ItemEnum::Function(func) = &item.inner {
                    methods.push(self.extract_method(item, func)?);
                }
            }
        }

        Ok(methods)
    }

    /// Extract trait query result (definition + methods + associated types)
    fn extract_trait_result(
        &self,
        krate: &Crate,
        item: &Item,
        _options: &QueryOptions,
    ) -> Result<TraitResult> {
        if let ItemEnum::Trait(trait_def) = &item.inner {
            let trait_path = self.get_qualified_path(krate, item.id)?;

            // Extract trait methods
            let mut methods = Vec::new();
            for item_id in &trait_def.items {
                if let Some(func_item) = krate.index.get(item_id) {
                    if let ItemEnum::Function(func) = &func_item.inner {
                        methods.push(self.extract_method(func_item, func)?);
                    }
                }
            }

            // Extract associated types
            let mut associated_types = Vec::new();
            for item_id in &trait_def.items {
                if let Some(type_item) = krate.index.get(item_id) {
                    if let ItemEnum::TypeAlias(ty_alias) = &type_item.inner {
                        let bounds = ty_alias
                            .generics
                            .params
                            .iter()
                            .map(|p| p.name.clone())
                            .collect::<Vec<_>>()
                            .join(", ");

                        let default = Some(TypeFormatter::format_type(&ty_alias.type_));

                        let mut assoc_type =
                            AssociatedTypeOutput::new(type_item.name.clone().unwrap_or_default());
                        if !bounds.is_empty() {
                            assoc_type = assoc_type.with_bounds(Some(bounds));
                        }
                        assoc_type = assoc_type.with_default(default);
                        associated_types.push(assoc_type);
                    }
                }
            }

            let mut result = TraitResult::new(item.name.clone().unwrap_or_default(), trait_path);
            methods.into_iter().for_each(|m| result.add_method(m));
            associated_types
                .into_iter()
                .for_each(|at| result.add_associated_type(at));

            Ok(result)
        } else {
            Err(anyhow::anyhow!("Item is not a trait"))
        }
    }

    /// Extract a single method output
    fn extract_method(&self, item: &Item, func: &Function) -> Result<MethodOutput> {
        let docs = DocExtractor::extract_docs(item);

        let mut method = MethodOutput::new(
            item.name.clone().unwrap_or_default(),
            TypeFormatter::format_signature(&func.sig),
            TypeFormatter::format_return_type(&func.sig.output),
            "public".to_string(),
            true,
        );
        method = method.with_docs(docs);
        method = method.with_is_trait_method(false);

        Ok(method)
    }
}
