use crate::error;
use clap::{
    builder::{Arg, Command},
    ArgAction,
};
use serde::Deserialize;

pub struct Branch {
    // name of the branch to synchronize
    pub name: String,
    // full ref name for the local branch
    pub refname: String,
}

pub struct Options {
    // path to the local repo
    pub repo: String,
    // name of the remote to synchronize from
    pub remote: String,

    // branches to synchronize
    pub branches: Vec<Branch>,

    pub commit_msg_filters: regex::RegexSet,

    pub bootstrap: bool,
    pub uproot: bool,
    pub verbose: bool,
    pub yes: bool,
    pub fetch: bool,
}

#[derive(Deserialize)]
struct YamlCfg {
    repo: Option<String>,
    remote: String,
    // TODO:  add uproot option per branch
    branch: Option<String>,
    branches: Option<Vec<String>>,
    filters: Option<Vec<String>>,
}

pub fn parse_args() -> Result<Options, error::Error> {
    let matches = Command::new("ripit")
        .version("0.9.2")
        .about("Copy commits between git repositories")
        // Configuration
        .arg(
            Arg::new("config_file")
                .required(true)
                .help("Path to configuration file")
                .long_help(
                    "A configuration file containing parameters related to the git \
            repository is required. \
            To create a new one, duplicate and modify config-template.yml, \
            which contains descriptions of all possible options.",
                ),
        )
        // Type of action
        .arg(
            Arg::new("bootstrap")
                .action(ArgAction::SetTrue)
                .long("bootstrap")
                .help("Bootstrap the local repository")
                .long_help(
                    "Before the two repositories can be synchronized, the local \
            repository must be boostrapped, by creating a single commit \
            containing the current state of the remote repository. This \
            is done for each branch to synchronize.",
                ),
        )
        // behavioral features
        .arg(
            Arg::new("uproot")
                .action(ArgAction::SetTrue)
                .short('u')
                .long("uproot")
                .help("Allow commits uprooting")
                .long_help(
                    "By default, a commit with an unknown parent cannot be \
                synchronized. This prevents mistakes and ensures the topology \
                of the sync'ed repository is preserved. \
                However, there are some legitimate cases when this situation can \
                happen, for example when synchronizing a merge commit with \
                one ancestor dating from prior to the bootstrap. \
                In that case, we want to cherry-pick the commits brought by \
                the merge (or in this context, \"uproot\" them). \
                This behavior can be activated with this flag.",
                ),
        )
        .arg(
            Arg::new("nofetch")
                .action(ArgAction::SetTrue)
                .short('F')
                .long("no-fetch")
                .help("Do not fetch private repository")
                .long_help(
                    "By default, ripit will fetch the last commits from the private \
            repository before computing the differences with the local \
            repository. This behavior can be deactivated with this option, \
            which can be useful if the fetch requires authentication which \
            is not handled in ripit.",
                ),
        )
        // common options shared by every action
        .arg(
            Arg::new("quiet")
                .action(ArgAction::SetTrue)
                .short('q')
                .long("quiet")
                .help("Do not print detailed logs of the execution's progress"),
        )
        .arg(
            Arg::new("yes")
                .action(ArgAction::SetTrue)
                .short('y')
                .long("yes")
                .help("Automatic yes to prompts"),
        )
        .get_matches();

    let path = matches.get_one::<String>("config_file").unwrap();
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(error) => {
            return Err(error::Error::FailedOpenCfg {
                path: path.to_owned(),
                error,
            })
        }
    };

    let cfg: YamlCfg = match serde_yaml::from_reader(file) {
        Ok(cfg) => cfg,
        Err(error) => {
            return Err(error::Error::FailedParseCfg {
                path: path.to_owned(),
                error,
            })
        }
    };
    // backward compatibility on legacy branch option
    let branch = cfg.branch.unwrap_or_else(|| "master".to_owned());
    let mut branches = cfg.branches.unwrap_or_default();
    if branches.is_empty() {
        branches.push(branch);
    }
    let branches = branches
        .into_iter()
        .map(|name| {
            let refname = format!("refs/heads/{}", name);
            Branch { name, refname }
        })
        .collect();

    let filters = cfg.filters.unwrap_or_default();
    let commit_msg_filters = match regex::RegexSet::new(filters) {
        Ok(set) => set,
        Err(regex_err) => {
            return Err(error::Error::InvalidConfig {
                field: "filter",
                error: regex_err,
            });
        }
    };

    Ok(Options {
        repo: cfg.repo.unwrap_or_else(|| ".".to_owned()),
        remote: cfg.remote,
        branches,
        commit_msg_filters,

        bootstrap: matches.get_flag("bootstrap"),
        uproot: matches.get_flag("uproot"),
        verbose: !matches.get_flag("quiet"),
        yes: matches.get_flag("yes"),
        fetch: !matches.get_flag("nofetch"),
    })
}
