use crate::tasks::task::Task;
use crate::util::collections::FixedVec;
use crate::util::sync::SpinMutex;

// tegen het advies van Remco in, schrijf ik toch mijn eigen scheduler.

pub struct Scheduler {
    tasks: FixedVec<100, Task>,
    current: usize,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            current: 0,
            tasks: FixedVec::new(),
        }
    }

    fn pick_next(&self) -> Option<usize> {
        // round robin, could be better
        if self.tasks.len() == 0 {
            return None;
        }

        Some((self.current + 1) % self.tasks.len())
    }
}

pub static SCHEDULER: SpinMutex<Scheduler> = SpinMutex::new(Scheduler::new());
