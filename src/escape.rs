pub fn unescape(input: &str) -> String {
    let mut iter = input.chars().peekable();
    let mut s = String::new();
    while let Some(c) = iter.next() {
        if c == '\\'
            && let Some(next_c) = iter.next()
        {
            if ('0'..='7').contains(&next_c) {
                let mut digits = format!("{next_c}");
                if let Some(&next_c) = iter.peek()
                    && ('0'..='7').contains(&next_c)
                {
                    iter.next();
                    digits.push(next_c);
                    if let Some(&next_c) = iter.peek()
                        && ('0'..='7').contains(&next_c)
                    {
                        iter.next();
                        digits.push(next_c);
                    }
                }

                s.push(char::from_u32(u32::from_str_radix(&digits, 8).unwrap()).unwrap());
            } else if next_c == 'x' {
                let mut digits = String::new();
                while let Some(&next_c) = iter.peek()
                    && (next_c.is_ascii_digit()
                        || ('a'..='f').contains(&next_c)
                        || ('A'..='F').contains(&next_c))
                {
                    iter.next();
                    digits.push(next_c);
                }

                s.push(char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap());
            } else {
                s.push(match next_c {
                    'a' => 7 as char, // bell
                    'b' => 8 as char, // backspace
                    't' => '\t',
                    'n' => '\n',
                    'v' => 0xb as char, // vertical tab
                    'f' => 0xc as char, // form feed
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    _ => panic!("unsupported escape: {next_c}"),
                });
            }
            continue;
        }

        s.push(c);
    }

    s
}

pub fn escape(input: &str) -> String {
    let mut s = String::new();
    for c in input.chars() {
        if c == '"' || c == '\\' {
            s.push('\\');
            s.push(c);
            continue;
        }

        if (' '..='~').contains(&c) {
            s.push(c);
            continue;
        }

        s += &format!("\\x{:X}", u32::from(c));
    }

    s
}
