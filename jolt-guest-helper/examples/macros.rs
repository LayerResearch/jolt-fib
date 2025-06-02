use jolt_guest_helper::{step, step_result};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of using the step! macro
    let result = step!("Performing operation", {
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_secs(1));
        42
    });
    println!("Result: {}", result);

    // Example of using the step_result! macro
    let result = step_result!("Performing operation with result", {
        // Simulate some work that might fail
        std::thread::sleep(std::time::Duration::from_secs(1));
        Ok::<i32, &str>(42)
    })?;
    println!("Result: {}", result);

    // Example of error handling with step_result!
    let result = step_result!("Performing operation that fails", {
        // Simulate a failure
        std::thread::sleep(std::time::Duration::from_secs(1));
        Err::<i32, &str>("Operation failed")
    });
    println!("Error result: {:?}", result);

    Ok(())
} 