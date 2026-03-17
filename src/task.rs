mod harvest;
mod spawn;

pub use harvest::Harvest;
pub use spawn::Spawn;

pub enum Status {
    Pending,
    Complete,
}

pub trait Task {
    fn poll(&mut self) -> Status;
}
