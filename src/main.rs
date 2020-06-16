mod abstract_syntax_tree;
mod executor;
mod constructors;
use code_tokenizer::get_tokens;
use crate::abstract_syntax_tree::*;
use std::borrow::{BorrowMut, Borrow};
use crate::executor::{VVA};
use std::cell::{RefCell, Ref};
use std::rc::Rc;
use crate::executor::Variable;


fn main() {
    let input_string = "\
    let my_variable = function(a, b, c){\
        return a + b + c;
    };
    let my_variable_3 = my_variable(1, 2, 3);
    print(my_variable_3)".to_string();
    let operators = vec![
        "+".to_string(),
        "-".to_string(),
        "*".to_string(),
        "/".to_string(),

        "+=".to_string(),
        "-=".to_string(),
        "*=".to_string(),
        "/=".to_string(),

        "=".to_string(),

        "==".to_string(),
        "!=".to_string(),
        "<".to_string(),
        ">".to_string(),

        "(".to_string(),
        ")".to_string(),
        "{".to_string(),
        "}".to_string(),

        ".".to_string(),
        ",".to_string(),

        ";".to_string()
    ];

    let operator_priorities = vec![
        0,  // +
        0,  // -
        1,  // *
        1,  // /

        -1, // +=
        -1, // -=
        -1, // *=
        -1, // /=

        -3, // =

        -2, // ==
        -2, // !=
        -2, // <
        -2, // >

        2,  // (
        0,  // )
        0,  // {
        0,  // }

        2, // .
        0, // ,

        0  // ;
    ];
    let tokens = get_tokens(input_string, operators.clone());
    let mut scope = Scope::parse(tokens.borrow(), operators.borrow(), operator_priorities.borrow(), 0.borrow_mut(), tokens.len() as i64);

    scope.accessible_variables.push(Rc::new(RefCell::new(Variable{
        name: Some("print".to_string()),
        constant: Some(Constant::Function(Rc::new(RefCell::new(executor::PrintFunction)))),
        members: vec![]
    })));

    let value = executor::execute_scope(Rc::new(RefCell::new(scope)));
}
