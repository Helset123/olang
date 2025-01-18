# Olang Tutorial

## Introduction
This document serves as a tutorial for the *Olang* scripting language

## The first program
We will start with a program that greets the user
```
printLn("What is your name?")
var name = readLn()
printLn("Hello, " + name + "!")
```
Save this code to a the text file "greeter.olang" and run it using:
```bash
olang greeter.olang
```
alternatively you can run code from the command line using the -c paramter
```bash
olang -c 'printLn("Hello, World!")'
```

## Lexical elements
This will be a summary of the individual lexical elements in the olang programming language.
### String literals
String literals are enclosed using double-quotes. They support being written between multiple lines.
```
var string = "hello"
var muliline = "this
is
an
multiline
string"
```
### Comments
The lexer will ignore comments when parsing the source code. Coments are marked using `#`. Alternatively you can use `#[` and `]#`.
```
printLn("Hello") # Greet the user
#[
  The content of this multiline string will be ignored
  print("Bye") <- this won't run
]#
```
