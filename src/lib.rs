// runtime.rs / libphia_runtime
#[repr(C)]
pub struct PhiaString {
    pub ptr: *mut u8,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub extern "C" fn phia_new_table() -> *mut Vec<u64> {
    // Aggressively leak vector container to the ABI as an opaque pointer
    Box::into_raw(Box::new(Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C" fn phia_read_file(s: PhiaString) -> PhiaString {
    let path_bytes = unsafe { std::slice::from_raw_parts(s.ptr, s.len) };
    let path_str = std::str::from_utf8(path_bytes).expect("Invalid UTF-8 path");

    // Read and immediately leak to bypass GC
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
pub extern "C" fn phia_print_int(val: i64) {
    println!("{}", val);
}

#[unsafe(no_mangle)]
pub extern "C" fn phia_print_float(val: f64) {
    println!("{}", val);
}
