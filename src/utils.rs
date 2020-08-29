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
