use crate::profile::Profile;
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
