#[inline]
pub fn is_sgml(mime_type: &str) -> bool {
    mime_type == "text/html" || mime_type.ends_with("+xml")
}

#[inline]
pub fn is_json(mime_type: &str) -> bool {
    if mime_type == "application/json" || mime_type.ends_with("+json") {
        return true;
    }

    if let Some(index) = mime_type.find('/') {
        let tail = &mime_type[index + 1..];

        return tail.starts_with("json+");
    }

    false
}
