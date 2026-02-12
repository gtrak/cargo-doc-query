// Type formatting and signature extraction

use crate::types::doc::DocExtractor;
use rustdoc_types::{Abi, FunctionPointer, FunctionSignature, GenericParamDef, Type};

pub struct TypeFormatter;

impl TypeFormatter {
    /// Format a Type as a string for JSON output
    pub fn format_type(ty: &Type) -> String {
        match ty {
            Type::ResolvedPath(path) => path.path.clone(),
            Type::DynTrait(dyn_trait) => {
                let traits: Vec<String> = dyn_trait
                    .traits
                    .iter()
                    .map(|pt| pt.trait_.path.clone())
                    .collect();
                let lifetime = dyn_trait
                    .lifetime
                    .as_ref()
                    .map(|l| format!(" + {}", l))
                    .unwrap_or_default();
                format!("dyn {}{}", traits.join(" + "), lifetime)
            }
            Type::Generic(name) => name.clone(),
            Type::Primitive(prim) => prim.clone(),
            Type::Tuple(types) => {
                let inner: Vec<String> = types.iter().map(Self::format_type).collect();
                if inner.len() == 1 {
                    format!("({},)", inner[0])
                } else if inner.is_empty() {
                    "()".to_string()
                } else {
                    format!("({})", inner.join(", "))
                }
            }
            Type::Array { type_, len } => {
                format!("[{}; {}]", Self::format_type(type_), len)
            }
            Type::Slice(ty) => {
                format!("[{}]", Self::format_type(ty))
            }
            Type::BorrowedRef {
                lifetime,
                is_mutable,
                type_,
            } => {
                let lt = lifetime
                    .as_ref()
                    .map(|l| format!("{} ", l))
                    .unwrap_or_default();
                let mut_str = if *is_mutable { "mut " } else { "" };
                format!("&{}{}{}", lt, mut_str, Self::format_type(type_))
            }
            Type::RawPointer { is_mutable, type_ } => {
                let kind = if *is_mutable { "mut" } else { "const" };
                format!("*{} {}", kind, Self::format_type(type_))
            }
            Type::FunctionPointer(func_ptr) => Self::format_function_pointer(func_ptr),
            Type::ImplTrait(bounds) => {
                let bounds_str: Vec<String> = bounds
                    .iter()
                    .map(|b| Self::format_generic_bound(b))
                    .collect();
                format!("impl {}", bounds_str.join(" + "))
            }
            Type::Infer => "_".to_string(),
            Type::QualifiedPath { name, .. } => name.clone(),
            Type::Pat { type_, .. } => {
                format!("{} is ...", Self::format_type(type_))
            }
        }
    }

    /// Format a function signature for JSON output
    pub fn format_signature(sig: &FunctionSignature) -> String {
        let args: Vec<String> = sig
            .inputs
            .iter()
            .map(|(_name, ty)| Self::format_type(ty))
            .collect();

        let return_str = match &sig.output {
            Some(ty) => format!(" -> {}", Self::format_type(ty)),
            None => String::new(),
        };

        // Note: FunctionSignature doesn't have generics directly
        // Generic params are on FunctionPointer, not FunctionSignature
        format!("fn({}){}", args.join(", "), return_str)
    }

    /// Format return type for JSON output
    pub fn format_return_type(output: &Option<Type>) -> String {
        match output {
            Some(ty) => Self::format_type(ty),
            None => "()".to_string(),
        }
    }

    /// Format a function pointer
    fn format_function_pointer(func_ptr: &FunctionPointer) -> String {
        let args: Vec<String> = func_ptr
            .sig
            .inputs
            .iter()
            .map(|(_name, ty)| format_type(ty))
            .collect();

        let return_str = match &func_ptr.sig.output {
            Some(ty) => format!(" -> {}", format_type(ty)),
            None => String::new(),
        };

        let generics = if func_ptr.generic_params.is_empty() {
            String::new()
        } else {
            let generic_names: Vec<String> = func_ptr
                .generic_params
                .iter()
                .map(Self::format_generic_param)
                .collect();
            format!("<{}>", generic_names.join(", "))
        };

        let unsafe_str = if func_ptr.header.is_unsafe {
            "unsafe "
        } else {
            ""
        };
        let extern_str = match &func_ptr.header.abi {
            Abi::Rust => String::new(),
            _ => format!("extern \"{}\" ", Self::format_abi(&func_ptr.header.abi)),
        };

        format!(
            "{}{}fn{}({}){}",
            extern_str,
            unsafe_str,
            generics,
            args.join(", "),
            return_str
        )
    }

    /// Format a generic parameter
    fn format_generic_param(param: &GenericParamDef) -> String {
        param.name.clone()
    }

    /// Format an ABI
    fn format_abi(abi: &Abi) -> &str {
        match abi {
            Abi::C { .. } => "C",
            Abi::Cdecl { .. } => "cdecl",
            Abi::Stdcall { .. } => "stdcall",
            Abi::Fastcall { .. } => "fastcall",
            Abi::Aapcs { .. } => "aapcs",
            Abi::Win64 { .. } => "win64",
            Abi::SysV64 { .. } => "sysv64",
            Abi::System { .. } => "system",
            _ => "unknown",
        }
    }

    /// Format a generic bound (simplified)
    fn format_generic_bound(_bound: &rustdoc_types::GenericBound) -> String {
        // Simplified for now - most common case is Trait
        "<trait>".to_string()
    }

    /// Format generic parameter names from GenericParamDefVec
    pub fn format_generic_params(params: &[GenericParamDef]) -> String {
        params
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

// Helper function to format types (used internally)
fn format_type(ty: &Type) -> String {
    TypeFormatter::format_type(ty)
}
