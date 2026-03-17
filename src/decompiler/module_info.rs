use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::Deserialize;
use tracing::info;

use crate::str_constants::FILE_UDON_MODULE_INFO_JSON;

use super::DecompileError;

static DEFAULT_MODULE_INFO_PATH: OnceLock<PathBuf> = OnceLock::new();
static DEFAULT_MODULE_INFO_CACHE: OnceLock<Result<UdonModuleInfo, String>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum FunctionDefinitionType {
    #[serde(rename = "method")]
    Method,
    #[serde(rename = "prop")]
    Field,
    #[serde(rename = "ctor")]
    Ctor,
    #[serde(rename = "op")]
    Operator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ParameterType {
    #[serde(rename = "IN")]
    In,
    #[serde(rename = "OUT")]
    Out,
    #[serde(rename = "IN_OUT")]
    InOut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternFunctionInfo {
    pub signature: String,
    pub type_name: String,
    pub function_name: String,
    pub is_static: bool,
    pub returns_void: bool,
    pub def_type: FunctionDefinitionType,
    pub parameters: Vec<ParameterType>,
    pub original_name: Option<String>,
}

impl ExternFunctionInfo {
    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }
}

#[derive(Debug, Clone)]
pub struct FunctionMetadata {
    pub def_type: FunctionDefinitionType,
    pub is_static: bool,
    pub returns_void: bool,
    pub parameters: Vec<ParameterType>,
    pub original_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleMetadata {
    pub type_name: String,
    pub functions: HashMap<String, FunctionMetadata>,
}

#[derive(Debug, Clone, Default)]
pub struct UdonModuleInfo {
    pub modules: HashMap<String, ModuleMetadata>,
}

impl UdonModuleInfo {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, DecompileError> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path).map_err(|e| {
            DecompileError::new(format!("Failed to read {}: {}", path.display(), e))
        })?;
        Self::load_from_json_string(&raw)
    }

    pub fn load_from_json_string(text: &str) -> Result<Self, DecompileError> {
        let decoded = serde_json::from_str::<HashMap<String, RawModuleMetadata>>(text)
            .map_err(|e| DecompileError::new(format!("Failed to parse module info json: {}", e)))?;
        let mut out = UdonModuleInfo::default();
        for (module_name, module_data) in decoded {
            let mut functions = HashMap::<String, FunctionMetadata>::new();
            for func in module_data.functions {
                functions.insert(
                    func.name.clone(),
                    FunctionMetadata {
                        def_type: func.def_type,
                        is_static: func.is_static,
                        returns_void: func.returns_void,
                        parameters: func.parameters,
                        original_name: func.original_name,
                    },
                );
            }
            out.modules.insert(
                module_name,
                ModuleMetadata {
                    type_name: module_data.type_name,
                    functions,
                },
            );
        }
        info!("successfully loaded module info");
        Ok(out)
    }

    pub fn set_default_module_info_path(path: impl Into<PathBuf>) -> Result<(), DecompileError> {
        let desired = path.into();
        if let Some(existing) = DEFAULT_MODULE_INFO_PATH.get() {
            if existing == &desired {
                return Ok(());
            }
            return Err(DecompileError::new(format!(
                "module info path already configured as '{}' and cannot be changed to '{}'",
                existing.display(),
                desired.display()
            )));
        }
        DEFAULT_MODULE_INFO_PATH
            .set(desired)
            .map_err(|_| DecompileError::new("failed to configure module info path"))?;
        Ok(())
    }

    pub fn load_default_cached() -> Result<&'static UdonModuleInfo, DecompileError> {
        let cached = DEFAULT_MODULE_INFO_CACHE.get_or_init(|| {
            let path = DEFAULT_MODULE_INFO_PATH
                .get()
                .cloned()
                .unwrap_or_else(|| PathBuf::from(FILE_UDON_MODULE_INFO_JSON));
            UdonModuleInfo::load_from_file(&path).map_err(|e| e.to_string())
        });
        match cached {
            Ok(v) => Ok(v),
            Err(e) => Err(DecompileError::new(e.clone())),
        }
    }

    pub fn get_function_info(&self, signature: &str) -> Option<ExternFunctionInfo> {
        let (module_name, function_name) = Self::parse_signature(signature)?;
        let module = self.modules.get(module_name)?;
        let function = module.functions.get(function_name)?;
        Some(ExternFunctionInfo {
            signature: signature.to_string(),
            type_name: module.type_name.clone(),
            function_name: function_name.to_string(),
            is_static: function.is_static,
            returns_void: function.returns_void,
            def_type: function.def_type,
            parameters: function.parameters.clone(),
            original_name: function.original_name.clone(),
        })
    }

    fn parse_signature(signature: &str) -> Option<(&str, &str)> {
        let mut parts = signature.splitn(3, '.');
        let module_name = parts.next()?;
        let function_name = parts.next()?;
        Some((module_name, function_name))
    }
}

#[derive(Debug, Deserialize)]
struct RawModuleMetadata {
    #[serde(rename = "type")]
    type_name: String,
    #[serde(default)]
    functions: Vec<RawFunctionMetadata>,
}

#[derive(Debug, Deserialize)]
struct RawFunctionMetadata {
    name: String,
    #[serde(default)]
    parameters: Vec<ParameterType>,
    #[serde(rename = "defType")]
    def_type: FunctionDefinitionType,
    #[serde(rename = "isStatic", default)]
    is_static: bool,
    #[serde(rename = "returnsVoid", default)]
    returns_void: bool,
    #[serde(rename = "originalName")]
    original_name: Option<String>,
}
