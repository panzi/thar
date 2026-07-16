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

pub fn crop(text: &str, start_width: usize, end_width: usize) -> &str {
    if end_width <= start_width {
        return "";
    }

    let mut chars = text.char_indices();
    let mut start_index = text.len();
    let mut end_index = text.len();

    let mut width = 0;

    if start_width == 0 {
        start_index = 0;
    } else {
        while let Some((index, c)) = chars.next() {
            width += c.char_width_ignore_unprintable();
            if width > start_width {
                start_index = index;
                break;
            }
        }
    }

    if width >= end_width {
        end_index = start_index;
    } else {
        for (index, c) in chars {
            if width >= end_width {
                end_index = index;
                break;
            }
            width += c.char_width_ignore_unprintable();
        }
    }

    &text[start_index..end_index]
}
