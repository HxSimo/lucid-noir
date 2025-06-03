use fm::{FileId, FileManager};
use lucid_noir::core::{entry_point::find_hir_entry_point, matcher::function::match_hir_ast_function, resolver::mod_resolver::resolve_mods};
use std::{collections::HashMap, fs::File, path::Path};
use walkdir::WalkDir;

use log::{LevelFilter, info};
use simplelog::{Config, WriteLogger};

use noirc_driver::{CompileOptions, compile_main, file_manager_with_stdlib, prepare_crate};
use noirc_frontend::{
    ParsedModule,
    hir::{Context, def_map::parse_file},
};

fn main() {
    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("lucid_noir.log").unwrap(),
    )
    .unwrap();

    // TODO: make those variables configurable
    let project_root = "noir/src/";
    let project_root_path = Path::new(project_root);
    let entry_path_str = format!("{}main.nr", project_root);
    let entry_file = Path::new(&entry_path_str);
    let entry_point_name = "main";

    let fm = setup_fm_from_path(project_root_path);
    let entry_file_id = fm
        .name_to_id(entry_file.to_path_buf())
        .expect(&format!("{:?} not find in fileId", entry_file));

    let parsed_files: HashMap<_, _> = fm
        .as_file_map()
        .all_file_ids()
        .map(|&fid| (fid, parse_file(&fm, fid)))
        .collect();

    let file_map = fm.as_file_map();
    for file_id in parsed_files.keys() {
        let name = file_map
            .get_absolute_name(file_id.clone())
            .unwrap()
            .to_string();
        if name.contains(project_root) {
            info!("Parsed file ID: {:?}: {:?}", file_id, name);
        }
    }

    let parsed_module: &ParsedModule = &parsed_files[&entry_file_id].0.clone();

    let context = match compile_circuit(entry_file, fm, parsed_files) {
        Ok(ctx) => {
            println!("✅ Compilation successful.");
            ctx
        }
        Err(err) => {
            panic!("❌ Compilation error:\n{err:?}");
        }
    };

    let modules = resolve_mods(&context);
    for module in &modules {
        info!("{}", module);
    }

    let entry_point_fn = find_hir_entry_point(&modules, entry_file_id, &entry_point_name.to_string())
        .unwrap_or_else(|| panic!("Entrypoint function '{}' not found", entry_point_name));

    let ast_entry_point = match_hir_ast_function(parsed_module, entry_point_fn);

    info!("{:?}", ast_entry_point);
    
       

}

fn setup_fm_from_path(project_root: &Path) -> FileManager {
    let mut fm = file_manager_with_stdlib(project_root);

    for entry in WalkDir::new(project_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "nr"))
    {
        let absolute_path = entry.path();
        let relative_path = absolute_path.strip_prefix(project_root).unwrap();
        let source = std::fs::read_to_string(absolute_path).unwrap();

        fm.add_file_with_source(relative_path, source);
    }

    fm
}

fn compile_circuit(
    entry_file: &Path,
    fm: FileManager,
    parsed_files: HashMap<FileId, (ParsedModule, Vec<noirc_frontend::parser::ParserError>)>,
) -> Result<Context, noirc_driver::ErrorsAndWarnings> {
    let mut context = Context::new(fm, parsed_files);
    let crate_id = prepare_crate(&mut context, entry_file);

    let (_compiled, warnings) =
        compile_main(&mut context, crate_id, &CompileOptions::default(), None)?;

    if !warnings.is_empty() {
        println!("⚠️ Warnings:");
        for diag in warnings {
            println!("- {}", diag.message);
        }
    }

    Ok(context)
}
