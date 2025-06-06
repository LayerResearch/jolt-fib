use spinners::{Spinner, Spinners};

/// A macro for executing an action with a spinner and status message.
///
/// This macro shows a spinner while executing the given action and displays
/// a success message when complete. It's useful for long-running operations
/// to provide visual feedback to the user.
///
/// # Arguments
///
/// * `$msg` - The message to display while the action is running
/// * `$action` - The action to execute (can be any expression)
///
/// # Returns
///
/// Returns the result of the action.
///
/// # Example
///
/// ```rust
/// use jolt_guest_helper::step;
///
/// let result = step!("Compiling program", {
///     // Your long-running operation here
///     compile_program()
/// });
/// ```
#[macro_export]
macro_rules! step {
    ($msg:expr, $action:expr) => {{
        use spinners::{Spinner, Spinners};
        let mut sp = Spinner::new(Spinners::Dots9, $msg.to_string());
        let result = $action;
        sp.stop_with_message(format!("✓ {}", $msg));
        result
    }};
}

/// A macro for executing an action with a spinner, status message, and error handling.
///
/// Similar to `step!` but includes error handling. If the action returns an error,
/// the spinner will show a failure message.
///
/// # Arguments
///
/// * `$msg` - The message to display while the action is running
/// * `$action` - The action to execute (should return a Result)
///
/// # Returns
///
/// Returns the Result from the action.
///
/// # Example
///
/// ```rust
/// use jolt_guest_helper::step_result;
///
/// let result = step_result!("Compiling program", {
///     // Your long-running operation here
///     compile_program()
/// })?;
/// ```
#[macro_export]
macro_rules! step_result {
    ($msg:expr, $action:expr) => {{
        use spinners::{Spinner, Spinners};
        let mut sp = Spinner::new(Spinners::Dots9, $msg.to_string());
        let result = $action;
        match &result {
            Ok(_) => sp.stop_with_message(format!("✓ {}", $msg)),
            Err(_) => sp.stop_with_message(format!("✗ {}", $msg)),
        }
        result
    }};
}
