use std::io;

pub fn find_matches<W>(content: &str, pattern: &str, mut writer: W) -> io::Result<()>
where
    W: io::Write,
{
    content
        .lines()
        .filter(move |line| line.contains(pattern))
        .try_for_each(|line| writeln!(writer, "{line}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_a_match() {
        let mut result = vec![];
        find_matches("lorem ipsum\ndolor sit amet", "lorem", &mut result).unwrap();
        assert_eq!(result, b"lorem ipsum\n");
    }
}
