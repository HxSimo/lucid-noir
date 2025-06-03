use std::{collections::HashMap, fmt};

use fm::FileId;
use noirc_arena::Index;
use noirc_errors::Location;
use noirc_frontend::{
    ast::{Ident, ItemVisibility},
    hir::{
        Context,
        def_map::{LocalModuleId, ModuleData, ModuleDefId},
    },
    node_interner::TraitId,
};

type Scope = HashMap<Option<TraitId>, (ModuleDefId, ItemVisibility, bool /*is_prelude*/)>;

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    local_id: Index,
    parent: Option<LocalModuleId>,
    children: HashMap<Ident, LocalModuleId>,
    file: FileId,
    definitions: Vec<DefinitionInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionInfo {
    name: String,
    kind: DefinitionKind,
    def_id: ModuleDefId,
    visibility: ItemVisibility,
    is_prelude: bool,
    location: Location
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefinitionKind {
    Function,
    Global,
    Module,
}

impl ModuleInfo {
    #[must_use]
    pub fn new(
        local_id: Index,
        parent: Option<LocalModuleId>,
        children: HashMap<Ident, LocalModuleId>,
        file: FileId,
    ) -> Self {
        Self {
            local_id,
            parent,
            children,
            file,
            definitions: Vec::new(),
        }
    }

    pub fn from_module(local_mod_id: Index, module_data: &ModuleData) -> Self {
        let mut definitions = Vec::new();
        for (ident, scope) in module_data.scope().values() {
            definitions.push(DefinitionInfo::from_item_scope_value(ident, scope));
        }
        Self {
            local_id: local_mod_id,
            parent: module_data.parent,
            children: module_data.children.clone(),
            file: module_data.location.file,
            definitions: definitions,
        }
    }

    pub fn local_id(&self) -> &Index {
        &self.local_id
    }

    pub fn parent(&self) -> Option<LocalModuleId> {
        self.parent
    }

    pub fn children(&self) -> &HashMap<Ident, LocalModuleId> {
        &self.children
    }

    pub fn file_id(&self) -> FileId {
        self.file
    }

    pub fn definitions(&self) -> &[DefinitionInfo] {
        &self.definitions
    }

    pub fn add_definition(&mut self, def: DefinitionInfo) {
        self.definitions.push(def);
    }
}

impl DefinitionInfo {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        kind: DefinitionKind,
        def_id: ModuleDefId,
        visibility: ItemVisibility,
        is_prelude: bool,
        location: Location,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            def_id,
            visibility,
            is_prelude,
            location,
        }
    }

    #[must_use]
    pub fn from_item_scope_value(ident: &Ident, scope: &Scope) -> Self {
        if let Some((def_id, visibility, is_prelude)) = scope.get(&None) {
            let name = ident.as_str();
            let kind = match def_id {
                ModuleDefId::FunctionId(_) => DefinitionKind::Function,
                ModuleDefId::GlobalId(_) => DefinitionKind::Global,
                ModuleDefId::ModuleId(_) => DefinitionKind::Module,
                other => panic!("Unhandled ModuleDefId variant: {other:?}"),
            };
            let is_prelude = is_prelude;
            let location = ident.location();
            Self {
                name: name.into(),
                kind,
                def_id: *def_id,
                visibility: *visibility,
                is_prelude: *is_prelude,
                location,
            }
        } else {
            panic!(
                "Definition comes from a trait — not handled yet: {:?}",
                ident.as_str()
            );
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &DefinitionKind {
        &self.kind
    }

    pub fn def_id(&self) -> &ModuleDefId {
        &self.def_id
    }

    pub fn visibility(&self) -> &ItemVisibility {
        &self.visibility
    }

    pub fn is_stdlib(&self) -> bool {
        self.is_prelude
    }

    pub fn location(&self) -> Location {
        self.location
    }
}

impl fmt::Display for ModuleInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Module:\n- Local ID: {:?}\n- Parent: {:?}\n- Children: {:?}\n- File: {:?}",
            self.local_id, self.parent, self.children, self.file
        )?;
        writeln!(f, "- Definitions:")?;
        for def in &self.definitions {
            writeln!(f, "  • {}", def)?;
        }
        Ok(())
    }
}

impl fmt::Display for DefinitionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({:?}) [{:?}]{} @ file {:?}",
            self.name,
            self.def_id,
            self.visibility,
            if self.is_prelude { " [prelude]" } else { "" },
            self.location.file
        )
    }
}

impl fmt::Display for DefinitionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DefinitionKind::Function => "function",
            DefinitionKind::Global => "global",
            DefinitionKind::Module => "module",
        };
        write!(f, "{}", s)
    }
}

pub fn resolve_mods(context: &Context) -> Vec<ModuleInfo> {
    let mut modules: Vec<ModuleInfo> = Vec::new();
    for (crate_id, def_map) in &context.def_maps {
        if !crate_id.is_stdlib() {
            for (local_mod_id, module_data) in def_map.modules().iter() {
                modules.push(ModuleInfo::from_module(local_mod_id, module_data));
            }
        }
    }
    modules
}
