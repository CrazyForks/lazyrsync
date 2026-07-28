use anyhow::{anyhow, Result};

use crate::profile::{Profile, Task};
use crate::rsync;

pub fn list(profiles: &[Profile]) {
    if profiles.is_empty() {
        println!("No profiles. Add one in the TUI (run `lazyrsync`).");
    }
    for p in profiles {
        println!("{}", p.name);
        for t in &p.tasks {
            println!("  {}", t.id);
            println!("      {}", rsync::resolved_command(t, false));
        }
        println!();
    }
}

fn select<'a>(profiles: &'a [Profile], target: &str) -> Result<Vec<&'a Task>> {
    if profiles.is_empty() {
        return Err(anyhow!(
            "no profiles configured (add one in the TUI: lazyrsync)"
        ));
    }
    let (name, task_id) = match target.split_once('/') {
        Some((n, t)) => (n, Some(t)),
        None => (target, None),
    };
    let profile = profiles.iter().find(|p| p.name == name).ok_or_else(|| {
        let names: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();
        anyhow!("no profile named '{name}' (have: {})", names.join(", "))
    })?;
    match task_id {
        None => Ok(profile.tasks.iter().collect()),
        Some(id) => profile
            .tasks
            .iter()
            .find(|t| t.id == id)
            .map(|t| vec![t])
            .ok_or_else(|| {
                let ids: Vec<&str> = profile.tasks.iter().map(|t| t.id.as_str()).collect();
                anyhow!(
                    "no task '{id}' in profile '{name}' (have: {})",
                    ids.join(", ")
                )
            }),
    }
}

pub fn run(profiles: &[Profile], target: &str, dry_run: bool, yes: bool) -> Result<i32> {
    let _ = (dry_run, yes);
    let tasks = match select(profiles, target) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {e}");
            return Ok(2);
        }
    };
    let _ = tasks;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Task;

    fn task(id: &str, source: &str, dest: &str) -> Task {
        let mut t = Task::new(format!("{id} label"), source, dest);
        t.id = id.to_string();
        t
    }

    fn profile(tasks: Vec<Task>) -> Profile {
        let mut p = Profile::new("backups");
        p.tasks = tasks;
        p
    }

    #[test]
    fn unknown_profile_returns_2() {
        let ps = vec![profile(vec![task("photos", "/src/", "/dst/")])];
        assert_eq!(run(&ps, "nope", false, false).unwrap(), 2);
        assert!(select(&ps, "nope")
            .unwrap_err()
            .to_string()
            .contains("backups"));
    }

    #[test]
    fn unknown_task_id_returns_2() {
        let ps = vec![profile(vec![task("photos", "/src/", "/dst/")])];
        assert_eq!(run(&ps, "backups/nope", false, false).unwrap(), 2);
        assert!(select(&ps, "backups/nope")
            .unwrap_err()
            .to_string()
            .contains("photos"));
    }

    #[test]
    fn empty_config_says_so_instead_of_dangling() {
        let err = select(&[], "backups").unwrap_err().to_string();
        assert!(err.contains("no profiles configured"), "got: {err}");
    }

    #[test]
    fn whole_profile_selects_every_task() {
        let ps = vec![profile(vec![
            task("photos", "/src/", "/dst/"),
            task("music", "/src2/", "/dst2/"),
        ])];
        let picked = select(&ps, "backups").unwrap();
        let ids: Vec<&str> = picked.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, vec!["photos", "music"]);
    }

    #[test]
    fn task_target_selects_only_that_task() {
        let ps = vec![profile(vec![
            task("photos", "/src/", "/dst/"),
            task("music", "/src2/", "/dst2/"),
        ])];
        let picked = select(&ps, "backups/music").unwrap();
        assert_eq!(picked.len(), 1);
        assert_eq!(picked[0].id, "music");
    }
}
