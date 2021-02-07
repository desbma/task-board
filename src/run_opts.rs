use structopt::StructOpt;

/// Command line arguments
#[derive(Default, StructOpt, Debug)]
#[structopt(version=env!("CARGO_PKG_VERSION"), about="Lean and fast taskwarrior web frontend.")]
pub struct RunOpts {
    /// Read only mode
    #[structopt(short, long)]
    pub dry_run: bool,

    /// Report width in characater count, 0 means unlimited
    #[structopt(default_value, short = "w", long = "width")]
    pub report_width: usize,

    /// Task data dir, if non default
    #[structopt(skip)]
    pub task_data_dir: Option<std::ffi::OsString>,

    /// Temporary test data dir, used for tests
    #[structopt(skip)]
    pub tmp_dir: Option<tempfile::TempDir>,
}

pub fn get_cl_opts() -> RunOpts {
    let mut opts = RunOpts::from_args();
    if let Some(task_data_dir) = std::env::var_os("TASKDATA") {
        opts.task_data_dir = Some(task_data_dir);
    }
    log::debug!("{:?}", opts);
    opts
}

pub fn get_default_opts() -> RunOpts {
    let opts: RunOpts = Default::default();
    log::debug!("{:?}", opts);
    opts
}
