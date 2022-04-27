use super::{parse, parse_ast};

fn parse_ast_has_error(filename: &str) {
    let program = std::fs::read_to_string(filename).expect(filename);
    let debug = true;
    let ast_response = parse(&program, debug);
    assert!(ast_response.is_ok());
    let ast = ast_response.unwrap();
    assert!(parse_ast(ast, debug).is_err());
}

#[test]
fn use_of_undeclared_variable() {
    let filename = "examples/invalid/undeclared-variable.ra";
    parse_ast_has_error(filename);
}

#[test]
fn use_of_undeclared_variable_if() {
    let filename = "examples/invalid/undeclared-variable-if.ra";
    parse_ast_has_error(filename);
}

#[test]
fn use_of_undeclared_variable_while() {
    let filename = "examples/invalid/undeclared-variable-while.ra";
    parse_ast_has_error(filename);
}

#[test]
fn redefinition_of_function() {
    let filename = "examples/invalid/redefined-function.ra";
    parse_ast_has_error(filename);
}

#[test]
fn redefine_variable_type() {
    let filename = "examples/invalid/redefine-variable-type.ra";
    parse_ast_has_error(filename);
}

#[test]
fn invalid_cast() {
    let filename = "examples/invalid/invalid-cast.ra";
    parse_ast_has_error(filename);
}
