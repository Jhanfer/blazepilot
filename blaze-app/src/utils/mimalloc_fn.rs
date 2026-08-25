use libmimalloc_sys::{mi_collect, mi_option_set};

pub unsafe fn set_mi_option() {
    unsafe {
        mi_option_set(5, 1);
        mi_option_set(15, 0);
    }
}

pub unsafe fn free_mi() {
    unsafe {
        mi_collect(true);
    }
}
