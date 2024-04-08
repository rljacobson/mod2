/*!

A couple of helpers for C-style creation and destruction of heap-allocated objects.

*/


/// Construct a new mutable pointer to a new heap allocated object. This is obviously
/// an unsafe operation. It is up to the user to manually destroy the object and
/// reclaim the memory. The `heap_destroy` macro is provided for this purpose.
#[macro_export]
macro_rules! heap_construct {
    ($expr:expr) => {{
        // Use Box::new to create the object on the heap
        let boxed = Box::new($expr);
        // Convert the Box into a raw pointer, transferring ownership
        // and thus preventing automatic deallocation
        Box::into_raw(boxed)
    }};
}
pub use heap_construct;


/// Destroy a heap allocated object pointed to by a mutable pointer. This is
/// the companion macro to `heap_construct`. It is up to the user to ensure
/// no use after free, no aliasing pointers, and all other safety checks.
#[macro_export]
macro_rules! heap_destroy {
    ($ptr:expr) => {{
        // Assert that the given expression is a mutable raw pointer to prevent misuse.
        // This line does nothing at runtime but ensures type correctness.
        let _ = $ptr as *mut _;

        // Convert the raw pointer back into a Box, taking ownership back
        // and enabling Rust's automatic memory management
        unsafe { Box::from_raw($ptr); }
        // The Box is dropped here, and the memory is deallocated
    }};
}
pub use heap_destroy;
