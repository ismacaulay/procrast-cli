#[cfg(production)]
pub fn println(_: String) {}

#[cfg(not(production))]
pub fn println(msg: String) {
    println!("{}", msg);
}
