use crate::{config, file_utils};
use git2::{build::RepoBuilder, FetchOptions, Repository};
use log::info;

pub fn checkout() -> Option<()> {
    let url = config::journal_repo_url()?;
    let path = file_utils::get_repo_path()?;

    let repo = if let Ok(repo) = Repository::discover(&path) {
        info!("Found repo in {}", path.to_string_lossy());
        repo
    } else {
        info!("Cloning journal from {} into {}", url, path.to_string_lossy());

        let mut fetch_opts = FetchOptions::new();
        if let Some((user, password)) = config::journal_repo_credentials() {
            info!("Git credentials supplied. Using basic auth to clone.");
            let auth = base64::encode(format!("{}:{}", user, password));
            let auth_header = format!("AUTHORIZATION: basic {}", auth);
            fetch_opts.custom_headers(&[&auth_header]);
        }
        RepoBuilder::new()
            .fetch_options(fetch_opts)
            .clone(&url, &path)
            .unwrap()
    };

    info!("Repo state: {:#?}", repo.state());

    Some(())
}
