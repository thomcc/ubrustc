#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;

use rustc_driver::Callbacks;
use rustc_hir::def_id::LocalDefId;
use rustc_interface::interface;
use rustc_middle::mir::BorrowCheckResult;
use rustc_middle::util::Providers;
use rustc_middle::ty::TyCtxt;
use rustc_session::{EarlyErrorHandler, Session};
use rustc_session::config::ErrorOutputType;
use rustc_errors::{ColorConfig, emitter::HumanReadableErrorType};

struct UbrustcCallbacks;

impl rustc_driver::Callbacks for UbrustcCallbacks {
    fn config(&mut self, config: &mut interface::Config) {
        config.override_queries = Some(override_queries);
    }
}

fn override_queries(_: &Session, providers: &mut Providers) {
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
    rustc_driver::install_ice_hook("https://github.com/thomcc/ubrustc/issues/new", |_| ());
    let handler = EarlyErrorHandler::new(ErrorOutputType::HumanReadable(HumanReadableErrorType::Default(ColorConfig::Auto)));
    rustc_driver::init_rustc_env_logger(&handler);
    std::process::exit(rustc_driver::catch_with_exit_code(move || {
        let args: Vec<String> = std::env::args().collect();
        run_compiler(args, &mut UbrustcCallbacks);
    }))
}

fn run_compiler<CB: Callbacks + Send>(mut args: Vec<String>, callbacks: &mut CB) -> ! {
    args.splice(1..1, std::iter::once("--cfg=ubrustc".to_string()));
    std::process::exit(rustc_driver::catch_with_exit_code(move || {
        rustc_driver::RunCompiler::new(&args, callbacks).run()
    }));
}
