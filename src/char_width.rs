unsafe extern "C" {
    fn wcwidth(ch: libc::wchar_t) -> libc::c_int;
}

#[inline]
pub fn wcswidth(s: &str) -> Option<usize> {
    let mut swidth: usize = 0;

    for ch in s.chars() {
        let cwidth = unsafe { wcwidth(ch as libc::wchar_t) };

        if cwidth < 0 {
            return None;
        }

        swidth += cwidth as usize;
    }

    Some(swidth)
}

#[inline]
pub fn wcswidth_ignore_unprintable(s: &str) -> usize {
    let mut swidth: usize = 0;

    for ch in s.chars() {
        let cwidth = unsafe { wcwidth(ch as libc::wchar_t) };

        if cwidth > 0 {
            swidth += cwidth as usize;
        }
    }

    swidth
}

pub trait CharWidth {
    fn char_width(&self) -> Option<usize>;
    fn char_width_ignore_unprintable(&self) -> usize;
}

impl CharWidth for str {
    #[inline]
    fn char_width(&self) -> Option<usize> {
        wcswidth(self)
    }

    #[inline]
    fn char_width_ignore_unprintable(&self) -> usize {
        wcswidth_ignore_unprintable(self)
    }
}

impl CharWidth for String {
    #[inline]
    fn char_width(&self) -> Option<usize> {
        wcswidth(self)
    }

    #[inline]
    fn char_width_ignore_unprintable(&self) -> usize {
        wcswidth_ignore_unprintable(self)
    }
}

impl CharWidth for char {
    #[inline]
    fn char_width(&self) -> Option<usize> {
        let cwidth = unsafe { wcwidth(*self as libc::wchar_t) };

        if cwidth < 0 {
            return None;
        }

        Some(cwidth as usize)
    }

    #[inline]
    fn char_width_ignore_unprintable(&self) -> usize {
        let cwidth = unsafe { wcwidth(*self as libc::wchar_t) };

        if cwidth < 0 {
            return 0;
        }

        cwidth as usize
    }
}
