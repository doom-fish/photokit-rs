use std::mem::ManuallyDrop;

use crate::error::PhotoKitError;
use crate::ffi;

fn is_main_thread() -> bool {
    unsafe { ffi::pthread_main_np() != 0 }
}

pub struct MainThreadCell<T> {
    value: ManuallyDrop<T>,
}

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T> Send for MainThreadCell<T> {}

unsafe impl<T> Sync for MainThreadCell<T> {}

impl<T> MainThreadCell<T> {
    pub fn new(value: T, what: &str) -> Result<Self, PhotoKitError> {
        if is_main_thread() {
            Ok(Self {
                value: ManuallyDrop::new(value),
            })
        } else {
            Err(PhotoKitError::OperationFailed(format!(
                "{what} must be created and used on the main thread"
            )))
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> {
        is_main_thread().then(|| f(&self.value))
    }
}

impl<T> Drop for MainThreadCell<T> {
    fn drop(&mut self) {
        if is_main_thread() {
            unsafe { ManuallyDrop::drop(&mut self.value) };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;

    use super::MainThreadCell;

    struct Counted(Arc<AtomicUsize>);

    impl Drop for Counted {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn cells_cannot_be_created_off_the_main_thread() {
        let drops = Arc::new(AtomicUsize::new(0));
        let value = Counted(Arc::clone(&drops));
        let message = thread::spawn(move || {
            MainThreadCell::new(value, "delegate")
                .map(drop)
                .unwrap_err()
                .to_string()
        })
        .join()
        .unwrap();

        assert!(message.contains("main thread"), "{message}");
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}
