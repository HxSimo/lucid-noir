use noirc_frontend::{ParsedModule, ast::NoirFunction, parser::ItemKind};

pub fn find_entry_point<'a>(
    parsed_module: &'a ParsedModule,
    entry_point_name: &String,
) -> Option<&'a NoirFunction> {
    for item in &parsed_module.items {
        if let ItemKind::Function(func) = &item.kind {
            if func.def.name.as_string() == entry_point_name {
                return Some(func);
            }
        }
    }
    None
}
