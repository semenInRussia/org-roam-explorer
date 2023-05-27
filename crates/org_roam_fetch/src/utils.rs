/// notice that you should ensure that `s` is wrapped with quoutes
pub fn remove_quotes_around<'a>(s: &'a String) -> &'a str {
    let end = s.len()-1;
    &s[1..end]
}

pub fn add_quotes_around<T>(s: T) -> String
where
    T: Into<String>,
{
    let mut res = String::new();
    res.push('"');
    res.push_str(s.into().as_str());
    res.push('"');
    res
}
