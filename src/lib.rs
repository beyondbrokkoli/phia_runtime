#[repr(C)]
pub struct PhiaString {
    pub ptr: *mut u8,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub extern "C" fn phia_new_table() -> *mut Vec<u64> {
    Box::into_raw(Box::new(Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C" fn phia_table_ensure_capacity(table: *mut Vec<u64>, limit: usize) {
    let vec = unsafe { &mut *table };
    let required_cap = limit + 1;

    if required_cap > vec.capacity() {
        let additional = required_cap.saturating_sub(vec.capacity());
        vec.reserve(additional);
    }

    // CRITICAL: Since LLVM writes directly via GEP without updating len,
    // we MUST bump len before the NEXT reserve(), otherwise Rust's allocator
    // won't copy the old elements to the new buffer.
    if required_cap > vec.len() {
        unsafe { vec.set_len(required_cap); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn phia_concat(a: PhiaString, b: PhiaString) -> PhiaString {
    let mut buf = Vec::with_capacity(a.len + b.len);
    unsafe {
        let a_slice = std::slice::from_raw_parts(a.ptr, a.len);
        let b_slice = std::slice::from_raw_parts(b.ptr, b.len);
        buf.extend_from_slice(a_slice);
        buf.extend_from_slice(b_slice);
    }

    let leaked = buf.into_boxed_slice();
    PhiaString {
        len: leaked.len(),
        ptr: Box::into_raw(leaked) as *mut u8,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn phia_read_file(s: PhiaString) -> PhiaString {
    let path_bytes = unsafe { std::slice::from_raw_parts(s.ptr, s.len) };
    let path_str = std::str::from_utf8(path_bytes).expect("Invalid UTF-8");
    let content = std::fs::read(path_str).expect("Failed to read file").into_boxed_slice();
    PhiaString {
        len: content.len(),
        ptr: Box::into_raw(content) as *mut u8,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn phia_get_byte(s: PhiaString, index: usize) -> u8 {
    unsafe { *s.ptr.add(index) }
}

#[unsafe(no_mangle)]
pub extern "C" fn phia_print_int(val: i64) { println!("{}", val); }

#[unsafe(no_mangle)]
pub extern "C" fn phia_print_float(val: f64) { println!("{}", val); }

#[unsafe(no_mangle)]
pub extern "C" fn phia_print_bool(val: bool) { println!("{}", val); }

#[unsafe(no_mangle)]
pub extern "C" fn phia_print_string(val: PhiaString) {
    let bytes = unsafe { std::slice::from_raw_parts(val.ptr, val.len) };
    if let Ok(s) = std::str::from_utf8(bytes) {
        println!("{}", s);
    }
}
