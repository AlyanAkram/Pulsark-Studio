use std::path::PathBuf;
use std::fs;

#[derive(Clone, Debug)]
pub struct FileNode {
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

pub fn build_tree(path: &PathBuf) -> Vec<FileNode> {
    let mut nodes = Vec::new();

    let Ok(entries) = fs::read_dir(path) else {
        return nodes;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Skip heavy/system folders
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == "node_modules"
                || name == ".git"
                || name == "target"
                || name == ".idea"
                || name == ".vscode"
            {
                continue;
            }
        }

        let is_dir = path.is_dir();

        let children = if is_dir {
            build_tree(&path)
        } else {
            Vec::new()
        };

        nodes.push(FileNode {
            path,
            is_dir,
            children,
        });
    }

    nodes
}