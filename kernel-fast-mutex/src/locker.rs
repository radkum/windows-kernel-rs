pub trait Locker {
    fn init(&mut self);

    fn lock(&mut self);

    fn unlock(&mut self);
}
