#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;

use rustc_driver::Callbacks;
use rustc_hir::def_id::LocalDefId;
use rustc_interface::interface;
use rustc_middle::mir::BorrowCheckResult;
use rustc_middle::ty::query::{ExternProviders, Providers};
use rustc_middle::ty::TyCtxt;
use rustc_session::Session;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

struct UbrustcCallbacks;

impl rustc_driver::Callbacks for UbrustcCallbacks {
    fn config(&mut self, config: &mut interface::Config) {
        config.override_queries = Some(override_queries);
    }
}

fn override_queries(_: &Session, providers: &mut Providers, _: &mut ExternProviders) {
    providers.mir_borrowck = not_a_borrowchecker;
}

fn not_a_borrowchecker(cx: TyCtxt<'_>, _: LocalDefId) -> &'_ BorrowCheckResult<'_> {
    cx.arena.alloc(BorrowCheckResult {
        concrete_opaque_types: Default::default(),
        closure_requirements: None,
        used_mut_upvars: Default::default(),
        tainted_by_errors: None,
    })
}

fn main() {
    rustc_driver::install_ice_hook();
    rustc_driver::init_rustc_env_logger();
    std::process::exit(rustc_driver::catch_with_exit_code(move || {
        let orig_args: Vec<String> = std::env::args().collect();

        let sysroot_arg = arg_value(&orig_args, "--sysroot");
        let have_sysroot_arg = sysroot_arg.is_some();
        let sysroot = sysroot_arg
            .map(ToString::to_string)
            .or_else(|| sysroot().map(|p| p.display().to_string()))
            .expect("Failed to find sysroot");

        let mut args: Vec<String> = orig_args.clone();

        if !have_sysroot_arg {
            args.extend(["--sysroot".to_string(), sysroot.to_string()]);
        }

        let our_exe_filename = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_stem().map(ToOwned::to_owned))
            .unwrap_or_else(|| "ubrustc".into());

        // This is probably wrong.
        let wrapper_mode = orig_args
            .get(1)
            .map(std::path::Path::new)
            .and_then(std::path::Path::file_stem)
            .map_or(false, |name| {
                name == our_exe_filename || name == "ubrustc" || name == "rustc"
            });

        if wrapper_mode {
            args.remove(1);
        }
        run_compiler(args, &mut UbrustcCallbacks);
    }))
}

fn arg_value<'a, T: AsRef<str>>(args: &'a [T], find_arg: &str) -> Option<&'a str> {
    let mut args = args.iter().map(|s| s.as_ref());
    while let Some(arg) = args.next() {
        let mut arg = arg.splitn(2, '=');
        if arg.next() != Some(find_arg) {
            continue;
        }

        if let Some(a) = arg.next().or_else(|| args.next()) {
            return Some(a);
        }
    }
    None
}

fn sysroot() -> Option<PathBuf> {
    fn rustup_sysroot<H: ?Sized + AsRef<OsStr>, T: ?Sized + AsRef<Path>>(
        home: &H,
        toolchain: &T,
    ) -> PathBuf {
        let mut path = PathBuf::from(home);
        path.push("toolchains");
        path.push(toolchain);
        path
    }
    fn runtime_rustup_sysroot() -> Option<PathBuf> {
        let home = std::env::var_os("RUSTUP_HOME")?;
        let toolchain = std::env::var_os("RUSTUP_TOOLCHAIN")?;
        Some(rustup_sysroot(&home, &toolchain))
    }
    fn compiletime_rustup_sysroot() -> Option<PathBuf> {
        let home: &str = option_env!("RUSTUP_HOME")?;
        let toolchain: &str = option_env!("RUSTUP_TOOLCHAIN")?;
        Some(rustup_sysroot(&home, &toolchain))
    }
    fn rustc_on_path_sysroot() -> Option<PathBuf> {
        std::process::Command::new("rustc")
            .arg("--print=sysroot")
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|s| PathBuf::from(s.trim()))
    }
    runtime_rustup_sysroot()
        .or_else(rustc_on_path_sysroot)
        .or_else(compiletime_rustup_sysroot)
}

fn run_compiler<CB: Callbacks + Send>(mut args: Vec<String>, callbacks: &mut CB) -> ! {
    args.splice(1..1, std::iter::once("--cfg=ubrustc".to_string()));
    std::process::exit(rustc_driver::catch_with_exit_code(move || {
        rustc_driver::RunCompiler::new(&args, callbacks).run()
    }));
}
