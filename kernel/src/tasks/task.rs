pub enum TaskState {
    Waiting,
    Blocked,
    Executing,
}

pub struct Task {
    id: usize,
    state: TaskState,
}

impl Task {}
