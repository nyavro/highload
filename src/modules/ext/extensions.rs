use log::warn;

pub trait ResultExt<T, E: std::fmt::Display> {
    fn warn(self, message: String) -> Option<T>;
}

impl<T, E: std::fmt::Display> ResultExt<T, E> for Result<T, E> {
    fn warn(self, message: String) -> Option<T> {
        if let Err(e) = self {
            warn!("Error {}. {}", e, message);
            None 
        } else {
            self.ok()
        }
    }
}