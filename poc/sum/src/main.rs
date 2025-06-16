fn sum(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let sum = sum(1, 2);
    println!("sum: {}", sum);
    std::process::exit(0); // Add this line for testing
}


