use serde::Serialize;

/// One parsed `phase: N% (done/total)` fragment from `git fetch --progress`.
/// Serialized to the UI as the `fetch-progress` event payload.
#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct FetchProgress {
    pub phase: String,
    pub done: u64,
    pub total: u64,
}

const PHASES: [&str; 5] = [
    "Enumerating objects",
    "Counting objects",
    "Compressing objects",
    "Receiving objects",
    "Resolving deltas",
];

const TAIL_LINES: usize = 20;

/// Git rewrites a meter in place with `\r` and ends it with `\n`, so fragments
/// split on both. Fragments that are not progress keep their text for the
/// error tail; progress fragments must not pollute a failure message.
pub(super) struct ProgressFeed {
    pending: Vec<u8>,
    tail: Vec<String>,
}

impl ProgressFeed {
    pub(super) fn new() -> Self {
        Self {
            pending: Vec::new(),
            tail: Vec::new(),
        }
    }

    pub(super) fn push(&mut self, chunk: &[u8], on_progress: &mut dyn FnMut(FetchProgress)) {
        self.pending.extend_from_slice(chunk);
        while let Some(at) = self
            .pending
            .iter()
            .position(|byte| matches!(byte, b'\r' | b'\n'))
        {
            let fragment: Vec<u8> = self.pending.drain(..at).collect();
            self.pending.remove(0);
            self.digest(&fragment, on_progress);
        }
    }

    pub(super) fn tail_text(&self) -> String {
        self.tail.join("\n")
    }

    fn digest(&mut self, fragment: &[u8], on_progress: &mut dyn FnMut(FetchProgress)) {
        let text = String::from_utf8_lossy(fragment);
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        if let Some(progress) = parse_progress(text) {
            on_progress(progress);
            return;
        }
        self.tail.push(text.to_string());
        if self.tail.len() > TAIL_LINES {
            self.tail.remove(0);
        }
    }
}

/// `From https://…` survives as `None`: its "phase" would be `From https`,
/// which is not a progress phase.
fn parse_progress(fragment: &str) -> Option<FetchProgress> {
    let line = fragment.strip_prefix("remote: ").unwrap_or(fragment);
    let (phase, rest) = line.split_once(':')?;
    let phase = phase.trim();
    if !PHASES.contains(&phase) {
        return None;
    }
    let open = rest.find('(')?;
    let close = rest[open..].find(')')? + open;
    let (done, total) = rest[open + 1..close].split_once('/')?;
    Some(FetchProgress {
        phase: phase.to_string(),
        done: done.trim().parse().ok()?,
        total: total.trim().parse().ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_progress, ProgressFeed};

    #[test]
    fn parses_a_local_progress_line() {
        let progress =
            parse_progress("Receiving objects:  45% (123/273), 1.10 MiB | 2.00 MiB/s").unwrap();
        assert_eq!(progress.phase, "Receiving objects");
        assert_eq!(progress.done, 123);
        assert_eq!(progress.total, 273);
    }

    #[test]
    fn parses_a_remote_side_progress_line() {
        let progress = parse_progress("remote: Compressing objects: 100% (8/8), done.").unwrap();
        assert_eq!(progress.phase, "Compressing objects");
        assert_eq!(progress.done, 8);
        assert_eq!(progress.total, 8);
    }

    #[test]
    fn ignores_lines_without_a_progress_counter() {
        assert!(parse_progress("remote: Enumerating objects: 5, done.").is_none());
        assert!(parse_progress("From https://example.test/repo").is_none());
        assert!(parse_progress("fatal: unable to access 'https://x': gone").is_none());
        assert!(parse_progress("").is_none());
    }

    #[test]
    fn feed_reports_progress_across_carriage_return_updates() {
        let mut feed = ProgressFeed::new();
        let mut events = Vec::new();
        feed.push(
            b"Receiving objects:  10% (10/100)\rReceiving obj",
            &mut |event| events.push(event),
        );
        feed.push(
            b"ects:  20% (20/100), done.\nFrom https://example.test/repo\n",
            &mut |event| events.push(event),
        );
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].done, 10);
        assert_eq!(events[1].done, 20);
        assert_eq!(feed.tail_text(), "From https://example.test/repo");
    }

    #[test]
    fn feed_keeps_only_the_last_tail_lines() {
        let mut feed = ProgressFeed::new();
        for index in 0..30 {
            feed.push(format!("line {index}\n").as_bytes(), &mut |_| {});
        }
        let tail = feed.tail_text();
        assert!(tail.starts_with("line 10\n"));
        assert_eq!(tail.lines().count(), 20);
    }
}
