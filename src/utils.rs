pub fn split_text_into_title_desc(text: &String) -> Option<(Option<String>, Option<String>)> {
    let trimmed = text.trim();
    if trimmed.len() > 0 {
        // TODO: Handle \r\n
        let mut iter = trimmed.splitn(2, '\n');
        let title = iter.next().map(|s| String::from(s.trim()));
        let description = iter.next().map(|s| String::from(s.trim()));

        return Some((title, description));
    }

    return None;
}

pub fn now() -> i64 {
    return match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(_) => panic!("Time before epoch"),
    };
}
