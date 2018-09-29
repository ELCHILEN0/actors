use crate::*;

#[derive(Clone)]
pub struct TaskWakeHandle(Arc<Mutex<Option<Task>>>);
pub struct TaskWakeGuard(Option<Task>);

impl TaskWakeHandle {
    pub fn empty() -> Self {
        TaskWakeHandle(Arc::new(Mutex::default()))
    }
}

pub trait WakeHandle: Sized {
    fn update(&self);
    fn notify(&self) -> TaskWakeGuard;
}

impl WakeHandle for TaskWakeHandle {
    fn update(&self)
    {
        let mut handle = self.0.lock().unwrap();
        *handle = Some(task::current());
    }

    fn notify(&self) -> TaskWakeGuard
    {
        TaskWakeGuard(self.0.lock().unwrap().take())
    }
}

impl Drop for TaskWakeGuard {
    fn drop(&mut self) {
        if let Some(handle) = self.0.as_ref() {
            handle.notify();
        }
    }
}