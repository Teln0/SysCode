use crate::executor::{Variable, Callable};
use crate::abstract_syntax_tree::{Value, ValueType, Constant, Scope, Function, Tuple};
use std::cell::{RefCell};
use std::rc::Rc;

pub fn construct_variable_from_integer(integer: i64) -> Rc<RefCell<Variable>> {
    return Rc::new(RefCell::new(Variable{
        name: None,
        constant: Some(Constant::Integer(integer)),
        members: vec![]
    }));
}

pub fn construct_variable_from_function(function: Rc<RefCell<Callable>>) -> Rc<RefCell<Variable>> {
    return Rc::new(RefCell::new(Variable {
        name: None,
        constant: Some(Constant::Function(function)),
        members: vec![]
    }))
}

pub fn construct_variable_from_tuple(tuple: Rc<RefCell<Tuple>>) -> Rc<RefCell<Variable>> {
    return Rc::new(RefCell::new(Variable {
        name: None,
        constant: Some(Constant::Tuple(tuple)),
        members: vec![]
    }));
}

pub fn construct_variable(value: Value, scope: Rc<RefCell<Scope>>) -> Rc<RefCell<Variable>> {
    match value.value_type {
        ValueType::Undefined => {
            return Rc::new(RefCell::new(Variable {
                members: vec![],
                name: None,
                constant: Some(Constant::Undefined),
            }));
        }
        ValueType::VariableName => {
            fn get_variable_by_name(scope: Rc<RefCell<Scope>>, name: &str) -> Rc<RefCell<Variable>> {
                for accessible_var in (*scope).borrow().accessible_variables.iter() {
                    if (*accessible_var).borrow_mut().name.as_ref().unwrap().eq(name) {
                        return accessible_var.clone();
                    }
                }

                if (*scope).borrow().parent.is_some() {
                    return get_variable_by_name((*scope).borrow().parent.as_ref().unwrap().clone(), name);
                }

                return Rc::new(RefCell::new(Variable {
                    members: vec![],
                    name: Some(name.to_string()),
                    constant: Some(Constant::Undefined),
                }));
            }

            return get_variable_by_name(scope, value.variable.as_ref().unwrap().as_str());
        }
        ValueType::Constant => {
            match value.constant.as_ref().unwrap() {
                Constant::Undefined => {
                    return Rc::new(RefCell::new(Variable {
                        members: vec![],
                        name: None,
                        constant: Some(Constant::Undefined),
                    }));
                },
                Constant::Integer(i) => {
                    return construct_variable_from_integer(*i);
                },
                Constant::Function(f) => {
                    return construct_variable_from_function(f.clone());
                }
                Constant::Tuple(t) => {
                    return construct_variable_from_tuple(t.clone());
                }
            }
        }
    }
}