use std::fmt;
use bunt::{
    write, writeln, print, println,
    termcolor::Buffer,
};


fn buf() -> Buffer {
    Buffer::ansi()
}
fn raw_str(buf: &Buffer) -> &str {
    std::str::from_utf8(buf.as_slice()).expect("test produced non-UTF8 string")
}

// Helper macro that checks the output of a `write!` invocation against an
// expected string.
macro_rules! check {
    ($expected:literal == $($t:tt)*) => {{
        let mut buf = buf();
        write!(buf, $($t)*).expect("failed to write to buffer in test");
        let actual = raw_str(&buf);
        if $expected != actual {
            panic!(
                "incorrect output for `write!({})`:\n   \
                    expected: {:?} ({})\n     \
                    actual: {:?} ({})\n",
                stringify!($($t)*),
                $expected,
                $expected,
                actual,
                actual,
            );
        }
    }};
}

#[test]
fn writeln() {
    let mut b = buf();
    writeln!(b, "").unwrap();
    assert_eq!(raw_str(&b), "\n");

    let mut b = buf();
    writeln!(b, "hello").unwrap();
    assert_eq!(raw_str(&b), "hello\n");

    let mut b = buf();
    writeln!(b, "a {$red}b{/$}").unwrap();
    assert_eq!(raw_str(&b), "a \x1b[0m\x1b[31mb\x1b[0m\n");
}

#[test]
fn no_move_buffer() {
    {
        let mut b = buf();
        write!(b, "hi").unwrap();
        drop(b);
    }
    {
        let mut b = buf();
        write!(&mut b, "hi").unwrap();
        drop(b);
    }

    write!(buf(), "hi").unwrap();
    write!(&mut buf(), "hi").unwrap();
}

#[test]
fn no_move_args() {
    #[derive(Debug)]
    struct NoCopy;

    impl fmt::Display for NoCopy {
        fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
            Ok(())
        }
    }

    {
        let string = "hi".to_string();
        let no_copy = NoCopy;
        let mut b = buf();
        write!(b, "a{[green]}b{:?}c", string, no_copy).unwrap();
        assert_eq!(raw_str(&b), "a\x1b[0m\x1b[32mhi\x1b[0mbNoCopyc");

        // We can still use the variables.
        drop(string);
        drop(no_copy);
    }
    {
        let s = "x".to_string();
        let mut b = buf();
        writeln!(b, "a{:?}b{}", s, s).unwrap();
        drop(s);
    }

    // The `print[ln]` use termcolor and thus circumvent the stdout capture by
    // the Rust test harness. To avoid ugly test output, we print basically no
    // text. Only `println` actually emits one newline.
    {
        let x = NoCopy;
        print!("{}", x);
        drop(x);
    }
    {
        let x = NoCopy;
        println!("{}", x);
        drop(x);
    }
}

#[test]
fn arg_referal() {
    check!("27" == "{peter}", peter = 27);
    check!("27 27" == "{0} {0}", 27);

    check!(
        "a p c b a m" ==
        "{} {peter} {2} {} {0} {mary}", 'a', 'b', 'c', peter = 'p', mary = 'm'
    );
}

#[test]
fn raw_strings() {
    check!("hello" == r"hello");
    check!(r"a\n" == r"a\n");
}

#[test]
fn no_style() {
    check!("hello" == "hello");
    check!("Foo 27 bar" == "Foo {} bar", 27);
    check!("x Foo 27 bar" == "{} Foo {} bar", 'x', 27);
    check!("Foo 8 barfren" == "Foo {} bar{}", 8, "fren");

    check!(
        "a\nb\tc\rd\0e\x48f\u{50}g\u{228}h\u{fffe}i\u{1F923}j" ==
        "a\nb\tc\rd\0e\x48f\u{50}g\u{228}h\u{fffe}i\u{1F923}j"
    );
    check!(
        "abc\
            def" ==
        "abcdef"
    );
}

#[test]
fn simple_tags() {
    check!("a\x1b[0m\x1b[31mb\x1b[0mc" == "a{$red}b{/$}c");
    check!("a\x1b[0m\x1b[33mbanana\x1b[0m" == "a{$yellow}banana{/$}");
    check!("\x1b[0m\x1b[34mocean\x1b[0m is wet" == "{$blue}ocean{/$} is wet");
    check!("\x1b[0m\x1b[1meverything\x1b[0m" == "{$bold}everything{/$}");

    check!("foo\x1b[0m\x1b[1m\x1b[32mbar\x1b[0mbaz" == "foo{$bold+green}bar{/$}baz");
    check!(
        "foo\x1b[0m\x1b[1m\x1b[32m\x1b[44mbar\x1b[0mbaz" ==
        "foo{$bold+green+bg:blue}bar{/$}baz"
    );
    check!(
        "foo\x1b[0m\x1b[1m\x1b[3m\x1b[38;5;10m\x1b[48;5;12mbar\x1b[0mbaz" ==
        "foo{$bold+green+bg:blue+intense+italic}bar{/$}baz"
    );
}

#[test]
fn nested_tags() {
    check!(
        "a\x1b[0m\x1b[31mb\x1b[0m\x1b[1m\x1b[31mc\x1b[0m\x1b[31md\x1b[0me" ==
        "a{$red}b{$bold}c{/$}d{/$}e"
    );
    check!(
        "a\x1b[0m\x1b[31mb\x1b[0m\x1b[33mc\x1b[0m\x1b[31md\x1b[0me" ==
        "a{$red}b{$yellow}c{/$}d{/$}e"
    );
    check!(
        "a\x1b[0m\x1b[31mb\x1b[0m\x1b[1m\x1b[31mc\x1b[0m\x1b[1m\x1b[33md\x1b[0m\x1b[1m\x1b[31me\
            \x1b[0m\x1b[31mf\x1b[0mg" ==
        "a{$red}b{$bold}c{$yellow}d{/$}e{/$}f{/$}g"
    );
}

#[test]
fn colored_args() {
    check!("\x1b[0m\x1b[32m27\x1b[0m" == "{[green]}", 27);
    check!("a\x1b[0m\x1b[32m27\x1b[0m" == "a{[green]}", 27);
    check!("\x1b[0m\x1b[32m27\x1b[0mb" == "{[green]}b", 27);
    check!("a\x1b[0m\x1b[32m27\x1b[0mb" == "a{[green]}b", 27);

    check!("\x1b[0m\x1b[35mtrue\x1b[0m" == "{[magenta]:?}", true);
    check!("\x1b[0m\x1b[35m3f\x1b[0m" == "{[magenta]:x}", 0x3f);
    check!("\x1b[0m\x1b[35m3F\x1b[0m" == "{[magenta]:X}", 0x3f);
    check!("\x1b[0m\x1b[35m123\x1b[0m" == "{[magenta]:o}", 0o123);
    check!("\x1b[0m\x1b[35m101010\x1b[0m" == "{[magenta]:b}", 0b101010);
    check!("\x1b[0m\x1b[35m3.14e0\x1b[0m" == "{[magenta]:e}", 3.14);
    check!("\x1b[0m\x1b[35m3.14E0\x1b[0m" == "{[magenta]:E}", 3.14);

    check!(
        "a \x1b[0m\x1b[32m27\x1b[0m b \x1b[0m\x1b[31mtrue\x1b[0m c \x1b[0m\x1b[33mbanana\x1b[0m" ==
        "a {[green]} b {[red]} c {[yellow]}", 27, true, "banana"
    );
    check!(
        "a \x1b[0m\x1b[32m27\x1b[0m b \x1b[0m\x1b[1m3.14\x1b[0m c" ==
        "a {[green]} b {[bold]} c", 27, 3.14
    );
}

#[test]
fn mixed_tag_args() {
    check!(
        "a \x1b[0m\x1b[1mb\x1b[0m\x1b[1m\x1b[32m27\x1b[0m\x1b[1mc\x1b[0md" ==
        "a {$bold}b{[green]}c{/$}d", 27
    );

    check!(
        "a \x1b[0m\x1b[33m...\x1b[0m\x1b[38;5;11m27\x1b[0m\x1b[33m...\x1b[0m\x1b[1m\x1b\
            [33mb\x1b[0m\x1b[1m\x1b[32mtrue\x1b[0m\x1b[1m\x1b[33mc\x1b[0m\x1b[33m\x1b[0md" ==
        "a {$yellow}...{[intense]}...{$bold}b{[green]}c{/$}{/$}d", 27, true
    );
}