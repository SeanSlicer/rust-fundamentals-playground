//! Closures: anonymous functions that capture their environment.
//!
//! The three closure traits describe HOW the environment is captured
//! and used:
//! * `Fn`     — reads captures (or captures nothing). Callable many times.
//! * `FnMut`  — mutates captures. Callable many times, needs `mut`.
//! * `FnOnce` — consumes captures. Callable exactly once.
//!
//! Every closure implements FnOnce; most also implement FnMut; the
//! tamest also implement Fn. When writing a function that ACCEPTS a
//! closure, ask for the weakest trait you can (FnOnce > FnMut > Fn in
//! permissiveness for the caller).

/// Taking a closure as a parameter — static dispatch via generics.
/// `Fn(i32) -> i32` because we call it twice and only read from it.
pub fn apply_twice(f: impl Fn(i32) -> i32, input: i32) -> i32 {
    f(f(input))
}

/// FnMut: the closure mutates state it captured. The parameter must be
/// declared `mut` here, because calling an FnMut is a mutation.
pub fn run_n_times(mut f: impl FnMut(), n: usize) {
    for _ in 0..n {
        f();
    }
}

/// FnOnce: we only promise to call it once, so callers may hand us a
/// closure that consumes its captures (e.g. moves a String out).
/// Accepting FnOnce here is maximally permissive for the caller.
pub fn call_with_default(f: impl FnOnce() -> String, use_default: bool) -> String {
    if use_default {
        String::from("default")
    } else {
        f()
    }
}

/// Returning a closure: the concrete type is unnameable, so we write
/// `impl Fn`. The `move` keyword forces the closure to take ownership
/// of `factor` — required, because the closure outlives this function
/// call and must not borrow from a dead stack frame.
pub fn multiplier(factor: i32) -> impl Fn(i32) -> i32 {
    move |x| x * factor
}

/// Storing closures in a struct requires either generics (one closure
/// type per struct instance) or boxing (any closure, dynamic
/// dispatch). Box<dyn Fn> is the flexible choice for collections of
/// different closures.
pub struct Pipeline {
    steps: Vec<Box<dyn Fn(i32) -> i32>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline { steps: Vec::new() }
    }

    /// Builder-style: consume and return self so calls chain.
    pub fn then(mut self, step: impl Fn(i32) -> i32 + 'static) -> Self {
        self.steps.push(Box::new(step));
        self
    }

    pub fn run(&self, input: i32) -> i32 {
        self.steps.iter().fold(input, |value, step| step(value))
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Closures vs functions: plain `fn` items coerce to closure traits,
/// so APIs taking `impl Fn` accept both. Capture is the only thing a
/// closure can do that a function cannot.
fn add_one(x: i32) -> i32 {
    x + 1
}

pub fn function_pointers_work_too() -> i32 {
    apply_twice(add_one, 0) // a plain fn where a closure is expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fn_closures_read_captures() {
        let offset = 10; // captured by reference — we only read it
        assert_eq!(apply_twice(|x| x + offset, 1), 21);
        assert_eq!(offset, 10); // still usable, it was only borrowed
    }

    #[test]
    fn fnmut_closures_mutate_captures() {
        let mut count = 0;
        run_n_times(|| count += 1, 3);
        assert_eq!(count, 3);
    }

    #[test]
    fn fnonce_closures_consume_captures() {
        let greeting = String::from("hello");
        // This closure MOVES greeting out when called — it can only be
        // FnOnce. call_with_default accepts it because it promises a
        // single call.
        let result = call_with_default(move || greeting, false);
        assert_eq!(result, "hello");
    }

    #[test]
    fn returned_closures_own_their_state() {
        let triple = multiplier(3);
        assert_eq!(triple(7), 21);
        assert_eq!(triple(0), 0); // Fn: callable repeatedly
    }

    #[test]
    fn boxed_closures_compose_into_pipelines() {
        let pipeline = Pipeline::new()
            .then(|x| x + 1)
            .then(|x| x * 2)
            .then(|x| x - 3);
        assert_eq!(pipeline.run(5), 9); // ((5+1)*2)-3
    }

    #[test]
    fn plain_functions_coerce() {
        assert_eq!(function_pointers_work_too(), 2);
    }
}

// Exercises
// ---------
// 1. Write `fn make_counter() -> impl FnMut() -> u32` returning a
//    closure that yields 1, 2, 3, ... on successive calls.
// 2. Why does Pipeline::then require `+ 'static` on the closure?
//    Remove it and read the error: what could go wrong with a closure
//    borrowing a local variable?
// 3. Change `apply_twice` to take FnMut. Does any existing caller
//    break? Which direction of loosening/tightening is backwards
//    compatible?
