pub fn compare_os_build_version_strings(v1: &str, v2: &str) -> i32 {
    fn parse(v: &str) -> Option<(i32, u8, i32)> {
        // "%d%c%d"
        let mut chars = v.chars().peekable();

        // major
        let mut major = 0i32;
        let mut any = false;
        while let Some(c) = chars.peek() {
            if c.is_ascii_digit() {
                any = true;
                major = major * 10 + (*c as i32 - '0' as i32);
                chars.next();
            } else {
                break;
            }
        }
        if !any {
            return None;
        }

        // minor (single char)
        let minor = chars.next()?;
        let minor = minor.to_ascii_uppercase() as u8;

        // build
        let mut build = 0i32;
        let mut any = false;
        while let Some(c) = chars.peek() {
            if c.is_ascii_digit() {
                any = true;
                build = build * 10 + (*c as i32 - '0' as i32);
                chars.next();
            } else {
                break;
            }
        }
        if !any {
            return None;
        }

        Some((major, minor, build))
    }

    let (m1, c1, b1) = match parse(v1) {
        Some(v) => v,
        None => return -1,
    };

    let (m2, c2, b2) = match parse(v2) {
        Some(v) => v,
        None => return 1,
    };

    if m1 != m2 {
        return m1 - m2;
    }
    if c1 != c2 {
        return c1 as i32 - c2 as i32;
    }
    b1 - b2
}

#[test]
fn test() {
    assert!(compare_os_build_version_strings("13A1", "13A2") < 0);
    assert!(compare_os_build_version_strings("13B1", "13A9") > 0);
    assert!(compare_os_build_version_strings("14A1", "13Z99") > 0);
    assert!(compare_os_build_version_strings("13A10", "13A2") > 0);
}
