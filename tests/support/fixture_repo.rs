use std::ffi::OsString;
use std::fs;
use std::path::Path;

use git_helper_core::{GitCommand, GitRepository, RepositoryConfig};
use tempfile::TempDir;

pub struct FixtureRepo {
    pub root: TempDir,
    pub repo: GitRepository,
}

impl FixtureRepo {
    pub fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let repo = GitRepository::open(RepositoryConfig {
            path: root.path().to_path_buf(),
            git_executable: "git".into(),
        })
        .unwrap();
        run(&repo, &["init", "-b", "base"]);
        run(&repo, &["config", "user.name", "Fixture User"]);
        run(&repo, &["config", "user.email", "fixture@example.test"]);
        write_file(root.path(), "README.md", "base\n");
        run(&repo, &["add", "--", "README.md"]);
        run(&repo, &["commit", "-m", "base"]);
        run(&repo, &["update-ref", "refs/remotes/origin/base", "HEAD"]);
        run(&repo, &["switch", "-c", "feature"]);
        Self { root, repo }
    }

    pub fn commit_file(&self, path: &str, content: &str, message: &str) {
        write_file(self.root.path(), path, content);
        run(&self.repo, &["add", "--", path]);
        run(&self.repo, &["commit", "-m", message]);
    }

    pub fn set_base_ref(&self) {
        run(
            &self.repo,
            &["update-ref", "refs/remotes/origin/base", "base"],
        );
    }

    pub fn configure_origin_to_self(&self) {
        let path = self.root.path().to_str().unwrap();
        run(&self.repo, &["config", "remote.origin.url", path]);
    }

    pub fn switch_to_base(&self) {
        run(&self.repo, &["switch", "base"]);
    }

    pub fn switch_to_feature(&self) {
        run(&self.repo, &["switch", "feature"]);
    }

    pub fn branch(&self, name: &str) {
        run(&self.repo, &["switch", "-c", name]);
    }

    pub fn checkout(&self, name: &str) {
        run(&self.repo, &["switch", name]);
    }

    pub fn merge(&self, name: &str, message: &str) {
        run(&self.repo, &["merge", "--no-ff", "-m", message, name]);
    }

    pub fn head(&self) -> String {
        let output = read(&self.repo, &["rev-parse", "HEAD"]);
        String::from_utf8(output).unwrap().trim().to_string()
    }

    pub fn status(&self) -> Vec<u8> {
        read(&self.repo, &["status", "--porcelain=v2", "-z"])
    }

    pub fn write_worktree_file(&self, path: &str, content: &str) {
        write_file(self.root.path(), path, content);
    }

    pub fn stage_file(&self, path: &str) {
        run(&self.repo, &["add", "--", path]);
    }

    pub fn cached_paths(&self) -> Vec<u8> {
        read(&self.repo, &["diff", "--cached", "--name-only", "-z"])
    }

    pub fn head_path(&self, path: &str) -> Vec<u8> {
        let spec = format!("HEAD:{path}");
        read_owned(&self.repo, vec!["show", spec.as_str()])
    }

    pub fn commit_message(&self, commit: &str) -> Vec<u8> {
        let bytes = read_owned(&self.repo, vec!["cat-file", "commit", commit]);
        let Some(separator) = bytes.windows(2).position(|pair| pair == b"\n\n") else {
            return Vec::new();
        };
        bytes[separator + 2..].to_vec()
    }

    pub fn commit_tree(&self, commit: &str) -> String {
        let spec = format!("{commit}^{{tree}}");
        String::from_utf8(read_owned(&self.repo, vec!["rev-parse", &spec]))
            .unwrap()
            .trim()
            .to_string()
    }

    pub fn tree_has_path(&self, path: &str) -> bool {
        let output = read_owned(&self.repo, vec!["ls-tree", "-r", "-z", "HEAD", "--", path]);
        !output.is_empty()
    }

    pub fn add_gitlink(&self, path: &str, object: &str, message: &str) {
        let values = vec![
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            object,
            path,
        ];
        run_owned(&self.repo, values);
        run(&self.repo, &["commit", "-m", message]);
    }
}

fn write_file(root: &Path, path: &str, content: &str) {
    let path = root.join(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content.as_bytes()).unwrap();
}

fn run(repo: &GitRepository, values: &[&str]) {
    let args = values.iter().map(|value| OsString::from(*value)).collect();
    repo.run(GitCommand::write(args)).unwrap();
}

fn run_owned(repo: &GitRepository, values: Vec<&str>) {
    run(repo, &values);
}

fn read(repo: &GitRepository, values: &[&str]) -> Vec<u8> {
    let args = values.iter().map(|value| OsString::from(*value)).collect();
    repo.run(GitCommand::read(args)).unwrap().stdout
}

fn read_owned(repo: &GitRepository, values: Vec<&str>) -> Vec<u8> {
    read(repo, &values)
}
