use super::*;

#[cfg(feature = "verbose-expansions")]
pub(in crate)
fn pretty_print_tokenstream (
    code: &'_ TokenStream2,
)
{
    fn try_format (input: &'_ str)
      -> Option<String>
    {Some({
        let mut child =
            ::std::process::Command::new("rustfmt")
                .args(&["--edition", "2018"])
                .stdin(::std::process::Stdio::piped())
                .stdout(::std::process::Stdio::piped())
                .stderr(::std::process::Stdio::piped())
                .spawn()
                .ok()?
        ;
        match child.stdin.take().unwrap() { ref mut stdin => {
            ::std::io::Write::write_all(stdin, input.as_bytes()).ok()?;
        }}
        let mut stdout = String::new();
        ::std::io::Read::read_to_string(
            &mut child.stdout.take().unwrap(),
            &mut stdout,
        ).ok()?;
        if child.wait().ok()?.success().not() { return None; }
        stdout
    })}
    let mut code = code.to_string();
    // Try to format the code, but don't sweat it if it fails.
    if let Some(formatted) = try_format(&code) {
        code = formatted;
    }
    // Now let's try to also colorize it:
    if  ::bat::PrettyPrinter::new()
            .input_from_bytes(code.as_ref())
            .language("rust")
            .true_color(false)
            .snip(true)
            .print()
            .is_err()
    {
        // Fallback to non-colorized output.
        println!("{}", code);
    }
}
