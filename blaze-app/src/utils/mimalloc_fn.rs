use libmimalloc_sys::{mi_collect, mi_option_set};

pub unsafe fn set_mi_option() {
    unsafe {
        // purge_delay a 0
        mi_option_set(10, 0);
        // purge_decommits a 1
        mi_option_set(11, 1);
    }
}

pub unsafe fn free_mi() {
    unsafe {
        mi_collect(true);
    }
}
