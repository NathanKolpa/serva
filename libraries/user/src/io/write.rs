pub trait Write {
    fn write(&mut self, buf: &[u8]) -> crate::io::Result<usize>;
}
