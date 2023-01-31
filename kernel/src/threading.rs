//! Kernel Multithreading

use core::{arch::global_asm, sync::atomic::AtomicU16};

use crate::{memory::allocator::allocate_of_size, trace};

static NEXT_TID: AtomicU16 = AtomicU16::new(0);

global_asm!(include_str!("asm/thread.S"));

extern "C" {
    fn _switch_stack(new_stack: u64, new_entry: u64);
}

/// A kernel thread
pub struct KThread {
    /// Unique identifier
    pub tid: u16,
    task: Task,
}

/// The task for a KThread
pub struct Task {
    /// The address of the task's stack
    stack: u64,
    /// The address of the task's entry point
    entry: u64,
}

impl Task {
    /// Create a new task that starts at the given memory address
    pub fn new(entry: u64) -> Self {
        // FIXME: Temporary stack location for task
        let stack = 0xffff_ffff_cafe_0000;
        let stack_size = 0x1000;
        allocate_of_size(stack, stack_size, false).unwrap();

        unsafe {
            let stack_arr =
                core::slice::from_raw_parts_mut(stack as *mut u64, (stack_size / 8) as usize);

            stack_arr[(stack_size / 8) as usize - 1] = entry;
        }

        Self {
            stack: stack + stack_size - 7 * 8,
            entry,
        }
    }
}

impl KThread {
    /// Creates a new thread with a unique TID
    pub fn new(task: Task) -> Self {
        let tid = Self::next_tid();
        trace!("New thread with TID {}", tid);
        Self { tid, task }
    }

    /// Switch to this kthread
    pub fn switch(&self) {
        unsafe {
            _switch_stack(self.task.stack, self.task.entry);
        }
    }

    fn next_tid() -> u16 {
        // TODO: Check if tid already exists
        NEXT_TID.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
    }
}
