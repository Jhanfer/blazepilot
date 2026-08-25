use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

#[derive(Debug, Clone)]
pub struct RepaintSignal {
    inner: Arc<RepaintSignalInner>,
}

#[derive(Debug)]
struct RepaintSignalInner {
    pub needs_repaint: AtomicBool,
    pub repaint_delay_ms: AtomicU64,
}

impl RepaintSignal {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RepaintSignalInner {
                needs_repaint: AtomicBool::new(false),
                repaint_delay_ms: AtomicU64::new(8),
            }),
        }
    }

    pub fn request_repaint_immediate(&self) {
        self.inner.repaint_delay_ms.store(0, Ordering::Relaxed);
        self.inner.needs_repaint.store(true, Ordering::Release);
    }

    pub fn request_repaint_after(&self, ms: u64) {
        self.inner.repaint_delay_ms.store(ms, Ordering::Relaxed);
        self.inner.needs_repaint.store(true, Ordering::Release);
    }

    pub fn take(&self) -> Option<u64> {
        if self.inner.needs_repaint.swap(false, Ordering::Acquire) {
            Some(self.inner.repaint_delay_ms.load(Ordering::Relaxed))
        } else {
            None
        }
    }
}
