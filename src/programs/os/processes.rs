use core::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use core::num::NonZeroU64;
use core::cmp::PartialOrd;

use mem;
use alloc::collections::BinaryHeap;

static mut next_pid: Pid = Pid::new(3);
static mut next_tid: Tid = Tid::new(3);
type Pid = NonZeroU64;
type Tid = NonZeroU64;

fn acquire_pid() -> Pid {
    unsafe {
        let out = next_pid;
        next_pid += 1;
        out
    }
}

fn acquire_tid() -> Tid {
    unsafe {
        let out = next_tid;
        next_tid += 1;
        out
    }
}

struct ThreadCtx {
    regs: [u32; 16],
    cpsr: u32,
}

struct Thread {
    tid: Tid,
    stack: mem::Array<u8>,
    saved_ctx: Option<ThreadCtx>,
}

impl Thread {
    fn new() {
        Self {
            tid: acquire_tid(),
            stack: mem::Array::aligned_new(size: 8192, alignment: 4),
            saved_ctx: None
        }
    }
}

struct Process {
    pid: Pid,
    threads: [ Option<Thread>; 4 ],
}

impl Process {
    fn new() -> Self {
        Self {
            pid: acquire_pid(),
            threads: [ None; 4 ]
        }
    }
}

struct PriorityTid(Tid);
impl PartialOrd for Tid {
    fn partial_cmp(&self, other: Option<Self>) {

    }
}

struct Scheduler<'a> {
    timer: Timer<'a>,
    priorities: BinaryHeap<PriorityTid>,
}
