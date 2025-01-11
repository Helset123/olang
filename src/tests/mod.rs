use super::*;

#[test]
fn fibonacci_sequence() {
    assert_eq!(eval(include_str!("fib.olang")).unwrap(), Value::Int(6765));
}

#[test]
fn strings() {
    assert_eq!(eval("\"arst\"").unwrap(), Value::String("arst".to_string()));
    assert_eq!(
        eval("\"arst\narstarst\"").unwrap(),
        Value::String("arst\narstarst".to_string())
    );
}

#[test]
fn int() {
    assert_eq!(eval("1").unwrap(), Value::Int(1));
    assert_eq!(eval("1234").unwrap(), Value::Int(1234));
    // assert_eq!(eval("-1234").unwrap(), Value::Int(-1234));
}

#[test]
fn pemdas() {
    assert_eq!(eval("6/2*(1+2)").unwrap(), Value::Int(9));
    assert_eq!(eval("3+5*2").unwrap(), Value::Int(13));
    assert_eq!(eval("(3+5)*2").unwrap(), Value::Int(16));
    assert_eq!(eval("8/4*2").unwrap(), Value::Int(4));
    assert_eq!(eval("10-2+3").unwrap(), Value::Int(11));
    assert_eq!(eval("((1+2)*(3+4))").unwrap(), Value::Int(21));
    assert_eq!(eval("((5-2)+(3*4))").unwrap(), Value::Int(15));
    assert_eq!(eval("((2+3)*(4-(1+1)))").unwrap(), Value::Int(10));
}

#[test]
fn pemdas_braces() {
    assert_eq!(eval("6/2*{1+2}").unwrap(), Value::Int(9));
    assert_eq!(eval("{3+5}*2").unwrap(), Value::Int(16));
    assert_eq!(eval("{{1+2}*{3+4}}").unwrap(), Value::Int(21));
    assert_eq!(eval("{{5-2}+{3*4}}").unwrap(), Value::Int(15));
    assert_eq!(eval("{{2+3}*{4-{1+1}}}").unwrap(), Value::Int(10));
}
