use std::ffi::c_void;
use winapi;
use std::ptr;

#[derive(Debug)]
pub struct ExecuteableMemory {
    address: *mut c_void,
    length: usize,
}

pub trait Memory {
    fn address(&mut self) -> *mut c_void;
    fn length(&mut self) -> usize;
}

pub trait Writeable {
    fn write(&mut self, bytes: &Vec<u8>);
}

pub trait Executable {
    fn as_function(&mut self) -> extern "C" fn() -> i64;
}

impl ExecuteableMemory {
    pub fn new(size: usize) -> Self {
        let page_size = get_page_size();
        let size = size.max(page_size);
        let address = unsafe {
            alloc_memory(size, 1)
        };

        ExecuteableMemory {
            address: address,
            length: size,
        }
    }
}

impl Writeable for ExecuteableMemory {
    fn write(&mut self, bytes: &Vec<u8>) {
        let mut write_ptr = self.address() as *mut u8;
        unsafe{
            for byte in bytes {
                ptr::write(write_ptr, *byte);
                write_ptr = write_ptr.add(1);
            }
        }
    }
}

impl Memory for ExecuteableMemory {
    fn address(&mut self) -> *mut c_void {
        self.address
    }

    fn length(&mut self) -> usize {
        self.length
    }
}

impl Executable for ExecuteableMemory {
    fn as_function(&mut self) -> extern "C" fn() -> i64 {
        unsafe {
            core::mem::transmute(self.address)
        }
    }
}

impl Drop for ExecuteableMemory {
    fn drop(&mut self) {
        unsafe {
            free_memory(self.address, self.length);
        }
    }
}

unsafe fn alloc_memory(page_size: usize, num_pages: usize) -> *mut c_void {
    let size = page_size * num_pages;
    let raw_addr: *mut winapi::ctypes::c_void;

    raw_addr = winapi::um::memoryapi::VirtualAlloc(
        ::core::ptr::null_mut(),
        size,
        winapi::um::winnt::MEM_RESERVE | winapi::um::winnt::MEM_COMMIT,
        winapi::um::winnt::PAGE_EXECUTE_READWRITE
    );

    assert_ne!(
        raw_addr, 0 as *mut winapi::ctypes::c_void,
        "Could not allocate memory. Error Code: {:?}",
        winapi::um::errhandlingapi::GetLastError()
    );

    core::mem::transmute(raw_addr)

}

unsafe fn free_memory(address: *mut c_void, _: usize) {
    winapi::um::memoryapi::VirtualFree(address as *mut _, 0, winapi::um::winnt::MEM_RELEASE);
}

fn get_page_size() -> usize {
    unsafe {
        let mut info:  winapi::um::sysinfoapi::SYSTEM_INFO = core::mem::zeroed();
        winapi::um::sysinfoapi::GetSystemInfo(&mut info as  winapi::um::sysinfoapi::LPSYSTEM_INFO);
        info.dwPageSize as usize
    }
}
