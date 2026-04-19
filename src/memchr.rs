use libc;

pub fn memchr<'a>(haystack: &'a [u8], needle: u8) -> Option<usize> {
    if haystack.is_empty() {
        return None;
    }

    let ptr = unsafe {
        libc::memchr(
            haystack.as_ptr() as *const libc::c_void,
            needle as libc::c_int,
            haystack.len() as libc::size_t,
        )
    };

    if ptr.is_null() {
        return None;
    }

    let index = unsafe { ptr.offset_from(haystack.as_ptr() as *const libc::c_void) };

    Some(index as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(memchr(b"", b'.'), None);
    }

    #[test]
    fn not_found() {
        assert_eq!(memchr(b"foobar", b'z'), None);
    }

    #[test]
    fn find_newline() {
        assert_eq!(memchr(b"foo\n", b'\n'), Some(3));
    }
}
