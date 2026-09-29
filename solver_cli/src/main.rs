fn main() {
    println!("Hello, world!");

    let adder = 1;
    let addend = 2;
    let value = solver_lib::add(adder, addend);

    println!(
        "Hello world! The value of adding {} to {} is {}!",
        adder, addend, value
    );
}
