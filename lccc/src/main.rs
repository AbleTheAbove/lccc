#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
mod argparse;

use crate::argparse::{parse_args, ArgSpec, TakesArg};
use std::fs::File;
use std::io::ErrorKind;
use std::ops::Deref;
use std::path::PathBuf;
use std::process::Command;
use target_tuples::Target;
use xlang::abi::io::{ReadAdapter, WriteAdapter};
use xlang::abi::string::StringView;
use xlang::plugin::{XLangCodegen, XLangFrontend, XLangPlugin};
use xlang::prelude::v1::*;
use xlang_host::dso::Handle;

static FRONTENDS: [&str; 1] = ["c"];
static CODEGENS: [&str; 1] = ["x86"];
type FrontendInit = extern "C" fn() -> DynBox<dyn XLangFrontend>;
type CodegenInit = extern "C" fn() -> DynBox<dyn XLangCodegen>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Preprocess,
    TypeCheck,
    Xir,
    Asm,
    CompileOnly,
    Link,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LinkOutput {
    Shared,
    Static,
    Executable,
    Pie,
    Manifest,
}

fn find_libraries(search_paths: &[PathBuf], names: &[&str], prefix: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();
    for &library in names {
        let library_name = if cfg!(windows) {
            prefix.to_owned() + "_" + library + ".dll"
        } else if cfg!(target_os = "linux") {
            "lib".to_owned() + prefix + "_" + library + ".so"
        } else if cfg!(target_os = "macos") {
            "lib".to_owned() + prefix + "_" + library + ".dylib"
        } else {
            panic!("unrecognized target OS; can't get frontend library name")
        };

        let mut path = None;
        for search_path in search_paths {
            let mut library_path = search_path.clone();
            library_path.push(&library_name);
            if library_path.exists() {
                path = Some(library_path);
                break;
            }
        }

        if let Some(path) = path {
            result.push(path);
        } else {
            eprintln!(
                "warning: couldn't locate library to load for {} \"{}\"",
                prefix, library
            );
        }
    }
    result
}

const XLANG_PLUGIN_DIR: &str = std::env!("xlang_plugin_dir");

#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
fn main() {
    let mut target = target_tuples::from_env!("default_target");
    let mut output = None;

    let mut tmpfiles = Vec::new();

    let mut mode = Mode::Link;
    let mut link_output = LinkOutput::Executable;

    let argspecs = xlang::vec![
        ArgSpec::new(
            "output",
            Vec::new(),
            xlang::vec!['o'],
            TakesArg::Always,
            true,
        ),
        ArgSpec::new(
            "intree",
            xlang::vec!["intree", "in-tree"],
            Vec::new(),
            TakesArg::Never,
            true
        ),
        ArgSpec::new(
            "plugindirs",
            xlang::vec!["plugin-dirs"],
            Vec::new(),
            TakesArg::Always,
            false
        ),
        ArgSpec::new(
            "target",
            xlang::vec!["target"],
            Vec::new(),
            TakesArg::Always,
            true
        ),
        ArgSpec::new(
            "compile",
            xlang::vec!["compile-only"],
            xlang::vec!['c'],
            TakesArg::Never,
            true
        ),
        ArgSpec::new(
            "typeck",
            xlang::vec!["type-check"],
            Vec::new(),
            TakesArg::Never,
            true
        ),
        ArgSpec::new(
            "shared",
            xlang::vec!["shared"],
            Vec::new(),
            TakesArg::Never,
            true
        ),
        ArgSpec::new(
            "ldout",
            xlang::vec!["linker-output"],
            Vec::new(),
            TakesArg::Never,
            true
        ),
    ];

    let (args, files) = parse_args(&argspecs);
    println!("{:?} {:?}", args, files);

    let mut search_paths = Vec::new();

    let mut intree = false;

    for arg in &args {
        match arg.name {
            "intree" => intree = true,
            "plugindirs" => search_paths.push(PathBuf::from(arg.value.as_deref().unwrap())),
            "target" => target = Target::parse(arg.value.as_ref().unwrap()),
            "output" => output = arg.value.clone(),
            "compile" => mode = Mode::CompileOnly,
            "typeck" => mode = Mode::TypeCheck,
            "ldout" => {
                link_output = match arg.value.as_deref() {
                    Some("shared") => LinkOutput::Shared,
                    Some("static") => LinkOutput::Static,
                    Some("pie") => LinkOutput::Pie,
                    Some("executable") => LinkOutput::Executable,
                    Some("manifest") => LinkOutput::Manifest,
                    Some(val) => panic!("Invalid or unknown type {}", val),
                    None => unreachable!(),
                };
            }
            _ => panic!(),
        }
    }
    if intree {
        let executable_path = std::env::current_exe()
            .expect("Unable to find executable location; can't use --intree");
        search_paths.push(executable_path.parent().unwrap().to_owned());
    }

    search_paths.push(XLANG_PLUGIN_DIR.into());

    let frontend_paths = find_libraries(&search_paths, &FRONTENDS, "frontend");

    let mut frontend_handles = Vec::new();
    for frontend_path in &frontend_paths {
        frontend_handles.push(Handle::open(frontend_path).expect("couldn't load frontend library"));
    }

    let mut frontends = Vec::new();
    for frontend_handle in &frontend_handles {
        let initializer: FrontendInit =
            unsafe { frontend_handle.function_sym("xlang_frontend_main") }
                .expect("frontend library missing required entry point");
        frontends.push(initializer());
    }

    let codegen_paths = find_libraries(&search_paths, &CODEGENS, "codegen");

    let mut codegen_handles = Vec::new();
    for codegen_path in &codegen_paths {
        codegen_handles.push(Handle::open(codegen_path).expect("couldn't load frontend library"));
    }
    let mut codegens = Vec::new();
    for codegen_handle in &codegen_handles {
        let initializer: CodegenInit = unsafe { codegen_handle.function_sym("xlang_backend_main") }
            .expect("frontend library missing required entry point");
        codegens.push(initializer());
    }

    let xtarget = xlang::targets::Target::from(&target);

    let properties = xlang::targets::properties::get_properties(xtarget.clone());

    let mut file_pairs = Vec::new();

    let mut codegen = None;

    for cg in &mut codegens {
        if cg.target_matches(&xtarget) {
            codegen = Some(cg);
            break;
        }
    }

    let codegen = if let Some(cg) = codegen {
        cg
    } else {
        panic!(
            "couldn't find a backend for target {}",
            Target::from(&xtarget)
        )
    };

    for file in &files {
        let file_view = StringView::new(file);
        let mut frontend = None;
        for fe in &mut frontends {
            if fe.file_matches(file_view) {
                frontend = Some(fe);
                break;
            }
        }
        if let Some(frontend) = frontend {
            let outputfile = if mode < Mode::Link {
                if let Some(ref output) = output {
                    output.clone()
                } else {
                    let mut filename = &**file;
                    if let std::option::Option::Some(offset) = filename.rfind('.') {
                        filename = &filename[..offset];
                    }
                    let mut name = String::from(filename);
                    name += properties.os.obj_suffix;
                    name
                }
            } else {
                let tmpfile = loop {
                    let tmpfile = temp_file::TempFile::with_prefix("lcccobj");

                    match tmpfile {
                        Ok(e) => break e,
                        Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
                        Err(e) => panic!("Cannot create object file: {}", e),
                    }
                };
                let path = tmpfile.path().as_os_str().to_str().unwrap().into();
                tmpfiles.push(tmpfile);
                path
            };
            file_pairs.push((file.clone(), outputfile.clone()));
            frontend.set_file_path(file_view);
            let mut read_adapter =
                ReadAdapter::new(File::open(&file).expect("can't read input file"));
            frontend
                .read_source(DynMut::unsize_mut(&mut read_adapter))
                .unwrap();
            let mut file = xlang::ir::File {
                target: xtarget.clone(),
                root: xlang::ir::Scope::default(),
            };
            frontend.set_target(xtarget.clone());
            frontend.accept_ir(&mut file).unwrap();
            if mode >= Mode::Asm {
                codegen.set_target(xtarget.clone());
                codegen.accept_ir(&mut file).unwrap();
                let mut write_adapter =
                    WriteAdapter::new(File::create(outputfile).expect("Can't create output file"));
                codegen
                    .write_output(DynMut::unsize_mut(&mut write_adapter))
                    .unwrap();
                // TODO: Handle `-S` and write assembly instead of an object
            } else if mode == Mode::Xir {
                todo!()
            }
        } else {
            file_pairs.push((file.clone(), file.clone()));
        }
    }

    if mode == Mode::Link {
        match link_output {
            LinkOutput::Shared => todo!(),
            LinkOutput::Static => {
                let outputs = file_pairs.iter().map(|(_, output)| output);

                match Command::new("ar")
                    .arg("r")
                    .arg("c")
                    .arg("s")
                    .arg(output.as_ref().unwrap())
                    .args(outputs)
                    .status()
                {
                    Ok(_) => todo!(),
                    Err(e) => panic!(
                        "Could not run command ar rcs {}: {}",
                        file_pairs
                            .iter()
                            .map(|(_, output)| output)
                            .map(Deref::deref)
                            .collect::<std::string::String>(),
                        e
                    ),
                }
            }
            LinkOutput::Executable | LinkOutput::Pie => {
                let mut link_args = Vec::<String>::new();

                let mut libdirs = Vec::new();

                for basedir in properties.os.base_dirs {
                    for libdir in properties.libdirs {
                        let targ1 = target.to_string();
                        let targ2 = {
                            let arch = target.arch_name();
                            let os = target.operating_system().map(|o| o.canonical_name());
                            let env = target.environment().map(|o| o.canonical_name());
                            let of = target.object_format().map(|o| o.canonical_name());
                            let mut name = String::from(arch);
                            name.push("-");
                            if let std::option::Option::Some(os) = os {
                                name.push(os);
                                name.push("-"); // Assume, for now, there's at least one more component
                            }

                            if let std::option::Option::Some(env) = env {
                                name.push(env);
                            }

                            if let std::option::Option::Some(of) = of {
                                name.push(of);
                            }

                            name
                        };

                        for &targ in &["", &targ1, &targ2] {
                            let mut path = PathBuf::from(basedir);
                            path.push(libdir);
                            path.push(targ);
                            libdirs.push(path);
                        }
                        for &targ in &[&*targ1, &targ2] {
                            let mut path = PathBuf::from(basedir);
                            path.push(targ);
                            path.push(libdir);
                            libdirs.push(path);
                        }
                    }
                }

                if link_output == LinkOutput::Pie {
                    link_args.push(String::from("-pie"));
                }

                let mut interp = None;

                for libdir in &libdirs {
                    let mut path = libdir.clone();
                    path.push(properties.interp);
                    if path.exists() {
                        interp = Some(path);
                        break;
                    }
                }

                let interp = interp.unwrap();

                let mut startfiles = Vec::new();
                for file in properties.startfiles {
                    let mut found = false;
                    for libdir in &libdirs {
                        let mut path = libdir.clone();
                        path.push(file);
                        if path.exists() {
                            startfiles.push(path);
                            found = true;
                            break;
                        }
                    }

                    #[allow(clippy::manual_assert)]
                    // This will be a proper error message at some point
                    if !found {
                        panic!("Could not find startfile {}", file);
                    }
                }

                let mut endfiles = Vec::new();
                for file in properties.endfiles {
                    let mut found = false;
                    for libdir in &libdirs {
                        let mut path = libdir.clone();
                        path.push(file);
                        if path.exists() {
                            endfiles.push(path);
                            found = true;
                            break;
                        }
                    }

                    #[allow(clippy::manual_assert)]
                    // This will be a proper error message at some point
                    if !found {
                        panic!("Could not find endfile {}", file);
                    }
                }

                match Command::new("ld")
                    .args(&link_args)
                    .arg("-dynamic-linker")
                    .arg(interp)
                    .args(
                        libdirs
                            .iter()
                            .map(|p| String::from("-L") + p.as_os_str().to_str().unwrap()),
                    )
                    .arg("-o")
                    .arg(output.as_deref().unwrap_or("a.out"))
                    .args(&startfiles)
                    .args(file_pairs.iter().map(|(_, s)| s))
                    .arg("--as-needed")
                    .args(
                        properties
                            .default_libs
                            .iter()
                            .map(|s| String::from("-l") + s),
                    )
                    .args(&endfiles)
                    .status()
                {
                    Ok(_) => {}
                    Err(e) => panic!(
                        "Failed to execute command ld {}: {}",
                        link_args
                            .iter()
                            .map(Deref::deref)
                            .collect::<std::string::String>(),
                        e
                    ),
                }
            }
            LinkOutput::Manifest => panic!("Manifest Handled elsewhere"),
        }
    }
}
