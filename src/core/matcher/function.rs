use noirc_frontend::{ast::NoirFunction, parser::ItemKind, ParsedModule};

use crate::core::resolver::mod_resolver::DefinitionInfo;

pub fn match_hir_ast_function<'a>(
    parsed: &'a ParsedModule,
    entry_fn: &DefinitionInfo,
) -> Option<&'a NoirFunction> {
    parsed.items.iter().find_map(|item| {
        if let ItemKind::Function(func) = &item.kind {
            let same_name = func.def.name.as_str() == entry_fn.name();
            let same_location = func.def.name.location() == entry_fn.location();
            if same_name && same_location {
                return Some(func);
            }
        }
        None
    })
}
