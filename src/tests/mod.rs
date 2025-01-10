use super::*;

#[test]
fn fibonacci_sequence() {
    assert_eq!(eval(include_str!("fib20.olang")).unwrap(), Value::Int(6765));
    assert_eq!(eval(include_str!("fib10.olang")).unwrap(), Value::Int(55));
    assert_eq!(eval(include_str!("fib3.olang")).unwrap(), Value::Int(2));
}
