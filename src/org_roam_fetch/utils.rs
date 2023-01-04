pub fn remove_quotes_around (s: String) -> String {
    let mut res = s.clone();
    res.remove(0);
    res.pop();
    res
}

pub fn add_quotes_around<T> (s: T) -> String where T: Into<String>{
    let mut res = String::new();
    let s: String = s.into();
    res.push('"');
    res.push_str(s.as_str());
    res.push('"');
    res
}
