use log::*;
use std::cell::RefCell;
use std::thread;

const CPU_UNKNOWN: usize = 0xffffffff;

thread_local!(static THREAD_CPU_ID: RefCell<usize> = RefCell::new(CPU_UNKNOWN));

pub struct CoreTracker {
    label: String,
    initial_cpu_id: usize,
    last_thread_id: RefCell<thread::ThreadId>,
    last_cpu_id: RefCell<usize>,
}

impl CoreTracker {
    pub fn new(label: String) -> Self {
        let mut cpu_id: usize = 0;

        THREAD_CPU_ID.with(|tci| cpu_id = *tci.borrow());
        debug!("CoreTracker: on core {}: {}", cpu_id, label);

        let current_thread_id = thread::current().id();
        Self {
            label,
            initial_cpu_id: cpu_id,
            last_thread_id: RefCell::new(current_thread_id),
            last_cpu_id: RefCell::new(cpu_id),
        }
    }

    pub fn touch(&self) {
        let current_thread_id = thread::current().id();
        if current_thread_id != *(self.last_thread_id.borrow()) {
            let current_cpu_id = Self::get_cpu_id();
            let old_location = *(self.last_cpu_id.borrow());
            debug!(
                "Worker hop from core {} to core {}: {} (initial {})",
                old_location, current_cpu_id, self.label, self.initial_cpu_id
            );

            *self.last_thread_id.borrow_mut() = current_thread_id;
            *self.last_cpu_id.borrow_mut() = current_cpu_id;
        }
    }

    pub fn register_thread(cpu_id: usize) {
        THREAD_CPU_ID.with(move |tci| *tci.borrow_mut() = cpu_id);
    }

    pub fn get_cpu_id() -> usize {
        let mut cpu_id: usize = 0;
        THREAD_CPU_ID.with(|tci| cpu_id = *tci.borrow());
        cpu_id
    }
}

unsafe impl Send for CoreTracker {}
unsafe impl Sync for CoreTracker {}
