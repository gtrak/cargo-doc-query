// Recursive type expansion with cycle detection and token budgeting

use rustdoc_types::{Crate, Id, Item, ItemEnum, Type};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;

use crate::cache::store::SerializableIndex;
use crate::parser::serde_helper::deserialize_with_stack;
use crate::query::lookup::PathResolver;
use crate::types::detail::DetailLevel;
use crate::types::detail::{
    extract_deprecation_info, extract_function_modifiers, extract_semantic_attrs, format_generics,
    visibility_to_string,
};
use crate::types::expand::{ExpansionResult, FieldInfo, TokenConfig, TypeGraph, TypeNode};

/// Errors that can occur during type expansion
#[derive(Error, Debug)]
pub enum ExpandError {
    #[error("No cached index found. Run `cargo doc-query build` first.")]
    NoCache,

    #[error("No items found matching path: {0}")]
    NotFound(String),

    #[error("Expansion failed: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, ExpandError>;

pub struct TypeExpander {
    index: SerializableIndex,
    crates: HashMap<String, Crate>,
    visited: HashSet<Id>,
    current_depth: u32,
    depth_limit: u32,
    token_config: TokenConfig,
    current_token_count: usize,
    truncated: Vec<String>,
    detail_level: DetailLevel,
}

impl TypeExpander {
    /// Create new expander with default config
    pub fn new(index: SerializableIndex, depth_limit: u32) -> Self {
        Self::with_config(
            index,
            depth_limit,
            TokenConfig::default(),
            DetailLevel::Standard,
        )
    }

    /// Create new expander with custom config
    pub fn with_config(
        index: SerializableIndex,
        depth_limit: u32,
        config: TokenConfig,
        detail_level: DetailLevel,
    ) -> Self {
        Self {
            index,
            crates: HashMap::new(),
            visited: HashSet::new(),
            current_depth: 0,
            depth_limit,
            token_config: config,
            current_token_count: 0,
            truncated: Vec::new(),
            detail_level,
        }
    }

    /// Load a crate's rustdoc JSON into memory
    fn load_crate(&mut self, crate_name: &str, crate_version: &str) -> Result<()> {
        use std::fs;

        let key = format!("{}::{}", crate_name, crate_version);
        if self.crates.contains_key(&key) {
            return Ok(());
        }

        let crate_node = self
            .index
            .nodes
            .iter()
            .find(|n| n.name == crate_name && n.version == crate_version)
            .ok_or_else(|| {
                anyhow::anyhow!("Crate {} v{} not found in index", crate_name, crate_version)
            })?;

        let json_path = PathBuf::from(&crate_node.json_path);
        let json_path = if json_path.is_absolute() {
            json_path
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(&json_path)
        };

        let json_str = fs::read_to_string(&json_path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to read rustdoc JSON from {}: {}",
                json_path.display(),
                e
            )
        })?;

        let krate: Crate = deserialize_with_stack(&json_str).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse rustdoc JSON from {}: {}",
                json_path.display(),
                e
            )
        })?;

        self.crates.insert(key, krate);
        Ok(())
    }

    /// Check if adding more tokens would exceed budget
    fn would_exceed_budget(&self, additional_tokens: usize) -> bool {
        match self.token_config.budget {
            None => false,
            Some(budget) => self.current_token_count + additional_tokens > budget,
        }
    }

    /// Check if approaching budget warning threshold
    fn is_approaching_budget(&self) -> bool {
        match self.token_config.budget {
            None => false,
            Some(budget) => {
                let threshold = (budget as f32 * self.token_config.warning_threshold) as usize;
                self.current_token_count >= threshold
            }
        }
    }

    /// Add tokens to current count
    fn add_tokens(&mut self, count: usize) {
        self.current_token_count += count;
    }

    pub fn expand(&mut self, path: &str, crate_filter: Option<&str>) -> Result<ExpansionResult> {
        let mut graph = TypeGraph::new(path.to_string(), self.depth_limit);

        // Collect crate names to search (sorted for deterministic order)
        let mut crates_to_search: Vec<(String, String)> = self
            .index
            .nodes
            .iter()
            .filter(|n| crate_filter.map_or(true, |f| n.name == f))
            .map(|n| (n.name.clone(), n.version.clone()))
            .collect();
        crates_to_search.sort_by(|a, b| a.0.cmp(&b.0));

        // Try each crate one at a time, stop when found
        for (name, version) in &crates_to_search {
            self.load_crate(name, version)?;

            // Check if this crate has the type
            let key = format!("{}::{}", name, version);
            let items: Vec<(String, Id, Item)> = {
                let krate = self.crates.get(&key).unwrap();
                PathResolver::find_by_path(krate, path)
                    .into_iter()
                    .map(|(id, item)| (key.clone(), id.clone(), item.clone()))
                    .collect()
            };

            if items.is_empty() {
                // Remove from cache to free memory
                self.crates.remove(&key);
                continue;
            }

            // Found it - expand
            for (crate_name, id, item) in items {
                self.visited.clear();
                self.current_depth = 0;

                match &item.inner {
                    ItemEnum::Struct(_)
                    | ItemEnum::Enum(_)
                    | ItemEnum::Union(_)
                    | ItemEnum::TypeAlias(_)
                    | ItemEnum::Trait(_) => {
                        if let Some(node) = self.expand_item(&crate_name, &id, 0)? {
                            // Update token count for the node
                            self.add_tokens(node.estimate_tokens());
                            graph.add_node(node);
                        }
                    }
                    ItemEnum::Module(_) => {
                        // Expand module contents
                        self.expand_module(&crate_name, &id, 0, &mut graph)?;
                    }
                    ItemEnum::Function(func) => {
                        // Handle functions - show as a node with signature
                        // crate_name is actually the key (name::version)
                        let krate = self.crates.get(&crate_name).unwrap();
                        let type_path = self.get_path(krate, id);
                        let visibility = visibility_to_string(&item.visibility);
                        let mut node = TypeNode::with_crate_visibility(
                            type_path.clone(),
                            "function".to_string(),
                            0,
                            self.extract_crate_name(&crate_name, &type_path),
                            visibility,
                        );

                        // Extract function signature from function item
                        let signature = format_function_signature(&item, func);

                        // Add as a module item so it shows in output
                        let module_item = crate::types::expand::ModuleItemInfo::new(
                            item.name.clone().unwrap_or_default(),
                            "function".to_string(),
                            type_path,
                        )
                        .with_signature(signature);
                        node.add_item(module_item);

                        self.add_tokens(node.estimate_tokens());
                        graph.add_node(node);
                    }
                    _ => {
                        // Skip items that aren't types, modules, or functions
                        continue;
                    }
                }
            }

            // Only expand first crate that has the type
            break;
        }

        if graph.nodes.is_empty() {
            return Err(ExpandError::NotFound(path.to_string()));
        }

        // Convert to minimal if requested
        let graph = if self.token_config.minimal_mode {
            graph.to_minimal()
        } else {
            graph
        };

        let mut result = ExpansionResult::new(graph);

        if !self.truncated.is_empty() {
            result = result.with_truncation(self.truncated.clone());
        }

        Ok(result)
    }

    fn expand_item(
        &mut self,
        crate_name: &str,
        item_id: &Id,
        depth: u32,
    ) -> Result<Option<TypeNode>> {
        if self.visited.contains(item_id) {
            return Ok(None);
        }

        self.visited.insert(*item_id);

        if depth >= self.depth_limit {
            return Ok(None);
        }

        // Check budget before expanding
        if let Some(budget) = self.token_config.budget {
            if self.current_token_count >= budget {
                // Budget exceeded - record truncation and skip
                let path = format!("{:?}", item_id);
                if !self.truncated.contains(&path) {
                    self.truncated.push(path);
                }
                return Ok(None);
            }
        }

        // Find the item in the SPECIFIC crate (not all crates - IDs are not globally unique!)
        let krate = self
            .crates
            .get(crate_name)
            .ok_or_else(|| anyhow::anyhow!("Crate {} not found", crate_name))?;
        let item = krate
            .index
            .get(item_id)
            .ok_or_else(|| anyhow::anyhow!("Item {:?} not found in crate {}", item_id, crate_name))?
            .clone();

        let type_path = self.get_path(krate, *item_id);

        let kind = match &item.inner {
            ItemEnum::Struct(_) => "struct",
            ItemEnum::Enum(_) => "enum",
            ItemEnum::Union(_) => "union",
            ItemEnum::TypeAlias(_) => "type alias",
            _ => "type",
        };

        // Extract crate name from path for filtering
        let crate_name = self.extract_crate_name(&crate_name, &type_path);
        // Get visibility from item
        let visibility = visibility_to_string(&item.visibility);

        let mut node = TypeNode::with_crate_visibility(
            type_path.clone(),
            kind.to_string(),
            depth,
            crate_name,
            visibility,
        );

        // Populate metadata based on DetailLevel
        let detail_level = self.detail_level;

        // FIELD-03: Generic parameters (Standard and Detailed)
        if detail_level.includes_generics() {
            if let Some(generics) = Self::extract_generics_from_item(&item) {
                if let Some(generics_str) = format_generics(&generics) {
                    if !generics_str.is_empty() {
                        node.add_generic_param(generics_str);
                    }
                }
            }
        }

        // FIELD-02, FIELD-04: Deprecation and Attributes (Detailed only)
        if detail_level.includes_deprecation() {
            if let Some((_, note)) = extract_deprecation_info(item.deprecation.as_ref()) {
                node = node.with_deprecation(true, note);
            }
        }

        if detail_level.includes_attributes() {
            let attrs = extract_semantic_attrs(&item.attrs);
            if !attrs.is_empty() {
                node = node.with_attributes(attrs);
            }
        }

        // Extract fields/variants based on type kind
        match &item.inner {
            ItemEnum::Struct(struct_type) => {
                use rustdoc_types::StructKind;
                match &struct_type.kind {
                    StructKind::Plain { fields, .. } => {
                        for field_id in fields {
                            if let Some(field_item) = krate.index.get(field_id) {
                                if let ItemEnum::StructField(field_type) = &field_item.inner {
                                    let field_name = field_item.name.clone().unwrap_or_default();
                                    let type_str = self.format_type(field_type);
                                    let field_info = FieldInfo::new(field_name, type_str, false);
                                    node.add_field(field_info);
                                }
                            }
                        }
                    }
                    StructKind::Unit => {
                        // Unit structs have no fields
                    }
                    StructKind::Tuple(fields) => {
                        for (i, field_id_opt) in fields.iter().enumerate() {
                            if let Some(field_id) = field_id_opt {
                                if let Some(field_item) = krate.index.get(field_id) {
                                    if let ItemEnum::StructField(field_type) = &field_item.inner {
                                        let type_str = self.format_type(field_type);
                                        let field_info =
                                            FieldInfo::new(format!("{}", i), type_str, false);
                                        node.add_field(field_info);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ItemEnum::Enum(enum_type) => {
                for variant_id in &enum_type.variants {
                    if let Some(variant_item) = krate.index.get(variant_id) {
                        if let ItemEnum::Variant(variant_data) = &variant_item.inner {
                            let variant_name = variant_item.name.clone().unwrap_or_default();
                            let mut variant_info =
                                crate::types::expand::VariantInfo::new(variant_name);

                            // Extract variant fields
                            use rustdoc_types::VariantKind;
                            match &variant_data.kind {
                                VariantKind::Plain => {}
                                VariantKind::Tuple(fields) => {
                                    for (i, field_id_opt) in fields.iter().enumerate() {
                                        if let Some(field_id) = field_id_opt {
                                            if let Some(field_item) = krate.index.get(field_id) {
                                                if let ItemEnum::StructField(field_type) =
                                                    &field_item.inner
                                                {
                                                    let type_str = self.format_type(field_type);
                                                    let field_info = FieldInfo::new(
                                                        format!("{}", i),
                                                        type_str,
                                                        false,
                                                    );
                                                    variant_info.add_field(field_info);
                                                }
                                            }
                                        }
                                    }
                                }
                                VariantKind::Struct { fields, .. } => {
                                    for field_id in fields {
                                        if let Some(field_item) = krate.index.get(field_id) {
                                            if let ItemEnum::StructField(field_type) =
                                                &field_item.inner
                                            {
                                                let field_name =
                                                    field_item.name.clone().unwrap_or_default();
                                                let type_str = self.format_type(field_type);
                                                let field_info =
                                                    FieldInfo::new(field_name, type_str, false);
                                                variant_info.add_field(field_info);
                                            }
                                        }
                                    }
                                }
                            }

                            node.add_variant(variant_info);
                        }
                    }
                }
            }
            ItemEnum::TypeAlias(type_alias) => {
                // Type alias - show what it points to
                let type_str = self.format_type(&type_alias.type_);
                node.add_generic_param(format!("= {}", type_str));
            }
            _ => {}
        }

        Ok(Some(node))
    }

    /// Expand a module and its contents
    fn expand_module(
        &mut self,
        crate_name: &str,
        module_id: &Id,
        depth: u32,
        graph: &mut TypeGraph,
    ) -> Result<()> {
        if self.visited.contains(module_id) {
            return Ok(());
        }

        self.visited.insert(*module_id);

        if depth >= self.depth_limit {
            return Ok(());
        }

        // Find the module in the SPECIFIC crate (not all crates - IDs are not globally unique!)
        let krate = self
            .crates
            .get(crate_name)
            .ok_or_else(|| anyhow::anyhow!("Crate {} not found", crate_name))?;
        let module_item = krate
            .index
            .get(module_id)
            .ok_or_else(|| {
                anyhow::anyhow!("Module {:?} not found in crate {}", module_id, crate_name)
            })?
            .clone();

        // Clone crate data to avoid borrow issues
        let krate = krate.clone();

        let module_path = self.get_path(&krate, *module_id);
        let mut node = TypeNode::with_crate_visibility(
            module_path.clone(),
            "module".to_string(),
            depth,
            self.extract_crate_name(crate_name, &module_path),
            visibility_to_string(&module_item.visibility),
        );
        let mut submodules_to_expand: Vec<Id> = Vec::new();

        // Get module items
        if let ItemEnum::Module(module) = &module_item.inner {
            for item_id in &module.items {
                if let Some(item) = krate.index.get(item_id) {
                    let item_path = self.get_path(&krate, *item_id);
                    let name = item.name.clone().unwrap_or_default();

                    // Handle Use items specially (re-exports)
                    if let ItemEnum::Use(use_data) = &item.inner {
                        // Use items have source path and optional local name
                        let use_name = if use_data.name.is_empty() {
                            // Extract name from end of source path
                            use_data.source.split("::").last().unwrap_or("").to_string()
                        } else {
                            use_data.name.clone()
                        };

                        let module_item = crate::types::expand::ModuleItemInfo::new(
                            use_name,
                            "re-export".to_string(),
                            use_data.source.clone(),
                        );
                        node.add_item(module_item);
                        continue;
                    }

                    // Determine item kind
                    let kind = match &item.inner {
                        ItemEnum::Struct(_) => "struct",
                        ItemEnum::Enum(_) => "enum",
                        ItemEnum::Union(_) => "union",
                        ItemEnum::TypeAlias(_) => "type",
                        ItemEnum::Trait(_) => "trait",
                        ItemEnum::Function(_) => "function",
                        ItemEnum::Module(_) => "module",
                        ItemEnum::Constant { .. } => "const",
                        ItemEnum::Static(_) => "static",
                        ItemEnum::Macro(_) => "macro",
                        _ => "other",
                    };

                    if kind == "module" {
                        // Collect submodule IDs for later expansion
                        if depth + 1 < self.depth_limit {
                            submodules_to_expand.push(*item_id);
                        }
                    } else {
                        // Add as module item (not a field!)
                        let mut module_item = crate::types::expand::ModuleItemInfo::new(
                            name,
                            kind.to_string(),
                            item_path,
                        );
                        // Extract signature for functions
                        if kind == "function" {
                            if let ItemEnum::Function(func) = &item.inner {
                                let signature = format_function_signature(item, func);
                                module_item = module_item.with_signature(signature);
                            }
                        }
                        node.add_item(module_item);
                    }
                }
            }
        }

        self.add_tokens(node.estimate_tokens());
        graph.add_node(node);

        // Now expand submodules (after releasing borrows)
        // Submodules are in the SAME crate as the parent
        for submodule_id in submodules_to_expand {
            self.expand_module(crate_name, &submodule_id, depth + 1, graph)?;
        }

        Ok(())
    }

    fn format_type(&self, ty: &Type) -> String {
        match ty {
            Type::ResolvedPath(path) => path.path.clone(),
            Type::Primitive(p) => p.clone(),
            Type::Generic(g) => g.clone(),
            Type::Slice(inner) => format!("[{}]", self.format_type(inner)),
            Type::Array { type_, len } => format!("[{}; {}]", self.format_type(type_), len),
            Type::Tuple(types) => {
                let parts: Vec<String> = types.iter().map(|t| self.format_type(t)).collect();
                format!("({})", parts.join(", "))
            }
            Type::RawPointer { type_, is_mutable } => {
                let mut_str = if *is_mutable { "mut " } else { "const " };
                format!("*{}{}", mut_str, self.format_type(type_))
            }
            Type::BorrowedRef { type_, .. } => self.format_type(type_),
            Type::ImplTrait(bounds) => {
                if bounds.is_empty() {
                    "impl <trait>".to_string()
                } else {
                    let parts: Vec<String> = bounds.iter().map(|b| format!("{:?}", b)).collect();
                    format!("impl {}", parts.join(" + "))
                }
            }
            Type::DynTrait(dyn_trait) => {
                let mut parts = Vec::new();
                for trait_bound in &dyn_trait.traits {
                    parts.push(trait_bound.trait_.path.clone());
                }
                if let Some(lifetime) = &dyn_trait.lifetime {
                    parts.push(lifetime.clone());
                }
                format!("dyn {}", parts.join(" + "))
            }
            Type::Infer => "_".to_string(),
            _ => "unknown".to_string(),
        }
    }

    fn get_path(&self, krate: &Crate, id: Id) -> String {
        krate
            .paths
            .get(&id)
            .map(|summary| summary.path.join("::"))
            .unwrap_or_else(|| format!("{:?}", id))
    }

    /// Extract crate name from path (first segment)
    fn extract_crate_name(&self, crate_name: &str, path: &str) -> String {
        // Use the crate_name from the search if it's specific
        if crate_name.starts_with(&format!("{}::", crate_name)) {
            crate_name.to_string()
        } else {
            // Extract from path
            path.split("::").next().unwrap_or(crate_name).to_string()
        }
    }

    /// Extract generics from an item
    fn extract_generics_from_item(item: &Item) -> Option<rustdoc_types::Generics> {
        match &item.inner {
            ItemEnum::Struct(s) => Some(s.generics.clone()),
            ItemEnum::Enum(e) => Some(e.generics.clone()),
            ItemEnum::Function(f) => Some(f.generics.clone()),
            ItemEnum::Trait(t) => Some(t.generics.clone()),
            ItemEnum::TypeAlias(t) => Some(t.generics.clone()),
            ItemEnum::Union(u) => Some(u.generics.clone()),
            _ => None,
        }
    }
}

/// Format a function's signature from its rustdoc item (without the function name)
fn format_function_signature(item: &Item, func: &rustdoc_types::Function) -> String {
    let mut sig_parts = Vec::new();

    // Generic parameters
    if !func.generics.params.is_empty() {
        let params: Vec<String> = func
            .generics
            .params
            .iter()
            .map(|p| p.name.clone())
            .collect();
        sig_parts.push(format!("<{}>", params.join(", ")));
    }

    // Parameters from FunctionSignature
    let params: Vec<String> = func
        .sig
        .inputs
        .iter()
        .map(|(name, ty)| format!("{}: {}", name, format_type_signature(ty)))
        .collect();
    sig_parts.push(format!("({})", params.join(", ")));

    // Return type
    if let Some(ret) = &func.sig.output {
        let ret_str = match ret {
            rustdoc_types::Type::ResolvedPath(path) => path.path.clone(),
            rustdoc_types::Type::Primitive(p) => p.clone(),
            rustdoc_types::Type::Tuple(types) => {
                let inner: Vec<String> = types.iter().map(|t| format_type_signature(t)).collect();
                format!("({})", inner.join(", "))
            }
            _ => format!("{:?}", ret),
        };
        sig_parts.push(format!("-> {}", ret_str));
    }

    sig_parts.join(" ")
}

fn format_type_signature(ty: &rustdoc_types::Type) -> String {
    match ty {
        rustdoc_types::Type::ResolvedPath(path) => path.path.clone(),
        rustdoc_types::Type::Primitive(p) => p.clone(),
        rustdoc_types::Type::Generic(g) => g.clone(),
        rustdoc_types::Type::Slice(inner) => format!("[{}]", format_type_signature(inner)),
        rustdoc_types::Type::Array { type_, len } => {
            format!("[{}; {}]", format_type_signature(type_), len)
        }
        rustdoc_types::Type::Tuple(types) => {
            let parts: Vec<String> = types.iter().map(|t| format_type_signature(t)).collect();
            format!("({})", parts.join(", "))
        }
        rustdoc_types::Type::RawPointer { type_, is_mutable } => {
            let mut_str = if *is_mutable { "mut " } else { "const " };
            format!("*{}{}", mut_str, format_type_signature(type_))
        }
        rustdoc_types::Type::BorrowedRef { type_, .. } => format_type_signature(type_),
        _ => format!("{:?}", ty),
    }
}

pub fn expand_type(
    path: &str,
    depth: u32,
    crate_filter: Option<&str>,
    detail_level: DetailLevel,
) -> Result<ExpansionResult> {
    let index = crate::cache::store::CacheStore::new()
        .map_err(|e| ExpandError::Other(e.into()))?
        .load_current()
        .map_err(|e| ExpandError::Other(e.into()))?
        .ok_or(ExpandError::NoCache)?;

    let mut expander = TypeExpander::new(index, depth);
    expander.detail_level = detail_level;
    expander.expand(path, crate_filter)
}

pub fn expand_type_with_config(
    path: &str,
    depth: u32,
    crate_filter: Option<&str>,
    config: TokenConfig,
    detail_level: DetailLevel,
) -> Result<ExpansionResult> {
    let index = crate::cache::store::CacheStore::new()
        .map_err(|e| ExpandError::Other(e.into()))?
        .load_current()
        .map_err(|e| ExpandError::Other(e.into()))?
        .ok_or(ExpandError::NoCache)?;

    let mut expander = TypeExpander::with_config(index, depth, config, detail_level);
    expander.expand(path, crate_filter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustdoc_types::*;

    #[test]
    fn test_token_config_default() {
        let config = TokenConfig::default();
        assert_eq!(config.budget, None);
        assert!(!config.minimal_mode);
        assert_eq!(config.warning_threshold, 0.8);
    }

    #[test]
    fn test_token_config_with_budget() {
        let config = TokenConfig::new().with_budget(Some(1000));
        assert_eq!(config.budget, Some(1000));
    }

    #[test]
    fn test_token_config_minimal() {
        let config = TokenConfig::new().with_minimal(true);
        assert!(config.minimal_mode);
    }

    #[test]
    fn test_token_config_threshold() {
        let config = TokenConfig::new().with_threshold(0.5);
        assert_eq!(config.warning_threshold, 0.5);
    }

    #[test]
    fn test_would_exceed_budget_unlimited() {
        let config = TokenConfig::default();
        let expander = TypeExpander::new(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10, // depth_limit
        );

        assert!(!expander.would_exceed_budget(100));
    }

    #[test]
    fn test_would_exceed_budget_with_budget() {
        let config = TokenConfig::new().with_budget(Some(10));
        let mut expander = TypeExpander::with_config(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10, // depth_limit
            config,
            DetailLevel::Standard,
        );

        assert!(!expander.would_exceed_budget(5));
        expander.add_tokens(5);
        assert!(!expander.would_exceed_budget(5));
        assert!(!expander.would_exceed_budget(1));
    }

    #[test]
    fn test_would_exceed_budget_exceeds() {
        let config = TokenConfig::new().with_budget(Some(10));
        let mut expander = TypeExpander::with_config(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10, // depth_limit
            config,
            DetailLevel::Standard,
        );

        expander.add_tokens(10);
        assert!(expander.would_exceed_budget(1));
    }

    #[test]
    fn test_is_approaching_budget() {
        let config = TokenConfig::new().with_budget(Some(100));
        let mut expander = TypeExpander::with_config(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10, // depth_limit
            config,
            DetailLevel::Standard,
        );

        // At default threshold of 0.8 (80 tokens), should not be approaching
        assert!(!expander.is_approaching_budget());

        expander.add_tokens(80);
        assert!(expander.is_approaching_budget());

        expander.add_tokens(100);
        assert!(expander.is_approaching_budget());
    }

    #[test]
    fn test_is_approaching_budget_custom_threshold() {
        let config = TokenConfig::new()
            .with_budget(Some(100))
            .with_threshold(0.5);
        let mut expander = TypeExpander::with_config(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10, // depth_limit
            config,
            DetailLevel::Standard,
        );

        // At 0.5 threshold (50 tokens), should approach at 50
        assert!(!expander.is_approaching_budget());

        expander.add_tokens(50);
        assert!(expander.is_approaching_budget());
    }

    #[test]
    fn test_format_type_resolved_path() {
        let expander = TypeExpander::new(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10,
        );
        let ty = Type::ResolvedPath(Path {
            path: "std::collections::HashMap".to_string(),
            id: Id(1),
            args: None,
        });
        assert_eq!(expander.format_type(&ty), "std::collections::HashMap");
    }

    #[test]
    fn test_format_type_primitive() {
        let expander = TypeExpander::new(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10,
        );
        let ty = Type::Primitive("u32".to_string());
        assert_eq!(expander.format_type(&ty), "u32");
    }

    #[test]
    fn test_format_type_generic() {
        let expander = TypeExpander::new(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10,
        );
        let ty = Type::Generic("T".to_string());
        assert_eq!(expander.format_type(&ty), "T");
    }

    #[test]
    fn test_format_type_slice() {
        let expander = TypeExpander::new(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10,
        );
        let ty = Type::Slice(Box::new(Type::Generic("T".to_string())));
        assert_eq!(expander.format_type(&ty), "[T]");
    }

    #[test]
    fn test_format_type_array() {
        let expander = TypeExpander::new(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10,
        );
        let ty = Type::Array {
            type_: Box::new(Type::Generic("T".to_string())),
            len: "10".to_string(),
        };
        assert_eq!(expander.format_type(&ty), "[T; 10]");
    }

    #[test]
    fn test_format_type_tuple() {
        let expander = TypeExpander::new(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10,
        );
        let ty = Type::Tuple(vec![
            Type::Generic("A".to_string()),
            Type::Generic("B".to_string()),
        ]);
        assert_eq!(expander.format_type(&ty), "(A, B)");
    }

    #[test]
    fn test_format_type_raw_pointer() {
        let expander = TypeExpander::new(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10,
        );
        let ty = Type::RawPointer {
            is_mutable: true,
            type_: Box::new(Type::Generic("T".to_string())),
        };
        assert_eq!(expander.format_type(&ty), "*mut T");
    }

    #[test]
    fn test_format_type_borrowed_ref() {
        let expander = TypeExpander::new(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10,
        );
        let ty = Type::BorrowedRef {
            lifetime: Some("'a".to_string()),
            is_mutable: false,
            type_: Box::new(Type::Generic("T".to_string())),
        };
        assert_eq!(expander.format_type(&ty), "T");
    }

    #[test]
    fn test_format_type_dyn_trait() {
        let expander = TypeExpander::new(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10,
        );
        let ty = Type::DynTrait(DynTrait {
            lifetime: None,
            traits: vec![PolyTrait {
                trait_: Path {
                    path: "Display".to_string(),
                    id: Id(1),
                    args: None,
                },
                generic_params: vec![],
            }],
        });
        assert_eq!(expander.format_type(&ty), "dyn Display");
    }

    #[test]
    fn test_format_type_infer() {
        let expander = TypeExpander::new(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10,
        );
        let ty = Type::Infer;
        assert_eq!(expander.format_type(&ty), "_");
    }

    #[test]
    fn test_format_type_unknown() {
        let expander = TypeExpander::new(
            crate::cache::store::SerializableIndex {
                format_version: 1,
                cache_key: "test".to_string(),
                nodes: vec![],
                edges: vec![],
            },
            10,
        );
        // Unknown type variants should return "unknown"
        let ty = Type::ImplTrait(vec![]);
        assert_eq!(expander.format_type(&ty), "impl <trait>");
    }

    #[test]
    fn test_path_resolver_find_by_path_empty_index() {
        let krate = Crate {
            root: Id(1),
            crate_version: None,
            includes_private: true,
            index: HashMap::new(),
            paths: HashMap::new(),
            external_crates: HashMap::new(),
            target: Target {
                triple: "x86_64-unknown-linux-gnu".to_string(),
                target_features: vec![],
            },
            format_version: 1,
        };

        let result = PathResolver::find_by_path(&krate, "nonexistent");
        assert!(result.is_empty());
    }

    #[test]
    fn test_path_resolver_find_by_path_exact_match() {
        let mut krate = Crate {
            root: Id(1),
            crate_version: None,
            includes_private: true,
            index: HashMap::new(),
            paths: HashMap::new(),
            external_crates: HashMap::new(),
            target: Target {
                triple: "x86_64-unknown-linux-gnu".to_string(),
                target_features: vec![],
            },
            format_version: 1,
        };

        krate.paths.insert(
            Id(1),
            ItemSummary {
                crate_id: 1,
                path: vec!["std".to_string()],
                kind: ItemKind::Module,
            },
        );

        krate.index.insert(
            Id(1),
            Item {
                id: Id(1),
                crate_id: 1,
                name: Some("std".to_string()),
                span: None,
                visibility: Visibility::Public,
                docs: None,
                links: HashMap::new(),
                attrs: vec![],
                deprecation: None,
                inner: ItemEnum::Module(Module {
                    is_crate: true,
                    items: vec![],
                    is_stripped: false,
                }),
            },
        );

        let result = PathResolver::find_by_path(&krate, "std");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_path_resolver_path_matches() {
        let paths = vec!["std::collections".to_string(), "std::io".to_string()];
        assert!(PathResolver::path_matches(&paths, "std::io"));
        assert!(PathResolver::path_matches(&paths, "io"));
        assert!(!PathResolver::path_matches(&paths, "other"));
    }

    #[test]
    fn test_path_resolver_path_matches_suffix() {
        let paths = vec!["std::collections::HashMap".to_string()];
        assert!(PathResolver::path_matches(&paths, "HashMap"));
        assert!(PathResolver::path_matches(&paths, "collections::HashMap"));
    }
}
