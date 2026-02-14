// Type formatting and signature extraction

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
            .map(|(_name, ty)| Self::format_type(ty))
            .collect();

        let return_str = match &func_ptr.sig.output {
            Some(ty) => format!(" -> {}", Self::format_type(ty)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use rustdoc_types::Id;

    #[test]
    fn test_format_type_resolved_path() {
        let ty = Type::ResolvedPath(rustdoc_types::Path {
            path: "HashMap".to_string(),
            id: Id(123),
            args: None,
        });
        assert_eq!(TypeFormatter::format_type(&ty), "HashMap");
    }

    #[test]
    fn test_format_type_generic() {
        let ty = Type::Generic("T".to_string());
        assert_eq!(TypeFormatter::format_type(&ty), "T");
    }

    #[test]
    fn test_format_type_primitive() {
        let ty = Type::Primitive("u32".to_string());
        assert_eq!(TypeFormatter::format_type(&ty), "u32");
    }

    #[test]
    fn test_format_type_unit() {
        let ty = Type::Tuple(vec![]);
        assert_eq!(TypeFormatter::format_type(&ty), "()");
    }

    #[test]
    fn test_format_type_single_item_tuple() {
        let ty = Type::Tuple(vec![Type::Generic("T".to_string())]);
        assert_eq!(TypeFormatter::format_type(&ty), "(T,)");
    }

    #[test]
    fn test_format_type_multi_item_tuple() {
        let ty = Type::Tuple(vec![
            Type::Generic("T".to_string()),
            Type::Generic("U".to_string()),
        ]);
        assert_eq!(TypeFormatter::format_type(&ty), "(T, U)");
    }

    #[test]
    fn test_format_type_array() {
        let ty = Type::Array {
            type_: Box::new(Type::Generic("T".to_string())),
            len: "10".to_string(),
        };
        assert_eq!(TypeFormatter::format_type(&ty), "[T; 10]");
    }

    #[test]
    fn test_format_type_slice() {
        let ty = Type::Slice(Box::new(Type::Generic("T".to_string())));
        assert_eq!(TypeFormatter::format_type(&ty), "[T]");
    }

    #[test]
    fn test_format_type_ref_immutable() {
        let ty = Type::BorrowedRef {
            lifetime: None,
            is_mutable: false,
            type_: Box::new(Type::Generic("T".to_string())),
        };
        assert_eq!(TypeFormatter::format_type(&ty), "&T");
    }

    #[test]
    fn test_format_type_ref_mutable() {
        let ty = Type::BorrowedRef {
            lifetime: None,
            is_mutable: true,
            type_: Box::new(Type::Generic("T".to_string())),
        };
        assert_eq!(TypeFormatter::format_type(&ty), "&mut T");
    }

    #[test]
    fn test_format_type_ref_with_lifetime() {
        let ty = Type::BorrowedRef {
            lifetime: Some("'a".to_string()),
            is_mutable: false,
            type_: Box::new(Type::Generic("T".to_string())),
        };
        assert_eq!(TypeFormatter::format_type(&ty), "&'a T");
    }

    #[test]
    fn test_format_type_raw_const_pointer() {
        let ty = Type::RawPointer {
            is_mutable: false,
            type_: Box::new(Type::Generic("T".to_string())),
        };
        assert_eq!(TypeFormatter::format_type(&ty), "*const T");
    }

    #[test]
    fn test_format_type_raw_mut_pointer() {
        let ty = Type::RawPointer {
            is_mutable: true,
            type_: Box::new(Type::Generic("T".to_string())),
        };
        assert_eq!(TypeFormatter::format_type(&ty), "*mut T");
    }

    #[test]
    fn test_format_type_infer() {
        let ty = Type::Infer;
        assert_eq!(TypeFormatter::format_type(&ty), "_");
    }

    #[test]
    fn test_format_signature_no_args_no_return() {
        let sig = FunctionSignature {
            is_c_variadic: false,
            inputs: vec![],
            output: None,
        };
        assert_eq!(TypeFormatter::format_signature(&sig), "fn()");
    }

    #[test]
    fn test_format_signature_with_args() {
        let sig = FunctionSignature {
            is_c_variadic: false,
            inputs: vec![
                ("x".to_string(), Type::Generic("x".to_string())),
                ("y".to_string(), Type::Generic("y".to_string())),
            ],
            output: None,
        };
        assert_eq!(TypeFormatter::format_signature(&sig), "fn(x, y)");
    }

    #[test]
    fn test_format_signature_with_return() {
        let sig = FunctionSignature {
            is_c_variadic: false,
            inputs: vec![],
            output: Some(Type::Generic("Result".to_string())),
        };
        assert_eq!(TypeFormatter::format_signature(&sig), "fn() -> Result");
    }

    #[test]
    fn test_format_signature_with_args_and_return() {
        let sig = FunctionSignature {
            is_c_variadic: false,
            inputs: vec![("x".to_string(), Type::Generic("x".to_string()))],
            output: Some(Type::Generic("u32".to_string())),
        };
        assert_eq!(TypeFormatter::format_signature(&sig), "fn(x) -> u32");
    }

    #[test]
    fn test_format_function_pointer() {
        let func_ptr = FunctionPointer {
            sig: FunctionSignature {
                is_c_variadic: false,
                inputs: vec![],
                output: None,
            },
            generic_params: vec![],
            header: rustdoc_types::FunctionHeader {
                is_const: false,
                is_unsafe: false,
                is_async: false,
                abi: rustdoc_types::Abi::Rust,
            },
        };
        assert!(TypeFormatter::format_function_pointer(&func_ptr).contains("fn()"));
    }

    #[test]
    fn test_format_function_pointer_with_generics() {
        let func_ptr = FunctionPointer {
            sig: FunctionSignature {
                is_c_variadic: false,
                inputs: vec![],
                output: None,
            },
            generic_params: vec![GenericParamDef {
                name: "T".to_string(),
                kind: rustdoc_types::GenericParamDefKind::Type {
                    bounds: vec![],
                    default: None,
                    is_synthetic: false,
                },
            }],
            header: rustdoc_types::FunctionHeader {
                is_const: false,
                is_unsafe: false,
                is_async: false,
                abi: rustdoc_types::Abi::Rust,
            },
        };
        let result = TypeFormatter::format_function_pointer(&func_ptr);
        assert!(result.contains("<T>"));
    }

    #[test]
    fn test_format_abi_rust() {
        let abi = rustdoc_types::Abi::Rust;
        assert_eq!(TypeFormatter::format_abi(&abi), "unknown");
    }

    #[test]
    fn test_format_abi_c() {
        let abi = rustdoc_types::Abi::C { unwind: false };
        assert_eq!(TypeFormatter::format_abi(&abi), "C");
    }

    #[test]
    fn test_format_abi_stdcall() {
        let abi = rustdoc_types::Abi::Stdcall { unwind: false };
        assert_eq!(TypeFormatter::format_abi(&abi), "stdcall");
    }

    #[test]
    fn test_format_abi_fastcall() {
        let abi = rustdoc_types::Abi::Fastcall { unwind: false };
        assert_eq!(TypeFormatter::format_abi(&abi), "fastcall");
    }

    #[test]
    fn test_format_generic_params_empty() {
        let params: Vec<GenericParamDef> = vec![];
        assert_eq!(TypeFormatter::format_generic_params(&params), "");
    }

    #[test]
    fn test_format_generic_params_single() {
        let params = vec![GenericParamDef {
            name: "T".to_string(),
            kind: rustdoc_types::GenericParamDefKind::Type {
                bounds: vec![],
                default: None,
                is_synthetic: false,
            },
        }];
        assert_eq!(TypeFormatter::format_generic_params(&params), "T");
    }

    #[test]
    fn test_format_generic_params_multiple() {
        let params = vec![
            GenericParamDef {
                name: "T".to_string(),
                kind: rustdoc_types::GenericParamDefKind::Type {
                    bounds: vec![],
                    default: None,
                    is_synthetic: false,
                },
            },
            GenericParamDef {
                name: "U".to_string(),
                kind: rustdoc_types::GenericParamDefKind::Type {
                    bounds: vec![],
                    default: None,
                    is_synthetic: false,
                },
            },
            GenericParamDef {
                name: "V".to_string(),
                kind: rustdoc_types::GenericParamDefKind::Type {
                    bounds: vec![],
                    default: None,
                    is_synthetic: false,
                },
            },
        ];
        assert_eq!(TypeFormatter::format_generic_params(&params), "T, U, V");
    }

    #[test]
    fn test_format_return_type_none() {
        assert_eq!(TypeFormatter::format_return_type(&None), "()");
    }

    #[test]
    fn test_format_return_type_some() {
        let ty = Type::ResolvedPath(rustdoc_types::Path {
            path: "String".to_string(),
            id: Id(123),
            args: None,
        });
        assert_eq!(TypeFormatter::format_return_type(&Some(ty)), "String");
    }

    #[test]
    fn test_format_dyn_trait_with_traits() {
        let ty = Type::DynTrait(rustdoc_types::DynTrait {
            lifetime: None,
            traits: vec![rustdoc_types::PolyTrait {
                trait_: rustdoc_types::Path {
                    path: "Display".to_string(),
                    id: Id(1),
                    args: None,
                },
                generic_params: vec![],
            }],
        });
        assert!(TypeFormatter::format_type(&ty).contains("Display"));
    }

    #[test]
    fn test_format_nested_type_tuple_in_tuple() {
        let ty = Type::Tuple(vec![
            Type::Tuple(vec![Type::Generic("A".to_string())]),
            Type::Tuple(vec![Type::Generic("B".to_string())]),
        ]);
        assert_eq!(TypeFormatter::format_type(&ty), "((A,), (B,))");
    }
}
