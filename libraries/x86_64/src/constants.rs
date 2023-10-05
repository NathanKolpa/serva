pub const MIN_STACK_SIZE: usize = if cfg!(debug_assertions) {
    1024 * 16
} else {
    1024 * 4
};
