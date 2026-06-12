use hyrmir_lib::vcs::VCS;

pub struct Git;

impl Git {
    pub fn new() -> Self { Self }
}

impl VCS for Git {}