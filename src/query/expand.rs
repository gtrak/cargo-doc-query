// Recursive type expansion with cycle detection and token budgeting

use anyhow::Result;
use rustdoc_types::{Crate, Id, Item, ItemEnum, Type};
use std::collections::HashMap;
use std::collections::HashSet;

use crate::cache::store::SerializableIndex;
use crate::query::lookup::PathResolver;
use crate::types::expand::{ExpansionResult, FieldInfo, TokenConfig, TypeGraph, TypeNode};

pub struct TypeExpander {
    index: SerializableIndex,
    crates: HashMap<String, Crate>,
    visited: HashSet<Id>,
    current_depth: u32,
    depth_limit: u32,
    token_config: TokenConfig,
    current_token_count: usize,
    truncated: Vec<String>,
}

impl TypeExpander {
    /// Create new expander with default config
    pub fn new(index: SerializableIndex, depth_limit: u32) -> Self {
        Self::with_config(index, depth_limit, TokenConfig::default())
    }

    /// Create new expander with custom config
    pub fn with_config(index: SerializableIndex, depth_limit: u32, config: TokenConfig) -> Self {
        Self {
            index,
            crates: HashMap::new(),
            visited: HashSet::new(),
            current_depth: 0,
            depth_limit,
            token_config: config,
            current_token_count: 0,
            truncated: Vec::new(),
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

        let json_path = &crate_node.json_path;
        let json_str = fs::read_to_string(json_path).map_err(|e| {
            anyhow::anyhow!("Failed to read rustdoc JSON from {}: {}", json_path, e)
        })?;

        let krate: Crate = serde_json::from_str(&json_str).map_err(|e| {
            anyhow::anyhow!("Failed to parse rustdoc JSON from {}: {}", json_path, e)
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

        // Collect crate names to load first (avoid borrow issues)
        let crates_to_load: Vec<(String, String)> = self
            .index
            .nodes
            .iter()
            .filter(|n| crate_filter.map_or(true, |f| n.name == f))
            .map(|n| (n.name.clone(), n.version.clone()))
            .collect();

        // Load all crates first
        for (name, version) in &crates_to_load {
            self.load_crate(name, version)?;
        }

        if self.crates.is_empty() {
            return Err(anyhow::anyhow!("No crates loaded"));
        }

        // Try to find and expand the type in each loaded crate
        // Sort keys for deterministic iteration order
        let mut crate_keys: Vec<String> = self.crates.keys().cloned().collect();
        crate_keys.sort();
        let mut found = false;

        for key in crate_keys {
            // Get a reference to the crate - we need to be careful about borrowing
            let items: Vec<(String, Id, Item)> = {
                let krate = self.crates.get(&key).unwrap();
                PathResolver::find_by_path(krate, path)
                    .into_iter()
                    .map(|(id, item)| (key.clone(), id.clone(), item.clone()))
                    .collect()
            };

            if items.is_empty() {
                continue;
            }

            found = true;

            for (crate_name, id, item) in items {
                self.visited.clear();
                self.current_depth = 0;

                match &item.inner {
                    ItemEnum::Struct(_)
                    | ItemEnum::Enum(_)
                    | ItemEnum::Union(_)
                    | ItemEnum::TypeAlias(_) => {
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
                    _ => {
                        // Skip items that aren't types or modules (macros, etc.)
                        continue;
                    }
                }
            }
        }

        if !found {
            return Err(anyhow::anyhow!("No items found matching path: {}", path));
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

        let mut node = TypeNode::new(type_path.clone(), kind.to_string(), depth);

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
        let mut node = TypeNode::new(module_path, "module".to_string(), depth);
        let mut submodules_to_expand: Vec<Id> = Vec::new();

        // Get module items
        if let ItemEnum::Module(module) = &module_item.inner {
            for item_id in &module.items {
                if let Some(item) = krate.index.get(item_id) {
                    let item_path = self.get_path(&krate, *item_id);
                    let name = item.name.clone().unwrap_or_default();

                    match &item.inner {
                        ItemEnum::Struct(_)
                        | ItemEnum::Enum(_)
                        | ItemEnum::Union(_)
                        | ItemEnum::TypeAlias(_)
                        | ItemEnum::Trait(_)
                        | ItemEnum::Function(_) => {
                            // Add as module item
                            let field_info = FieldInfo::new(name, item_path, false);
                            node.add_field(field_info);
                        }
                        ItemEnum::Module(_) => {
                            // Collect submodule IDs for later expansion
                            if depth + 1 < self.depth_limit {
                                submodules_to_expand.push(*item_id);
                            }
                        }
                        _ => {}
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
                let parts: Vec<String> = bounds.iter().map(|b| format!("{:?}", b)).collect();
                format!("impl {}", parts.join(" + "))
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
}

pub fn expand_type(path: &str, depth: u32, crate_filter: Option<&str>) -> Result<ExpansionResult> {
    let index = crate::cache::store::CacheStore::new()?
        .load_current()?
        .ok_or_else(|| {
            anyhow::anyhow!("No cached index found. Run `cargo doc-query build` first.")
        })?;

    let mut expander = TypeExpander::new(index, depth);
    expander.expand(path, crate_filter)
}

pub fn expand_type_with_config(
    path: &str,
    depth: u32,
    crate_filter: Option<&str>,
    config: TokenConfig,
) -> Result<ExpansionResult> {
    let index = crate::cache::store::CacheStore::new()?
        .load_current()?
        .ok_or_else(|| {
            anyhow::anyhow!("No cached index found. Run `cargo doc-query build` first.")
        })?;

    let mut expander = TypeExpander::with_config(index, depth, config);
    expander.expand(path, crate_filter)
}
