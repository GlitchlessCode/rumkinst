use std::{
    borrow::Cow,
    sync::{LazyLock, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use indicatif_log_bridge::LogWrapper;
use log::{LevelFilter, Log};

static PROGRESS_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::with_template("[{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>3}/{len:3} {msg}")
        .expect("should be able to unwrap main ProgressStyle")
});

static CENTRAL_PROGRESS_WRAPPER: OnceLock<CentralProgressWrapper> = OnceLock::new();

struct CentralProgressWrapper {
    multi: MultiProgress,
    current: RwLock<Option<ProgressBar>>,
}

impl CentralProgressWrapper {
    fn get_current(&self) -> RwLockReadGuard<Option<ProgressBar>> {
        self.current
            .read()
            .expect("current progressbar rwlock is poisoned")
    }

    fn get_current_mut(&self) -> RwLockWriteGuard<Option<ProgressBar>> {
        self.current
            .write()
            .expect("current progressbar rwlock is poisoned")
    }
}

fn get_wrapper() -> &'static CentralProgressWrapper {
    CENTRAL_PROGRESS_WRAPPER
        .get()
        .expect("log wrapper not initialized, make sure to call setup_log_wrapper first")
}

pub fn setup_log_wrapper(logger: impl Log + 'static, filter: LevelFilter) {
    let multi = MultiProgress::new();

    LogWrapper::new(multi.clone(), logger)
        .try_init()
        .expect("should have successfully initialized log wrapper");
    log::set_max_level(filter);

    if CENTRAL_PROGRESS_WRAPPER
        .set(CentralProgressWrapper {
            multi,
            current: RwLock::new(None),
        })
        .is_err()
    {
        panic!("setup_log_wrapper should only be called once");
    }
}

pub fn progress_wrapper<F, R>(length: u64, logic: F) -> R
where
    F: Fn() -> R,
{
    let wrapper = get_wrapper();
    let mut current_pb = wrapper.get_current_mut();

    if current_pb.is_some() {
        panic!("progress bar already in use, cannot initialize another");
    }

    let pb = wrapper
        .multi
        .add(ProgressBar::new(length))
        .with_style(PROGRESS_STYLE.clone());

    current_pb.replace(pb.clone());
    drop(current_pb);

    let result = logic();

    let _ = wrapper.get_current_mut().take();
    pb.finish();

    result
}

pub fn increment_progress(amount: u64) {
    if let Some(pb) = &*get_wrapper().get_current() {
        pb.inc(amount);
    }
}

pub fn set_progress_message<S: Into<Cow<'static, str>>>(msg: S) {
    if let Some(pb) = &*get_wrapper().get_current() {
        pb.set_message(msg);
    }
}
