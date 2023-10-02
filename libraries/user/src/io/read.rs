pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> crate::io::Result<usize>;
}
