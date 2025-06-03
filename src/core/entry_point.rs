use fm::FileId;

use crate::core::resolver::mod_resolver::{DefinitionInfo, DefinitionKind, ModuleInfo};

pub fn find_hir_entry_point<'a>(
    modules: &'a [ModuleInfo],
    entry_file_id: FileId,
    entry_point_name: &str,
) -> Option<&'a DefinitionInfo> {
    modules
        .iter()
        .filter(|m| m.file_id() == entry_file_id)
        .flat_map(|m| m.definitions())
        .find(|d| d.name() == entry_point_name && matches!(d.kind(), DefinitionKind::Function))
}
