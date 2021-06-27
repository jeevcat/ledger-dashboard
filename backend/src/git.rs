use std::path::PathBuf;

use git2::{
    build::RepoBuilder, FetchOptions, IndexAddOption, PushOptions, Remote, Repository, Status,
};
use log::info;

use crate::{config, file_utils};

type Result<T> = std::result::Result<T, git2::Error>;

const BRANCH: &str = "master";
const REMOTE: &str = "origin";

pub fn checkout() -> Repository {
    let url = config::journal_repo_url();
    let repo = clone_or_pull(&url);

    info!("Repo state: {:#?}", repo.state());

    repo
}

pub fn commit_and_push(commit_msg: &str) -> Result<()> {
    let repo = get_repo()?;

    if !is_dirty(&repo)? {
        info!("Nothing to commit!");
        return Ok(());
    }

    commit(&repo, commit_msg)?;
    push(&repo)?;

    Ok(())
}

pub fn get_dirty_files() -> Result<Vec<String>> {
    let repo = get_repo()?;
    let paths = repo
        .statuses(None)?
        .iter()
        .filter_map(|s| match s.status() {
            Status::IGNORED | Status::CURRENT => None,
            _ => s.path().map(|p| p.to_owned()),
        })
        .collect();
    Ok(paths)
}

fn get_repo_path() -> PathBuf {
    let url = config::journal_repo_url();
    file_utils::get_repo_path()
        .unwrap_or_else(|| panic!("Failed to determine file path from repo url: {}", &url))
}

fn get_repo() -> Result<Repository> {
    let path = get_repo_path();
    Repository::discover(&path)
}

fn is_dirty(repo: &Repository) -> Result<bool> {
    Ok(repo
        .statuses(None)?
        .iter()
        .any(|s| !matches!(s.status(), Status::IGNORED | Status::CURRENT)))
}

fn commit(repo: &Repository, commit_msg: &str) -> Result<()> {
    let mut index = repo.index()?;
    index.add_all(["."].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;

    let head = repo.head()?.target().unwrap();
    let head = repo.find_commit(head)?;

    let mut index = repo.index()?;
    let id = index.write_tree()?;

    let tree = repo.find_tree(id)?;
    if tree.is_empty() {
        info!("Nothing to commit");
        return Ok(());
    }

    let sig = repo.signature()?;

    info!(
        "Committing to repo as {} with commit message '{}'",
        sig.name().unwrap_or("Unknown"),
        commit_msg
    );

    repo.commit(Some("HEAD"), &sig, &sig, commit_msg, &tree, &[&head])?;
    Ok(())
}

fn push(repo: &Repository) -> Result<()> {
    let mut remote = get_default_remote(&repo)?;
    do_fetch(&repo, &mut remote)?;
    let remote_branch = get_default_branch(&remote)?;

    info!("Got default branch {}", remote_branch);
    let refs: &[&str] = &[&remote_branch];
    info!(
        "Pushing {} to {}",
        remote_branch,
        remote.name().unwrap_or("Unknown")
    );
    let mut opts = get_push_options();
    remote.push(refs, Some(&mut opts))?;
    Ok(())
}

fn get_auth_headers() -> Option<String> {
    config::journal_repo_credentials().map(|(user, password)| {
        info!("Git credentials supplied. Using basic auth.");
        let auth = base64::encode(format!("{}:{}", user, password));
        format!("AUTHORIZATION: basic {}", auth)
    })
}

fn get_fetch_options<'a>() -> FetchOptions<'a> {
    let mut opts = FetchOptions::new();
    if let Some(headers) = get_auth_headers() {
        opts.custom_headers(&[&headers]);
    }
    opts
}

fn get_push_options<'a>() -> PushOptions<'a> {
    let mut opts = PushOptions::new();
    if let Some(headers) = get_auth_headers() {
        opts.custom_headers(&[&headers]);
    }
    opts
}

fn clone_or_pull(url: &str) -> Repository {
    let path = file_utils::get_repo_path()
        .unwrap_or_else(|| panic!("Failed to determine file path from repo url: {}", &url));

    if let Ok(repo) = Repository::discover(&path) {
        info!("Found repo in {}", path.to_string_lossy());

        {
            let mut remote = get_default_remote(&repo).expect("Couldn't determine default remote");
            let fetch_commit = do_fetch(&repo, &mut remote).expect("Failed to fetch");
            let remote_branch =
                get_default_branch(&remote).expect("Couldn't determine default remote branch");
            info!("Determined default remote branch to be {}", remote_branch);
            do_merge(&repo, &remote_branch, fetch_commit).expect("Couldn't merge remote");
        }

        repo
    } else {
        info!(
            "Cloning journal from {} into {}",
            url,
            path.to_string_lossy()
        );

        let fetch_opts = get_fetch_options();
        RepoBuilder::new()
            .fetch_options(fetch_opts)
            .clone(&url, &path)
            .unwrap()
    }
}

fn get_default_remote(repo: &Repository) -> Result<Remote> {
    // TODO: Can this hardcoded value be discovered automatically?
    repo.find_remote(REMOTE)
}

fn get_default_branch(remote: &Remote) -> Result<String> {
    Ok(remote
        .default_branch()?
        .as_str()
        .unwrap_or(BRANCH)
        .to_owned())
}

fn do_fetch<'a>(
    repo: &'a Repository,
    remote: &mut git2::Remote,
) -> Result<git2::AnnotatedCommit<'a>> {
    let refs: &[&str] = &[];
    let mut fo = get_fetch_options();

    info!("Fetching {} for repo", remote.name().unwrap());
    remote.fetch(refs, Some(&mut fo), None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    repo.reference_to_annotated_commit(&fetch_head)
}

fn do_merge<'a>(
    repo: &'a Repository,
    remote_branch: &str,
    fetch_commit: git2::AnnotatedCommit<'a>,
) -> Result<()> {
    // 1. do a merge analysis
    let analysis = repo.merge_analysis(&[&fetch_commit])?;

    // 2. Do the appopriate merge
    if analysis.0.is_fast_forward() {
        info!("Doing a fast forward");
        // do a fast forward
        let refname = format!("refs/heads/{}", remote_branch);
        match repo.find_reference(remote_branch) {
            Ok(mut r) => {
                fast_forward(repo, &mut r, &fetch_commit)?;
            }
            Err(_) => {
                // The branch doesn't exist so just set the reference to the
                // commit directly. Usually this is because you are pulling
                // into an empty repository.
                repo.reference(
                    &refname,
                    fetch_commit.id(),
                    true,
                    &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
                )?;
                repo.set_head(&refname)?;
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        };
    } else if analysis.0.is_normal() {
        // do a normal merge
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(&repo, &head_commit, &fetch_commit)?;
    } else {
        info!("Nothing to do...");
    }
    Ok(())
}

fn fast_forward(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<()> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    info!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            // For some reason the force is required to make the working directory actually get updated
            // I suspect we should be adding some logic to handle dirty working directory states
            // but this is just an example so maybe not.
            .force(),
    ))?;
    Ok(())
}

fn normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<()> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        info!("Merge conficts detected...");
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    // Do our merge commit and set current branch head to that commit.
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    // Set working tree to match head.
    repo.checkout_head(None)?;
    Ok(())
}
