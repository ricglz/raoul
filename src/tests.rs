use super::{parse, DirFunc};

#[test]
fn use_of_undeclared_variable() {
    let filename = "examples/invalid/undeclared-variable.ra";
    let program = std::fs::read_to_string(filename).expect(filename);
    let debug = true;
    let ast_response = parse(&program, debug);
    assert!(ast_response.is_ok());
    let ast = ast_response.unwrap();
    let mut dir_func = DirFunc::new();
    let build_res = dir_func.build_dir_func(ast);
    assert!(build_res.is_err());
}

#[test]
fn redefinition_of_function() {
    let filename = "examples/invalid/redefined-function.ra";
    let program = std::fs::read_to_string(filename).expect(filename);
    let debug = true;
    let ast_response = parse(&program, debug);
    assert!(ast_response.is_ok());
    let ast = ast_response.unwrap();
    let mut dir_func = DirFunc::new();
    let build_res = dir_func.build_dir_func(ast);
    assert!(build_res.is_err());
}

#[test]
fn redefine_variable_type() {
    let filename = "examples/invalid/redefine-variable-type.ra";
    let program = std::fs::read_to_string(filename).expect(filename);
    let debug = true;
    let ast_response = parse(&program, debug);
    assert!(ast_response.is_ok());
    let ast = ast_response.unwrap();
    let mut dir_func = DirFunc::new();
    let build_res = dir_func.build_dir_func(ast);
    assert!(build_res.is_err());
}
