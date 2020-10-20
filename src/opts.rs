use structopt::StructOpt;

/// Command line arguments
#[derive(Default, StructOpt, Debug)]
#[structopt(version=env!("CARGO_PKG_VERSION"), about="Lean and fast taskwarrior web frontend.")]
pub struct Opts {
    /// Report width in characater count, default is unlimited
    #[structopt(default_value, short = "w", long = "width")]
    pub report_width: usize,
}

pub fn get_cl_opts() -> Opts {
    let opts = Opts::from_args();
    log::debug!("{:?}", opts);
    opts
}

pub fn get_default_opts() -> Opts {
    let opts: Opts = Default::default();
    log::debug!("{:?}", opts);
    opts
}
