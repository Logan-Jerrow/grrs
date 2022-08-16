use std::io;

pub fn find_matches<W>(content: &str, pattern: &str, mut writer: W) -> io::Result<()>
where
    W: io::Write,
{
    content
        .lines()
        .filter(|line| line.contains(pattern))
        .try_for_each(|line| writeln!(writer, "{line}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(input_content: &str, input_pattern: &str, expected_result: &str) {
        let mut actual_result = vec![];
        find_matches(input_content, input_pattern, &mut actual_result).unwrap();
        assert_eq!(actual_result, expected_result.as_bytes());
    }

    #[test]
    fn find_a_match() {
        check("lorem ipsum\ndolor sit amet", "lorem", "lorem ipsum\n");
    }

    #[test]
    fn find_none() {
        check("lorem ipsum\ndolor sit amet", "@@@", "");
    }
}
