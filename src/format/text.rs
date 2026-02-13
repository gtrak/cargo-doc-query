//! Human-readable text output formatters

use crate::types::query::{QueryContent, QueryMatch, QueryResponse};
use console::style;

/// Format a query response as human-readable text
pub fn format_query_response(response: &QueryResponse, query_path: &str) {
    if response.matches.is_empty() {
        println!(
            "{}",
            style(format!("No results found for: {}", query_path)).red()
        );
        return;
    }

    for (i, match_) in response.matches.iter().enumerate() {
        if i > 0 {
            println!(); // Blank line between matches
        }
        format_query_match(match_);
    }
}

fn format_query_match(match_: &QueryMatch) {
    // Header: crate::Type (kind)
    let header = format!(
        "{}::{} ({}",
        match_.crate_name, match_.fully_qualified_path, match_.kind
    );
    println!("{}", style(header).bold().cyan());
    println!("{}", style("─".repeat(60)).dim());

    match &match_.content {
        QueryContent::Type(type_result) => format_type_result(type_result),
        QueryContent::Trait(trait_result) => format_trait_result(trait_result),
        QueryContent::Module(module_result) => format_module_result(module_result),
    }
}

fn format_type_result(type_result: &crate::types::query::TypeResult) {
    // Kind
    println!("  {}: {}", style("Kind").bold(), type_result.kind);

    // Methods
    if !type_result.methods.is_empty() {
        println!("\n  {}", style("Methods:").bold());
        for method in &type_result.methods {
            format_method(method, 2);
        }
    }

    // Trait implementations
    if !type_result.trait_implementations.is_empty() {
        println!("\n  {}", style("Trait Implementations:").bold());
        for trait_impl in &type_result.trait_implementations {
            format_trait_impl(trait_impl, 2);
        }
    }
}

fn format_trait_result(trait_result: &crate::types::query::TraitResult) {
    // Associated types
    if !trait_result.associated_types.is_empty() {
        println!("\n  {}", style("Associated Types:").bold());
        for assoc_type in &trait_result.associated_types {
            let bounds_str = assoc_type
                .bounds
                .as_ref()
                .map(|b| format!(" where {}", b))
                .unwrap_or_default();
            let default_str = assoc_type
                .default
                .as_ref()
                .map(|d| format!(" = {}", d))
                .unwrap_or_default();
            println!(
                "    • {}{}{}",
                style(&assoc_type.name).yellow(),
                bounds_str,
                default_str
            );
        }
    }

    // Methods
    if !trait_result.methods.is_empty() {
        println!("\n  {}", style("Methods:").bold());
        for method in &trait_result.methods {
            format_method(method, 2);
        }
    }
}

fn format_module_result(module_result: &crate::types::query::ModuleResult) {
    // Items grouped by kind
    let mut items_by_kind: std::collections::HashMap<&str, Vec<&crate::types::query::ModuleItem>> =
        std::collections::HashMap::new();

    for item in &module_result.items {
        items_by_kind.entry(&item.kind).or_default().push(item);
    }

    // Display items by kind in a consistent order
    let kind_order = [
        "struct",
        "enum",
        "trait",
        "type",
        "function",
        "macro",
        "re-export",
        "const",
        "static",
    ];

    for kind in &kind_order {
        if let Some(items) = items_by_kind.get(*kind) {
            if !items.is_empty() {
                println!(
                    "\n  {} ({}):",
                    style(format!("{:?}", kind)).bold(),
                    items.len()
                );
                for item in items {
                    if item.name.is_empty() {
                        println!("    • {}", style(&item.path).dim());
                    } else {
                        println!("    • {}: {}", style(&item.name).yellow(), item.path);
                    }
                }
            }
        }
    }

    // Submodules
    if !module_result.submodules.is_empty() {
        println!("\n  {}:", style("Submodules").bold());
        for submodule in &module_result.submodules {
            println!("    • {}", style(submodule).magenta());
        }
    }
}

fn format_method(method: &crate::types::query::MethodOutput, indent: usize) {
    let indent_str = "  ".repeat(indent);

    // Signature line
    println!(
        "{}• {} {}",
        indent_str,
        style(&method.name).yellow(),
        style(&method.signature).dim()
    );

    // Return type
    if !method.return_type.is_empty() && method.return_type != "()" {
        println!("{}  → {}", indent_str, style(&method.return_type).green());
    }

    // Docs (if present)
    if let Some(docs) = &method.docs {
        let trimmed = docs.trim();
        if !trimmed.is_empty() {
            // Show first line only for brevity
            let first_line = trimmed.lines().next().unwrap_or(trimmed);
            if first_line.len() > 80 {
                println!("{}  {}", indent_str, style(&first_line[..77]).dim());
                println!("{}  {}", indent_str, style("...").dim());
            } else {
                println!("{}  {}", indent_str, style(first_line).dim());
            }
        }
    }
}

fn format_trait_impl(trait_impl: &crate::types::query::TraitImplOutput, indent: usize) {
    let indent_str = "  ".repeat(indent);

    println!("{}• {}", indent_str, style(&trait_impl.trait_name).yellow());

    if !trait_impl.methods.is_empty() {
        println!("{}  Methods:", indent_str);
        for method in &trait_impl.methods {
            format_method(method, indent + 2);
        }
    }
}

/// Format an expansion result as human-readable text
pub fn format_expand_result(result: &crate::types::expand::ExpansionResult, root_path: &str) {
    println!(
        "{}",
        style(format!("Expanding: {}", root_path)).bold().cyan()
    );
    println!("{}", style("─".repeat(60)).dim());

    if result.graph.nodes.is_empty() {
        println!("{}", style("No types found in expansion").yellow());
        return;
    }

    // Group nodes by depth
    let mut nodes_by_depth: std::collections::HashMap<u32, Vec<&crate::types::expand::TypeNode>> =
        std::collections::HashMap::new();

    for node in &result.graph.nodes {
        nodes_by_depth.entry(node.depth).or_default().push(node);
    }

    // Print nodes by depth
    let mut depths: Vec<u32> = nodes_by_depth.keys().copied().collect();
    depths.sort();

    for depth in depths {
        if let Some(nodes) = nodes_by_depth.get(&depth) {
            if depth == 0 {
                // Root level
                for node in nodes {
                    format_type_node(node, 0);
                }
            } else {
                // Nested levels
                println!("\n  {}", style(format!("Depth {}:", depth)).bold().dim());
                for node in nodes {
                    format_type_node(node, 2);
                }
            }
        }
    }

    // Show warnings
    if result.budget_exceeded {
        println!(
            "\n{}",
            style("⚠ Token budget exceeded - some types truncated").yellow()
        );
    }
}

fn format_type_node(node: &crate::types::expand::TypeNode, indent: usize) {
    let indent_str = "  ".repeat(indent);

    // Header: path (kind)
    let kind_style = match node.kind.as_str() {
        "struct" => style(&node.kind).green(),
        "enum" => style(&node.kind).magenta(),
        "trait" => style(&node.kind).cyan(),
        "module" => style(&node.kind).blue(),
        "function" => style(&node.kind).yellow(),
        _ => style(&node.kind).dim(),
    };

    println!(
        "{}• {} ({})",
        indent_str,
        style(&node.id).bold(),
        kind_style
    );

    // Fields
    if !node.fields.is_empty() {
        for field in &node.fields {
            if field.is_optional {
                println!(
                    "{}  • {}: {} (optional)",
                    indent_str,
                    style(&field.name).yellow(),
                    style(&field.type_path).dim()
                );
            } else {
                println!(
                    "{}  • {}: {}",
                    indent_str,
                    style(&field.name).yellow(),
                    style(&field.type_path).dim()
                );
            }
        }
    }

    // Variants (for enums)
    if !node.variants.is_empty() {
        for variant in &node.variants {
            if variant.fields.is_empty() {
                println!("{}  • {}", indent_str, style(&variant.name).yellow());
            } else {
                println!(
                    "{}  • {} ({})",
                    indent_str,
                    style(&variant.name).yellow(),
                    style(format!("{} fields", variant.fields.len())).dim()
                );
            }
        }
    }

    // Module items
    if !node.items.is_empty() {
        // Group by kind
        let mut by_kind: std::collections::HashMap<
            &str,
            Vec<&crate::types::expand::ModuleItemInfo>,
        > = std::collections::HashMap::new();
        for item in &node.items {
            by_kind.entry(&item.kind).or_default().push(item);
        }

        for (kind, items) in by_kind {
            let kind_label = match kind {
                "struct" => style("Structs").green(),
                "enum" => style("Enums").magenta(),
                "trait" => style("Traits").cyan(),
                "function" => style("Functions").yellow(),
                "re-export" => style("Re-exports").dim(),
                _ => style(kind).dim(),
            };
            println!("{}  {}:", indent_str, kind_label);
            for item in items {
                println!(
                    "{}    • {}: {}",
                    indent_str,
                    style(&item.name).yellow(),
                    item.path
                );
            }
        }
    }

    // Generic params
    if !node.generic_params.is_empty() {
        println!("{}  <{}>", indent_str, node.generic_params.join(", "));
    }
}
