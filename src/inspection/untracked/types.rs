/// Extensions the Files-diff highlighter knows. Kept in sync with
/// `ui/app/Private/files-diff/highlight.ts` so "unknown type" filtering agrees.
const KNOWN_EXTENSIONS: &[&str] = &[
    "ts", "mts", "cts", "tsx", "js", "mjs", "cjs", "jsx", "rs", "css", "html", "htm", "svg",
    "xml", "json", "md", "toml", "sh", "bash", "py", "yml", "yaml",
];

pub(super) fn is_known_type(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let Some(dot) = name.rfind('.') else {
        return false;
    };
    if dot == 0 {
        return false;
    }
    let extension = &name[dot + 1..];
    KNOWN_EXTENSIONS
        .iter()
        .any(|known| known.eq_ignore_ascii_case(extension))
}
