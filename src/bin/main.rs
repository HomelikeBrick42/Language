use lang::{parsing::parse, pretty_printing::pretty_print_ast, INTERNER};

fn main() {
    let filepath = "test.lang";
    let source = "
fn foo(param) -> int {
    let x = 1 + 2 * 3;
    let y = x / 2;
    fn double(x) {
        return x * 2;
    }
    let z = double(y);
    return 10 / param - (z - 5);
}
";
    let asts = parse(INTERNER.get_or_intern(filepath), source).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1)
    });
    let stdout = &mut std::io::stdout();
    for ast in asts {
        pretty_print_ast(&ast, 0, stdout).unwrap();
    }
}
