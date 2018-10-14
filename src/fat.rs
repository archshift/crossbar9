use core::fmt;

static mut STR_BUF: [u8; 512] = [0;512];
unsafe fn to_cstring(s: &str) -> *const u8 {
    let mut pos = 0;
    for c in s.bytes() {
        assert!(c < 128);
        STR_BUF[pos] = c;
        pos += 1;
    }
    STR_BUF[pos] = 0;
    &STR_BUF[0]
}

extern {
    fn InitFS() -> bool;
    fn DeinitFS();
    fn FileGetSize() -> usize;
    fn FileOpen(path: *const u8) -> bool;
    fn FileCreate(path: *const u8, truncate: bool) -> bool;
    fn FileRead(buf: *mut u8, size: usize, foffset: usize) -> usize;
    fn FileWrite(buf: *const u8, size: usize, foffset: usize) -> usize;
    fn FileClose();
}

pub struct Fs {
    _private: ()
}

impl Fs {
    pub fn init() -> Self {
        assert!(unsafe { InitFS() });
        Self {_private: ()}
    }

    pub fn _create(&mut self, file: &str) -> File {
        let filename = unsafe { to_cstring(file) };
        assert!(unsafe { FileCreate(filename, true) });
        File {
            _fs: self,
            offset: 0,
            buf: [0; 2048],
            buf_offset: 0
        }
    }

    pub fn open(&mut self, file: &str) -> File {
        let filename = unsafe { to_cstring(file) };
        assert!(unsafe { FileOpen(filename) });
        File {
            _fs: self,
            offset: 0,
            buf: [0; 2048],
            buf_offset: 0
        }
    }
}

impl Drop for Fs {
    fn drop(&mut self) {
        unsafe {
            DeinitFS();
        }
    }
}

pub struct File<'a> {
    _fs: &'a mut Fs,
    offset: usize,
    buf: [u8; 2048],
    buf_offset: usize,
}

impl<'a> File<'a> {
    pub fn _seek(&mut self, offset: usize) {
        self.flush();
        self.offset = offset;
    }

    pub fn write(&mut self, mut bytes: &[u8]) {
        while !bytes.is_empty() {
            let space = self.buf.len() - self.buf_offset;
            let write_len = space.min(bytes.len());
            
            self.buf[self.buf_offset .. self.buf_offset + write_len]
                .copy_from_slice(&bytes[..write_len]);
            
            self.buf_offset += write_len;
            bytes = &bytes[write_len..];

            if self.buf_offset == self.buf.len() {
                self.flush();
            }
        }
    }

    pub fn read(&mut self, dst: &mut [u8]) -> usize {
        let amount = unsafe { FileRead(dst.as_mut_ptr(), dst.len(), self.offset) };
        unsafe {
            let ptr = dst.as_mut_ptr();
            log!("Reading {} of {} bytes from {:#X}: {:02X} {:02X} {:02X} {:02X}",
                amount, dst.len(), self.offset, *ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3));
        }
        self.offset += amount;
        amount
    }

    pub fn flush(&mut self) {
        if self.buf_offset == 0 {
            return
        }
        unsafe {
            self.offset += FileWrite(self.buf.as_ptr(), self.buf.len(), self.offset);
        }
        self.buf_offset = 0;
    }

    pub fn size(&self) -> usize {
        unsafe { FileGetSize() }
    }
}

impl<'a> fmt::Write for File<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl<'a> Drop for File<'a> {
    fn drop(&mut self) {
        unsafe {
            self.flush();
            FileClose();
        }
    }
}
