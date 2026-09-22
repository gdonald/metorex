// Building a Method from the parameter list it was written with.

use super::*;

/// Build a Method from a parameter list, handling positional, variadic,
/// keyword, block, and default parameters. Shared by the FunctionDef arms in
/// `apply_class_body` (i.e. `def` inside `Class.new do ... end`) so they pick
/// up the same parameter modes as the MethodDef path.
pub(crate) fn build_method_from_params(
    name: String,
    parameters: &[crate::ast::Parameter],
    body: Vec<Statement>,
    refinements: Vec<(Rc<Class>, Vec<String>)>,
    nesting: Vec<Rc<Class>>,
) -> Method {
    let param_names: Vec<String> = parameters
        .iter()
        .filter(|p| !p.is_named_keyword && !p.is_keyword && !p.is_block)
        .map(|p| p.name.clone())
        .collect();
    let keyword_parameters: Vec<(String, Option<Expression>)> = parameters
        .iter()
        .filter(|p| p.is_named_keyword)
        .map(|p| (p.name.clone(), p.default_value.clone()))
        .collect();
    let block_parameter = parameters
        .iter()
        .find(|p| p.is_block)
        .map(|p| p.name.clone());
    let default_parameters: Vec<(usize, Expression)> = parameters
        .iter()
        .filter(|p| !p.is_named_keyword && !p.is_keyword && !p.is_block)
        .enumerate()
        .filter_map(|(i, p)| p.default_value.clone().map(|dv| (i, dv)))
        .collect();
    let variadic_param = parameters
        .iter()
        .filter(|p| !p.is_named_keyword && !p.is_keyword && !p.is_block)
        .enumerate()
        .find(|(_, p)| p.is_variadic)
        .map(|(i, p)| (i, p.name.clone()));
    let mut m = Method::new(name, param_names, body);
    m.default_parameters = default_parameters;
    m.keyword_parameters = keyword_parameters;
    m.keyword_rest_parameter = parameters
        .iter()
        .find(|p| p.is_keyword)
        .map(|p| p.name.clone());
    m.block_parameter = block_parameter;
    m.variadic_param = variadic_param;
    m.captured_refinements = refinements;
    m.captured_nesting = nesting;
    m
}
