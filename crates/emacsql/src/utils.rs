/// if a given string wrapped with double quotes, remove them, otherwise just
/// return the given string
pub fn maybe_remove_quotes_around<'a, I, O>(s: I) -> O
where
    I: Into<String>,
    O: From<String>,
{
    let s = s.into();
    let mut bs = s.chars();
    let fst = bs.next();
    let lst = bs.last();
    if fst == Some('"') && lst == Some('"') {
        remove_quotes_around(s)
    } else {
        s.into()
    }
}

// notice that you should ensure that `s` is wrapped with quoutes
pub fn remove_quotes_around<'a, I, O>(s: I) -> O
where
    I: Into<String>,
    O: From<String>,
{
    let s = s.into();
    let end = s.len() - 1;
    let s = String::from(&s[1..end]);
    s.into()
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
