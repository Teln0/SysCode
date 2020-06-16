use crate::abstract_syntax_tree::{Expression, ExpressionType, Value, ValueType, Constant, Dumpable, Statement, Scope, Function, Tuple};
use std::borrow::{Borrow};
use crate::constructors::construct_variable;
use std::cell::RefCell;
use std::rc::Rc;
use std::ops::Deref;
use crate::abstract_syntax_tree::ValueType::VariableName;

pub struct Variable {
    pub name: Option<String>,
    pub constant: Option<Constant>,
    pub members: Vec<Rc<RefCell<Variable>>>,
}

impl Clone for Variable {
    fn clone(&self) -> Self {
        return Variable {
            name: self.name.clone(),
            constant: self.constant.clone(),
            members: self.members.clone(),
        };
    }
}

impl Dumpable for Variable {
    fn get_dump(&self) -> String {
        let mut result = "".to_string();
        if self.name.is_some() {
            result.push_str(self.name.as_ref().unwrap().as_str());
        } else {
            result.push_str("nameless");
        }
        result.push_str(" : ");
        if self.constant.is_some() {
            result.push_str(self.constant.as_ref().unwrap().get_dump().as_str());
        }
        return result;
    }

    fn dump(&self) {
        println!("{}", self.get_dump());
    }
}

impl Variable {
    fn assign(&mut self, other_variable: Rc<RefCell<Variable>>) {
        self.constant = (*other_variable).borrow().constant.clone();
        self.members = (*other_variable).borrow().members.clone();
    }
}

pub enum VVA {
    Variable(Rc<RefCell<Variable>>),
    Value(Value),
}

impl VVA {
    fn to_variable(self, scope: Rc<RefCell<Scope>>) -> Rc<RefCell<Variable>> {
        match self {
            VVA::Variable(var) => {
                return var;
            }
            VVA::Value(val) => {
                return construct_variable(val, scope);
            }
        }
    }
}

pub fn execute_scope(scope: Rc<RefCell<Scope>>) -> Option<VVA> {
    let statementslen = (*scope).borrow().statements.len();
    for i in 0..statementslen {
        let statement;
        {
            let s = scope.clone();
            let s1 = (*s).borrow();
            statement = s1.statements[i].clone();
        }
        execute_statement(statement.borrow(), scope.clone());
        if (*scope).borrow().return_value.is_some() {
            return Some(VVA::Variable((*scope).borrow().return_value.as_ref().unwrap().clone()));
        }
    }

    return None;
}

pub fn execute_statement(statement: &Statement, scope: Rc<RefCell<Scope>>) -> Option<VVA> {
    match statement {
        Statement::Undefined => {
            return None;
        }
        Statement::Expression(expression) => {
            return Some(execute_expression(expression, scope));
        }
        Statement::VariableDeclaration(expression) => {
            let result;
            {
                result = execute_expression(expression, scope.clone()).to_variable(scope.clone());
            }
            if (*result).borrow().name.is_some() {
                {
                    for i in (*scope).borrow().accessible_variables.iter() {
                        if i.deref().borrow().name.as_ref().unwrap().eq((*result).borrow().name.as_ref().unwrap().as_str()) {
                            i.deref().borrow_mut().assign(result.clone());
                            return None;
                        }
                    }
                }

                (*scope.clone()).borrow_mut().accessible_variables.push(result.clone());
            } else {
                println!("Error: Cannot create nameless variable.");
                panic!();
            }
        },
        Statement::ReturnStatement(expression) => {
            let result;
            {
                result = execute_expression(expression, scope.clone()).to_variable(scope.clone());
            }
            (*scope.clone()).borrow_mut().return_value = Some(result.clone());
        }
    }

    return None;
}

pub fn execute_expression(expression: &Expression, scope: Rc<RefCell<Scope>>) -> VVA {
    match expression.expression_type {
        ExpressionType::Undefined => {
            return VVA::Value(Value {
                value_type: ValueType::Undefined,
                constant: None,
                variable: None,
            });
        }
        ExpressionType::Value => {
            match expression.value.as_ref().unwrap().value_type {
                ValueType::VariableName => {
                    return VVA::Value(expression.value.as_ref().unwrap().clone());
                },
                _ => {
                    return VVA::Variable(construct_variable(expression.value.as_ref().unwrap().clone(), scope));
                }
            }
        }
        ExpressionType::Operation => {
            let left_value = execute_expression(expression.left.as_ref().unwrap(), scope.clone()).to_variable(scope.clone());

            let right_value = execute_expression(expression.right.as_ref().unwrap(), scope.clone());
            match right_value {
                VVA::Variable(variable) => {
                    let result = Variable::apply_operator_right(
                        left_value,
                        variable.clone(),
                        expression.operator.as_ref().unwrap(),
                        scope.clone(),
                    );
                    return VVA::Variable(result);
                }
                VVA::Value(value) => {
                    match value.value_type {
                        ValueType::VariableName => {
                            if expression.operator.as_ref().unwrap().eq(".") {
                                let result = Variable::apply_operator_right_vn(
                                    left_value,
                                    value.variable.as_ref().unwrap(),
                                    expression.operator.as_ref().unwrap(),
                                    scope.clone(),
                                );
                                return VVA::Variable(result);
                            } else {
                                let result = Variable::apply_operator_right(
                                    left_value,
                                    construct_variable(value, scope.clone()),
                                    expression.operator.as_ref().unwrap(),
                                    scope.clone(),
                                );
                                return VVA::Variable(result);
                            }
                        }
                        _ => {
                            let result = Variable::apply_operator_right(
                                left_value,
                                construct_variable(value, scope.clone()),
                                expression.operator.as_ref().unwrap(),
                                scope.clone(),
                            );
                            return VVA::Variable(result);
                        }
                    };
                }
            }
        }
    }
}

trait Evaluable {
    fn apply_operator_right(var_ref_cell: Rc<RefCell<Variable>>, right: Rc<RefCell<Variable>>, operator: &String, accessible_variables: Rc<RefCell<Scope>>) -> Rc<RefCell<Variable>>;
    fn apply_operator_right_vn(var_ref_cell: Rc<RefCell<Variable>>, right: &String, operator: &String, accessible_variables: Rc<RefCell<Scope>>) -> Rc<RefCell<Variable>>;
}

impl Evaluable for Variable {
    fn apply_operator_right(var_ref_cell: Rc<RefCell<Variable>>, right: Rc<RefCell<Variable>>, operator: &String, scope: Rc<RefCell<Scope>>) -> Rc<RefCell<Variable>> {
        match operator.as_str() {
            "=" => {
                (*var_ref_cell.clone()).borrow_mut().assign(right);
                return var_ref_cell;
            },
            "(" => {
                let constant;
                {
                    constant = var_ref_cell.as_ref().borrow().constant.clone().unwrap();
                }
                match constant {
                    Constant::Undefined => {
                        return construct_variable(Value {
                            value_type: ValueType::Undefined,
                            constant: None,
                            variable: None,
                        }, scope.clone());
                    }
                    Constant::Function(f) => {
                        return f.deref().borrow_mut().call((*right).borrow().constant.as_ref().unwrap().as_tuple(), scope.clone());
                    }
                    _ => {
                        println!("Error: No such operator type.");
                        panic!();
                    }
                };
            }
            "+" => {
                match (*var_ref_cell).borrow().constant.as_ref().unwrap() {
                    Constant::Undefined => {
                        return construct_variable(Value {
                            value_type: ValueType::Undefined,
                            constant: None,
                            variable: None,
                        }, scope.clone());
                    }
                    Constant::Integer(i) => {
                        return construct_variable(Value {
                            value_type: ValueType::Constant,
                            constant: Some(Constant::Integer(i + (*right).borrow_mut().constant.as_ref().unwrap().as_integer())),
                            variable: None,
                        }, scope.clone());
                    }
                    _ => {
                        println!("Error: No such operator type.");
                        panic!();
                    }
                };
            }
            "+=" => {
                let constant;
                {
                    constant = var_ref_cell.as_ref().borrow().constant.clone().unwrap();
                }
                match constant {
                    Constant::Undefined => {
                        return construct_variable(Value {
                            value_type: ValueType::Undefined,
                            constant: None,
                            variable: None,
                        }, scope.clone());
                    }
                    Constant::Integer(i) => {
                        let result;
                        {
                            result = Some(Constant::Integer(i + (*right).borrow_mut().constant.as_ref().unwrap().as_integer()));
                        }
                        (*var_ref_cell).borrow_mut().constant = result;
                        return var_ref_cell.clone();
                    }
                    _ => {
                        println!("Error: No such operator type.");
                        panic!();
                    }
                };
            }
            "-" => {
                match (*var_ref_cell).borrow().constant.as_ref().unwrap() {
                    Constant::Undefined => {
                        return construct_variable(Value {
                            value_type: ValueType::Undefined,
                            constant: None,
                            variable: None,
                        }, scope.clone());
                    }
                    Constant::Integer(i) => {
                        return construct_variable(Value {
                            value_type: ValueType::Constant,
                            constant: Some(Constant::Integer(i - (*right).borrow_mut().constant.as_ref().unwrap().as_integer())),
                            variable: None,
                        }, scope.clone());
                    }
                    _ => {
                        println!("Error: No such operator type.");
                        panic!();
                    }
                };
            }
            "-=" => {
                let constant;
                {
                    constant = var_ref_cell.as_ref().borrow().constant.clone().unwrap();
                }
                match constant {
                    Constant::Undefined => {
                        return construct_variable(Value {
                            value_type: ValueType::Undefined,
                            constant: None,
                            variable: None,
                        }, scope.clone());
                    }
                    Constant::Integer(i) => {
                        let result;
                        {
                            result = Some(Constant::Integer(i - (*right).borrow_mut().constant.as_ref().unwrap().as_integer()));
                        }
                        (*var_ref_cell).borrow_mut().constant = result;
                        return var_ref_cell.clone();
                    }
                    _ => {
                        println!("Error: No such operator type.");
                        panic!();
                    }
                };
            }
            "*" => {
                match (*var_ref_cell).borrow().constant.as_ref().unwrap() {
                    Constant::Undefined => {
                        return construct_variable(Value {
                            value_type: ValueType::Undefined,
                            constant: None,
                            variable: None,
                        }, scope.clone());
                    }
                    Constant::Integer(i) => {
                        return construct_variable(Value {
                            value_type: ValueType::Constant,
                            constant: Some(Constant::Integer(i * (*right).borrow_mut().constant.as_ref().unwrap().as_integer())),
                            variable: None,
                        }, scope.clone());
                    }
                    _ => {
                        println!("Error: No such operator type.");
                        panic!();
                    }
                };
            }
            "*=" => {
                let constant;
                {
                    constant = var_ref_cell.as_ref().borrow().constant.clone().unwrap();
                }
                match constant {
                    Constant::Undefined => {
                        return construct_variable(Value {
                            value_type: ValueType::Undefined,
                            constant: None,
                            variable: None,
                        }, scope.clone());
                    }
                    Constant::Integer(i) => {
                        let result;
                        {
                            result = Some(Constant::Integer(i * (*right).borrow_mut().constant.as_ref().unwrap().as_integer()));
                        }
                        (*var_ref_cell).borrow_mut().constant = result;
                        return var_ref_cell.clone();
                    }
                    _ => {
                        println!("Error: No such operator type.");
                        panic!();
                    }
                };
            }
            "/" => {
                match (*var_ref_cell).borrow().constant.as_ref().unwrap() {
                    Constant::Undefined => {
                        return construct_variable(Value {
                            value_type: ValueType::Undefined,
                            constant: None,
                            variable: None,
                        }, scope.clone());
                    }
                    Constant::Integer(i) => {
                        return construct_variable(Value {
                            value_type: ValueType::Constant,
                            constant: Some(Constant::Integer(i / (*right).borrow_mut().constant.as_ref().unwrap().as_integer())),
                            variable: None,
                        }, scope.clone());
                    }
                    _ => {
                        println!("Error: No such operator type.");
                        panic!();
                    }
                };
            }
            "/=" => {
                let constant;
                {
                    constant = var_ref_cell.as_ref().borrow().constant.clone().unwrap();
                }
                match constant {
                    Constant::Undefined => {
                        return construct_variable(Value {
                            value_type: ValueType::Undefined,
                            constant: None,
                            variable: None,
                        }, scope.clone());
                    }
                    Constant::Integer(i) => {
                        let result;
                        {
                            result = Some(Constant::Integer(i / (*right).borrow_mut().constant.as_ref().unwrap().as_integer()));
                        }
                        (*var_ref_cell).borrow_mut().constant = result;
                        return var_ref_cell.clone();
                    }
                    _ => {
                        println!("Error: No such operator type.");
                        panic!();
                    }
                };
            }
            _ => {
                println!("Error: Unknown operator '{}'", operator);
                panic!();
            }
        }
    }

    fn apply_operator_right_vn(var_ref_cell: Rc<RefCell<Variable>>, right: &String, operator: &String, _accessible_variables: Rc<RefCell<Scope>>) -> Rc<RefCell<Variable>> {
        match operator.as_str() {
            "." => {
                let deref = var_ref_cell.clone();

                for field_var in deref.deref().borrow().members.iter() {
                    if field_var.deref().borrow_mut().name.as_ref().unwrap().eq(right.as_str()) {
                        return field_var.clone();
                    }
                }

                println!("Error: No such member in variable");
                panic!();
            }
            _ => {
                println!("Error: Unknown operator for variable names '{}'", operator);
                panic!();
            }
        }
    }
}

trait Convertible {
    fn as_integer(&self) -> i64;
    fn as_tuple(&self) -> Rc<RefCell<Tuple>>;
}

pub trait Callable {
    fn call(&mut self, args: Rc<RefCell<Tuple>>, scope: Rc<RefCell<Scope>>) -> Rc<RefCell<Variable>>;
    fn get_args(&self) -> Option<&Vec<String>>;
    fn get_scope(&self) -> Option<Rc<RefCell<Scope>>>;
}

impl Dumpable for Callable {
    fn get_dump(&self) -> String {
        let mut str = "function( ".to_string();
        if self.get_args().is_some() {
            for s in self.get_args().unwrap().iter() {
                str.push_str(s.as_str());
                str.push_str(" ");
            }
        }
        else {
            str.push_str("Undefined Arguments");
        }
        str.push_str(")");
        if self.get_scope().is_some() {
            str.push_str(self.get_scope().as_ref().unwrap().deref().borrow().get_dump().as_str());
        }
        else {
            str.push_str("{Native Code}");
        }
        return str;
    }

    fn dump(&self) {
        println!("{}", self.get_dump());
    }
}

impl Convertible for Constant {
    fn as_integer(&self) -> i64 {
        match self {
            Constant::Integer(i) => {
                return *i;
            }
            _ => {
                unimplemented!();
            }
        };
    }

    fn as_tuple(&self) -> Rc<RefCell<Tuple>> {
        match self {
            Constant::Tuple(t) => {
                return t.clone();
            }
            _ => {
                unimplemented!();
            }
        };
    }
}

impl Callable for Function {
    fn call(&mut self, args: Rc<RefCell<Tuple>>, scope: Rc<RefCell<Scope>>) -> Rc<RefCell<Variable>> {
        let mut scope_to_exec = (*self.scope).borrow().clone();
        for i in 0..self.args.len() {
            if i >= (*args).borrow().expressions.len() {
                scope_to_exec.accessible_variables.push(Rc::new(RefCell::new(Variable {
                    name: Some(self.args.get(i).unwrap().clone()),
                    constant: Some(Constant::Undefined),
                    members: vec![]
                })));
            }
            else {
                let var = Rc::new(RefCell::new(Variable {
                    name: Some(self.args.get(i).unwrap().clone()),
                    constant: Some(Constant::Undefined),
                    members: vec![]
                }));
                var.deref().borrow_mut().assign(execute_expression((*args).borrow().expressions.get(i).unwrap(), scope.clone()).to_variable(scope.clone()));
                scope_to_exec.accessible_variables.push(var);
            }
        }

        scope_to_exec.parent = Some(scope.clone());
        let scope_to_exec_rc = Rc::new(RefCell::new(scope_to_exec));
        execute_scope(scope_to_exec_rc.clone());
        if (*scope_to_exec_rc).borrow().return_value.is_some() {
            return (*scope_to_exec_rc).borrow().return_value.as_ref().unwrap().clone();
        }
        else {
            return Rc::new(RefCell::new(Variable {
                name: None,
                constant: Some(Constant::Undefined),
                members: vec![]
            }));
        }
    }

    fn get_args(&self) -> Option<&Vec<String>> {
        return Some(&self.args);
    }

    fn get_scope(&self) -> Option<Rc<RefCell<Scope>>> {
        return Some(self.scope.clone());
    }
}

pub struct PrintFunction;

impl Callable for PrintFunction {
    fn call(&mut self, args: Rc<RefCell<Tuple>>, scope: Rc<RefCell<Scope>>) -> Rc<RefCell<Variable>> {
        for e in (*args).borrow().expressions.iter() {
            execute_expression(e, scope.clone()).to_variable(scope.clone()).deref().borrow().dump();
        }
        return Rc::new(RefCell::new(Variable {
            name: None,
            constant: Some(Constant::Undefined),
            members: vec![]
        }))
    }

    fn get_args(&self) -> Option<&Vec<String>> {
        return None;
    }

    fn get_scope(&self) -> Option<Rc<RefCell<Scope>>> {
        return None;
    }
}