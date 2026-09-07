use std::time::Duration;

pub fn reap_children() {
    let children = direct_children();
    if children.is_empty() {
        return;
    }

    for pid in &children {
        signal(*pid, libc::SIGTERM);
    }

    std::thread::sleep(Duration::from_millis(150));

    for pid in direct_children() {
        signal(pid, libc::SIGKILL);
    }
}

fn signal(pid: i32, sig: i32) {
    if pid <= 1 {
        return;
    }
    unsafe {
        libc::kill(pid, sig);
    }
}

fn direct_children() -> Vec<i32> {
    let mut pids = from_proc_children();
    if pids.is_empty() {
        pids = from_proc_scan();
    }
    pids.sort_unstable();
    pids.dedup();
    pids
}

fn from_proc_children() -> Vec<i32> {
    let Ok(tasks) = std::fs::read_dir("/proc/self/task") else {
        return Vec::new();
    };
    let mut pids = Vec::new();
    for task in tasks.flatten() {
        let Ok(text) = std::fs::read_to_string(task.path().join("children")) else {
            continue;
        };
        pids.extend(
            text.split_ascii_whitespace()
                .filter_map(|p| p.parse::<i32>().ok()),
        );
    }
    pids
}

fn from_proc_scan() -> Vec<i32> {
    let me = std::process::id() as i32;
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    let mut pids = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(pid) = name.to_str().and_then(|n| n.parse::<i32>().ok()) else {
            continue;
        };
        let Ok(stat) = std::fs::read_to_string(entry.path().join("stat")) else {
            continue;
        };
        if parent_of(&stat) == Some(me) {
            pids.push(pid);
        }
    }
    pids
}

fn parent_of(stat: &str) -> Option<i32> {
    let tail = &stat[stat.rfind(')')? + 1..];
    tail.split_ascii_whitespace().nth(1)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::parent_of;

    #[test]
    fn reads_ppid_past_a_normal_name() {
        assert_eq!(parent_of("1234 (star) S 987 1234 1234 0"), Some(987));
    }

    #[test]
    fn reads_ppid_past_a_name_containing_spaces_and_parens() {
        assert_eq!(parent_of("77 (WebKit (web) S 42 77 77 0"), Some(42));
    }

    #[test]
    fn rejects_a_line_with_no_name() {
        assert_eq!(parent_of("garbage"), None);
    }
}
