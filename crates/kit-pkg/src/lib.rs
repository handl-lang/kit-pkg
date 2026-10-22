//! `handl-kit-pkg` — project/package discovery for the prelude kit.
//!
//! Extracted from `handl-kit-prelude`'s `Kit` impl: `.handl.toml`
//! manifest discovery, `[dependencies]` wiring into the module loader,
//! and the `module_root` constructors. Provided as an extension trait so
//! call sites read the same as before (`Kit::discover(path)` with
//! `use handl_kit_pkg::KitProjectExt`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use handl_kit_prelude::{Kit, prelude_engine_registry};
use handl_language::LangError;

/// Resolve a project's `.hnl` dependencies into an import-name → absolute
/// package directory map for `ModuleLoader::with_dependencies`.
///
/// Delegates to [`handl_platform::resolve_dependencies`] so the kit and the
/// CLI agree on one loader: `.handl.toml` `[dependencies]` (validated) plus
/// the package repo's own `pkg.hnl` `deps` table (lenient), with
/// `.handl.toml` winning on a name collision. `pkg.hnl`'s refs are not
/// fetched — locally a dep resolves to the sibling `lib-<name>` checkout
/// (ADR-0128).
fn load_project_dependencies(project_root: &Path) -> Result<HashMap<String, PathBuf>, LangError> {
    handl_platform::resolve_dependencies(project_root)
        .map_err(|e| LangError::engine(format!("project dependencies: {e}")))
}

/// Project-discovery constructors for [`Kit`].
pub trait KitProjectExt: Sized {
    /// Build the prelude kit with a `module_root` directory for resolving
    /// relative `import "path"` statements (Phase 9.5).
    ///
    /// # Errors
    ///
    /// Returns [`LangError`] if fragment assembly or engine binding fails.
    fn new_with_module_root(module_root: PathBuf) -> Result<Self, LangError>;

    /// Build the prelude kit for a project directory: reads
    /// `<project_root>/.handl.toml` (if present) and wires its
    /// `[dependencies]` table into the module loader, with `project_root`
    /// as the module root for relative `import "path"` resolution.
    ///
    /// Each `[dependencies]` entry maps an import name to a package
    /// directory: `import "<name>/<rest>"` resolves `<rest>` inside
    /// `<project_root>/<dep.path>` (absolute dep paths are honored as-is).
    /// Vendored/path dependencies only — no registry (ADR-0010 is the
    /// content-addressed future).
    ///
    /// # Errors
    ///
    /// Returns [`LangError`] if the manifest exists but fails to parse, or
    /// if fragment assembly / engine binding fails.
    fn new_with_project(project_root: PathBuf) -> Result<Self, LangError>;

    /// Build the prelude kit for a project, discovering its `.handl.toml`
    /// manifest by walking up the directory tree from `start` (a file or
    /// directory). This is the `hnl run`/`hnl eval` entry point for
    /// project-aware imports.
    ///
    /// - If a `.handl.toml` is found in `start` or any ancestor, the
    ///   manifest's directory becomes the module root and its
    ///   `[dependencies]` table is wired into the module loader.
    /// - Otherwise the module root is `start` itself (its parent directory
    ///   when `start` is a file) and no dependencies are declared.
    ///
    /// # Errors
    ///
    /// Returns [`LangError`] if a discovered manifest fails to parse, or if
    /// fragment assembly / engine binding fails.
    fn discover(start: &Path) -> Result<Self, LangError>;

    /// Like [`KitProjectExt::discover`] but with a specific eval engine
    /// name (e.g. `"bytecode"`).
    ///
    /// # Errors
    ///
    /// See [`KitProjectExt::discover`]; additionally returns
    /// [`LangError::Engine`] for an unknown engine name.
    fn discover_with_eval(start: &Path, eval_name: &str) -> Result<Self, LangError>;
}

impl KitProjectExt for Kit {
    fn new_with_module_root(module_root: PathBuf) -> Result<Self, LangError> {
        let kit = handl_kit_data_prelude::assemble()?;
        let lang = kit.lang().clone();
        let config = kit.runtime_config().clone().with_typecheck("nbe");
        Self::build_kit(
            lang,
            config,
            Some(module_root),
            HashMap::new(),
            None,
            prelude_engine_registry(),
        )
    }

    fn new_with_project(project_root: PathBuf) -> Result<Self, LangError> {
        let kit = handl_kit_data_prelude::assemble()?;
        let lang = kit.lang().clone();
        let config = kit.runtime_config().clone().with_typecheck("nbe");
        let deps = load_project_dependencies(&project_root)?;
        Self::build_kit(
            lang,
            config,
            Some(project_root),
            deps,
            None,
            prelude_engine_registry(),
        )
    }

    fn discover(start: &Path) -> Result<Self, LangError> {
        Self::discover_inner(start, None)
    }

    fn discover_with_eval(start: &Path, eval_name: &str) -> Result<Self, LangError> {
        Self::discover_inner(start, Some(eval_name))
    }
}

trait DiscoverInner {
    fn discover_inner(start: &Path, eval_name: Option<&str>) -> Result<Self, LangError>
    where
        Self: Sized;
}

impl DiscoverInner for Kit {
    fn discover_inner(start: &Path, eval_name: Option<&str>) -> Result<Self, LangError> {
        // Normalize `start` to a directory: a file path contributes its
        // parent directory.
        let start_dir = if start.is_file() {
            start.parent().map(Path::to_path_buf)
        } else {
            Some(start.to_path_buf())
        }
        .and_then(|p| std::fs::canonicalize(p).ok())
        .unwrap_or_else(|| PathBuf::from("."));

        // Walk up looking for `.handl.toml`.
        let manifest_dir = start_dir
            .ancestors()
            .find(|dir| dir.join(".handl.toml").is_file())
            .map(Path::to_path_buf);

        // Assemble the prelude data kit. Custom kit selection via
        // `.handl.toml` is temporarily disabled until `DataKit` is
        // re-exported for caller assembly (ADR-0103 follow-up).
        let data_kit = handl_kit_data_prelude::assemble()?;
        let lang = data_kit.lang().clone();
        let mut config = data_kit.runtime_config().clone().with_typecheck("nbe");
        if let Some(name) = eval_name {
            config = config.with_eval(name);
        }

        let deps = manifest_dir
            .map(|root| load_project_dependencies(&root))
            .unwrap_or_else(|| Ok(HashMap::new()))?;
        Kit::build_kit(
            lang,
            config,
            Some(start_dir),
            deps,
            None,
            prelude_engine_registry(),
        )
    }
}
