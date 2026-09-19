#[repr(C)]
pub struct PhiaString {
    pub ptr: *mut u8,
    pub len: usize,
}

#[repr(C)]
pub struct PhiaTable {
    pub ptr: *mut u64,
    pub cap: usize,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub extern "C" fn phia_new_table() -> *mut PhiaTable {
    let vec: Vec<u64> = Vec::new();
    let table = Box::new(PhiaTable {
        ptr: vec.as_ptr() as *mut u64,
        cap: vec.capacity(),
        len: vec.len(),
    });
    // Leak the Vec buffer so Rust doesn't free it when the function exits
    std::mem::forget(vec);
    Box::into_raw(table)
}

#[unsafe(no_mangle)]
pub extern "C" fn phia_table_ensure_capacity(table: *mut PhiaTable, limit: usize) {
    let t = unsafe { &mut *table };
    let required = limit + 1;

    if required > t.cap {
        // Reconstruct the Vec from raw parts to use Rust's native allocator
        let mut vec = unsafe { Vec::from_raw_parts(t.ptr, t.len, t.cap) };

        vec.reserve(required.saturating_sub(t.cap));

        // Because LLVM writes directly to memory behind Rust's back,
        // we must force the length up so the next reallocation copies the data.
        unsafe { vec.set_len(required); }

        // Update our C-ABI struct with the new buffer location
        t.ptr = vec.as_mut_ptr();
        t.cap = vec.capacity();
        t.len = required;

        // Leak it again!
        std::mem::forget(vec);
    } else if required > t.len {
        t.len = required;
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
