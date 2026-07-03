extern crate alloc;
extern crate std;

use std::{
    collections::VecDeque,
    eprintln,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use alloc::boxed::Box;
use log::debug;

type WorkerTask<T> = Box<dyn FnOnce(&mut T) + Send + 'static>;

pub struct LowPriorityWorker<T: Send + 'static> {
    queue: Arc<Mutex<VecDeque<WorkerTask<T>>>>,
    cv: Arc<Condvar>,
    shutdown: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

fn set_thread_name(name: &str) {
    use std::ffi::CString;

    let mut bytes = name.as_bytes();
    if bytes.len() > 15 {
        bytes = &bytes[..15];
    }

    let cstr = CString::new(bytes).unwrap();

    unsafe {
        let rc = libc::pthread_setname_np(libc::pthread_self(), cstr.as_ptr());

        if rc != 0 {
            eprintln!("pthread_setname_np failed: {}", std::io::Error::from_raw_os_error(rc));
        }
    }
}

impl<T: Send + 'static> LowPriorityWorker<T> {
    pub fn new(name: &'static str, state: T) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let cv = Arc::new(Condvar::new());
        let shutdown = Arc::new(AtomicBool::new(false));

        Self {
            thread: Some(thread::spawn({
                let queue = queue.clone();
                let cv = cv.clone();
                let shutdown = shutdown.clone();
                move || Self::worker(name, state, queue, cv, shutdown)
            })),
            queue,
            cv,
            shutdown,
        }
    }

    pub fn enqueue(&self, task: WorkerTask<T>) {
        self.queue.lock().unwrap().push_back(task);
        self.cv.notify_one();
    }

    pub fn enqueue_cb<C: FnOnce(&mut T) + Send + 'static>(&self, task: C) {
        self.enqueue(Box::new(task));
    }

    pub fn barrier(&self) {
        let completed = Arc::new((Mutex::new(false), Condvar::new()));
        let completed2 = completed.clone();

        self.enqueue_cb(move |_| {
            let (lock, cv) = &*completed2;
            *lock.lock().unwrap() = true;
            cv.notify_one();
        });

        let (lock, cv) = &*completed;
        let mut done = lock.lock().unwrap();
        while !*done {
            done = cv.wait(done).unwrap();
        }
    }

    pub fn worker(
        name: &'static str,
        mut state: T,
        queue: Arc<Mutex<VecDeque<WorkerTask<T>>>>,
        cv: Arc<Condvar>,
        shutdown: Arc<AtomicBool>,
    ) {
        let tid = unsafe { libc::syscall(libc::SYS_gettid) } as libc::id_t;
        unsafe {
            libc::setpriority(libc::PRIO_PROCESS, tid, 19);
        }

        set_thread_name(name);

        let mut guard = queue.lock().unwrap();

        loop {
            while guard.is_empty() && !shutdown.load(Ordering::Acquire) {
                guard = cv.wait(guard).unwrap();
            }

            if let Some(task) = guard.pop_front() {
                drop(guard);
                task(&mut state);
                guard = queue.lock().unwrap();
                continue;
            }

            if shutdown.load(Ordering::Acquire) {
                break;
            }
        }

        debug!("{} worker exiting", name);
    }
}

impl<T: Send + 'static> Drop for LowPriorityWorker<T> {
    fn drop(&mut self) {
        debug!("Stopping lowprio worker on drop");

        self.shutdown.store(true, Ordering::Release);
        self.cv.notify_one();
        let _ = self.thread.take().unwrap().join();
    }
}
