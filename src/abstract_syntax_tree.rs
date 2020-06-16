use std::borrow::{Borrow, BorrowMut};
use crate::abstract_syntax_tree::Statement::{VariableDeclaration, ReturnStatement};
use std::rc::Rc;
use std::cell::{RefCell};
use crate::executor::{Variable, Callable};
use std::ops::Deref;

pub struct Tuple {
    pub expressions: Vec<Expression>
}

impl Dumpable for Tuple {
    fn get_dump(&self) -> String {
        let mut str = "( ".to_string();
        for e in self.expressions.iter() {
            str.push_str(e.get_dump().as_str());
            str.push_str(", ");
        }
        str.push_str(")");
        return str;
    }

    fn dump(&self) {
        println!("{}", self.get_dump());
    }
}

pub struct Function {
    pub args: Vec<String>,
    pub scope: Rc<RefCell<Scope>>
}

pub enum ExpressionType {
    Undefined,
    Value,
    Operation,
}

impl Copy for ExpressionType {}

impl Clone for ExpressionType {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct Expression {
    pub expression_type: ExpressionType,
    pub left: Option<Box<Expression>>,
    pub right: Option<Box<Expression>>,
    pub value: Option<Value>,
    pub operator: Option<String>,
}

impl Parsable for Expression {
    fn parse(&mut self, tokens: &Vec<String>, operators: &Vec<String>, operator_priorities: &Vec<i32>, iterator: &mut i64, parse_end: i64) -> u8 {
        let mut expr_objs: Vec<ExprObj> = vec![];
        while *iterator < parse_end &&
            !tokens.get(*iterator as usize).unwrap().eq(";") &&
            !tokens.get(*iterator as usize).unwrap().eq("}") &&
            !tokens.get(*iterator as usize).unwrap().eq(",") &&
            !tokens.get(*iterator as usize).unwrap().eq(")") {
            let mut e = ExprObj {
                expr_obj_type: ExprObjType::Undefined,
                in_parentheses: None,
                in_value: None,
                in_operator: None,
            };

            e.parse(tokens, operators, operator_priorities, iterator, parse_end, &mut expr_objs);
            expr_objs.push(e);
        }

        fn create_expression_from_exprobjs(mut expression: &mut Expression, operators: &Vec<String>, operator_priorities: &Vec<i32>, expr_objs: &Vec<ExprObj>) {
            enum EOP {
                Expression(Expression),
                Operator(String),
            }

            let mut max_operator_priority = 0;
            let mut min_operator_priority = 0;
            for p in operator_priorities {
                if *p > max_operator_priority {
                    max_operator_priority = *p;
                }
                if *p < min_operator_priority {
                    min_operator_priority = *p;
                }
            }

            if expr_objs.is_empty() {
                println!("Error: Empty expression.");
                panic!();
            }

            if expr_objs.len() == 1 {
                let expr_obj = expr_objs.get(0).unwrap();
                match expr_obj.expr_obj_type {
                    ExprObjType::Operator => {
                        println!("Error: Found operator instead of value.");
                        panic!();
                    }
                    ExprObjType::Parentheses => {
                        create_expression_from_exprobjs(expression, operators, operator_priorities, expr_obj.in_parentheses.as_ref().unwrap());
                    }
                    ExprObjType::Value => {
                        expression.expression_type = ExpressionType::Value;
                        expression.value = expr_obj.in_value.clone();
                    }
                    ExprObjType::Undefined => {
                        panic!();
                    }
                }
                return;
            }

            let mut eops: Vec<EOP> = vec![];

            for eo in expr_objs.iter() {
                match eo.expr_obj_type {
                    ExprObjType::Operator => {
                        eops.push(EOP::Operator(eo.in_operator.as_ref().unwrap().clone().to_string()));
                    }
                    ExprObjType::Parentheses => {
                        let mut new_exp = Expression {
                            expression_type: ExpressionType::Undefined,
                            value: None,
                            left: None,
                            right: None,
                            operator: None,
                        };
                        create_expression_from_exprobjs(new_exp.borrow_mut(), operators, operator_priorities, eo.in_parentheses.as_ref().unwrap());

                        eops.push(EOP::Expression(new_exp));
                    }
                    ExprObjType::Value => {
                        eops.push(EOP::Expression(Expression {
                            expression_type: ExpressionType::Value,
                            value: eo.in_value.clone(),
                            left: None,
                            right: None,
                            operator: None,
                        }));
                    }
                    ExprObjType::Undefined => {
                        panic!();
                    }
                }
            }

            let mut current_operator_priority = max_operator_priority;

            while current_operator_priority > min_operator_priority - 1 {
                let mut eops_len = eops.len();
                let mut i = 0;
                while i < eops_len {
                    match eops[i].borrow().clone() {
                        EOP::Operator(operator_string) => {
                            if i == 0 {
                                println!("Error: Found operator instead of value.");
                                panic!();
                            }
                            if i == eops.len() - 1 {
                                println!("Error: Expression cannot end with an operator.");
                                panic!();
                            }

                            if get_operator_priority(operators, operator_priorities, operator_string) == current_operator_priority {
                                let mut left_expression: Option<Box<Expression>> = None;
                                let mut right_expression: Option<Box<Expression>> = None;

                                if let EOP::Expression(expression) = eops.get(i - 1).unwrap() {
                                    left_expression = Some(Box::from(expression.clone()));
                                }
                                if let EOP::Expression(expression) = eops.get(i + 1).unwrap() {
                                    right_expression = Some(Box::from(expression.clone()));
                                }

                                let operation_expression = Expression {
                                    expression_type: ExpressionType::Operation,
                                    value: None,
                                    left: left_expression,
                                    right: right_expression,
                                    operator: Some(operator_string.clone()),
                                };

                                eops[i] = EOP::Expression(operation_expression);
                                eops.remove(i + 1);
                                eops.remove(i - 1);

                                eops_len -= 2;
                                i -= 1;
                            }
                        }
                        EOP::Expression(_) => {}
                    }
                    i += 1;
                }

                current_operator_priority -= 1;
            }

            if let EOP::Expression(result) = eops.pop().unwrap() {
                *expression = result;
            }
        }
        create_expression_from_exprobjs(self, operators, operator_priorities, expr_objs.borrow());
        return 0;
    }
}

impl Clone for Expression {
    fn clone(&self) -> Expression {
        Expression {
            expression_type: self.expression_type.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
            value: self.value.clone(),
            operator: self.operator.clone(),
        }
    }
}

impl Dumpable for Expression {
    fn get_dump(&self) -> String {
        match self.expression_type {
            ExpressionType::Value => {
                return self.value.as_ref().unwrap().get_dump();
            }
            ExpressionType::Operation => {
                let mut str = "(".to_string();
                str.push_str(self.left.as_ref().unwrap().get_dump().as_str());
                str.push_str(" [");
                str.push_str(self.operator.as_ref().unwrap().as_str());
                str.push_str("] ");
                str.push_str(self.right.as_ref().unwrap().get_dump().as_str());
                str.push_str(")");
                return str;
            }
            ExpressionType::Undefined => {
                return "Undefined".to_string();
            }
        };
    }

    fn dump(&self) {
        println!("{}", self.get_dump());
    }
}

pub enum Constant{
    Undefined,
    Integer(i64),
    Function(Rc<RefCell<Callable>>),
    Tuple(Rc<RefCell<Tuple>>)
}

impl Clone for Constant {
    fn clone(&self) -> Self {
        return match self {
            Constant::Undefined => Constant::Undefined,
            Constant::Integer(i) => Constant::Integer(*i),
            Constant::Function(f) => Constant::Function(f.clone()),
            Constant::Tuple(t) => Constant::Tuple(t.clone())
        }
    }
}

impl Dumpable for Constant {
    fn get_dump(&self) -> String {
        return match self {
            Constant::Undefined => "Undefined".to_string(),
            Constant::Integer(i) => i.to_string(),
            Constant::Function(f) => f.deref().borrow().get_dump(),
            Constant::Tuple(t) => t.deref().borrow().get_dump()
        };
    }

    fn dump(&self) {
        println!("{}", self.get_dump());
    }
}

pub enum ValueType {
    Undefined,
    VariableName,
    Constant,
}

impl Copy for ValueType {}

impl Clone for ValueType {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct Value {
    pub value_type: ValueType,
    pub variable: Option<String>,
    pub constant: Option<Constant>,
}

impl Clone for Value {
    fn clone(&self) -> Self {
        let result = Value {
            value_type: self.value_type,
            variable: self.variable.clone(),
            constant: self.constant.clone(),
        };

        return result;
    }
}

impl Dumpable for Value {
    fn get_dump(&self) -> String {
        return match self.value_type {
            ValueType::Undefined => "Undefined".to_string(),
            ValueType::Constant => {
                let mut str = "const : ".to_string();
                str.push_str(self.constant.as_ref().unwrap().get_dump().as_str());
                return str;
            }
            ValueType::VariableName => {
                let mut str = "var : ".to_string();
                str.push_str(self.variable.as_ref().unwrap().as_str());
                return str;
            }
        };
    }

    fn dump(&self) {
        println!("{}", self.get_dump());
    }
}

enum ExprObjType {
    Undefined,
    Parentheses,
    Operator,
    Value,
}

struct ExprObj {
    expr_obj_type: ExprObjType,
    in_parentheses: Option<Vec<ExprObj>>,
    in_operator: Option<String>,
    in_value: Option<Value>,
}

impl ExprObj {
    fn parse(&mut self, tokens: &Vec<String>, operators: &Vec<String>, operator_priorities: &Vec<i32>, iterator: &mut i64, parse_end: i64, all: &mut Vec<ExprObj>) -> u8 {
        let mut token: &String = tokens.get(*iterator as usize).unwrap();

        if token.eq("function") {
            *iterator += 2;
            let mut args: Vec<String> = vec![];
            let mut expected_variable_name = true;
            while !token.to_string().eq(")") {
                if expected_variable_name {
                    let mut e = ExprObj {
                        expr_obj_type: ExprObjType::Undefined,
                        in_parentheses: None,
                        in_value: None,
                        in_operator: None,
                    };
                    e.parse(tokens, operators, operator_priorities, iterator, parse_end, all);
                    match e.expr_obj_type {
                        ExprObjType::Value => {
                            match e.in_value.as_ref().unwrap().value_type {
                                ValueType::VariableName => {
                                    args.push(e.in_value.unwrap().variable.unwrap());
                                },
                                _ => {
                                    println!("Error: expected variable name.");
                                    panic!();
                                }
                            }
                        },
                        _ => {
                            println!("Error: expected variable name.");
                            panic!();
                        }
                    }
                }
                else {
                    if !token.eq(",") {
                        println!("Error: Expected operator ','.");
                        panic!();
                    }
                    *iterator += 1;
                }
                expected_variable_name = !expected_variable_name;

                token = tokens.get(*iterator as usize).unwrap();
            }

            *iterator += 2;

            self.expr_obj_type = ExprObjType::Value;
            self.in_value = Some(Value{
                value_type: ValueType::Constant,
                variable: None,
                constant: Some(Constant::Function(Rc::new(RefCell::new(Function{
                    scope: Rc::new(RefCell::new(Scope::parse(tokens, operators, operator_priorities, iterator, parse_end))),
                    args
                }))))
            });

            *iterator += 1;
            return 0;
        }

        let previous = all.last();
        if token.eq("(") {
            if previous.is_some() {
                if let ExprObjType::Operator = previous.unwrap().expr_obj_type {}
                else {
                    // Function call
                    *iterator += 1;
                    token = tokens.get(*iterator as usize).unwrap();
                    let mut exprs: Vec<Expression> = vec![];
                    let mut expected_expression = true;
                    while !token.to_string().eq(")") {
                        if expected_expression {
                            let mut e = Expression {
                                expression_type: ExpressionType::Undefined,
                                value: None,
                                operator: None,
                                left: None,
                                right: None
                            };
                            e.parse(tokens, operators, operator_priorities, iterator, parse_end);
                            exprs.push(e);
                        }
                        else {
                            if !token.eq(",") {
                                println!("Error: Expected operator ','.");
                                panic!();
                            }
                            *iterator += 1;
                        }
                        expected_expression = !expected_expression;

                        token = tokens.get(*iterator as usize).unwrap();
                    }

                    self.expr_obj_type = ExprObjType::Value;
                    self.in_value = Some(Value {
                        value_type: ValueType::Constant,
                        variable: None,
                        constant: Some(Constant::Tuple(Rc::new(RefCell::new(Tuple {
                            expressions: exprs
                        }))))
                    });

                    all.push(ExprObj {
                        expr_obj_type: ExprObjType::Operator,
                        in_value: None,
                        in_parentheses: None,
                        in_operator: Some("(".to_string())
                    });

                    *iterator += 1;
                    return 0;
                }
            }

            // Mathematical parentheses
            *iterator += 1;
            token = tokens.get(*iterator as usize).unwrap();
            let mut expr_objs: Vec<ExprObj> = vec![];
            while !token.to_string().eq(")") {
                let mut e = ExprObj {
                    expr_obj_type: ExprObjType::Undefined,
                    in_parentheses: None,
                    in_value: None,
                    in_operator: None,
                };

                e.parse(tokens, operators, operator_priorities, iterator, parse_end, all);
                expr_objs.push(e);
                token = tokens.get(*iterator as usize).unwrap();
            }
            self.expr_obj_type = ExprObjType::Parentheses;
            self.in_parentheses = Some(expr_objs);

            *iterator += 1;
            return 0;
        }

        if operator_exists(operators, token) {
            self.expr_obj_type = ExprObjType::Operator;
            self.in_operator = Some(token.to_string());

            *iterator += 1;
            return 0;
        } else {
            let test = token.parse::<i64>();
            let is_integer: bool = test.is_ok();
            if is_integer {
                self.expr_obj_type = ExprObjType::Value;
                self.in_value = Some(Value {
                    value_type: ValueType::Constant,
                    variable: None,
                    constant: Some(Constant::Integer(test.ok().unwrap())
                    ),
                });

                *iterator += 1;
                return 0;
            }

            self.expr_obj_type = ExprObjType::Value;
            self.in_value = Some(Value{
                value_type: ValueType::VariableName,
                variable: Some(token.to_string()),
                constant: None
            });

            *iterator += 1;
            return 0;
        }
    }
}

impl Dumpable for ExprObj {
    fn get_dump(&self) -> String {
        match self.expr_obj_type {
            ExprObjType::Undefined => {
                return "Undefined".to_string();
            }
            ExprObjType::Parentheses => {
                let mut str = "(\n".to_string();
                for e in self.in_parentheses.as_ref().unwrap().iter() {
                    str.push_str(e.get_dump().as_str());
                    str.push_str("\n");
                }
                str.push_str(")");
                return str;
            }
            ExprObjType::Operator => {
                let mut str = "Operator ".to_string();
                str.push_str(self.in_operator.as_ref().unwrap().as_str());
                return str;
            }
            ExprObjType::Value => {
                let mut str = "Value ".to_string();
                str.push_str(self.in_value.as_ref().unwrap().get_dump().as_str());
                return str;
            }
        }
    }

    fn dump(&self) {
        println!("{}", self.get_dump());
    }
}

pub enum Statement {
    Undefined,
    Expression(Expression),
    VariableDeclaration(Expression),
    ReturnStatement(Expression)
}

impl Statement {
    pub fn parse(tokens: &Vec<String>, operators: &Vec<String>, operator_priorities: &Vec<i32>, iterator: &mut i64, parse_end: i64) -> Statement {
        match tokens.get(*iterator as usize).unwrap().as_str() {
            "let" => {
                *iterator += 1;
                let mut expression = Expression{
                    expression_type: ExpressionType::Undefined,
                    left: None,
                    right: None,
                    value: None,
                    operator: None
                };
                expression.parse(tokens, operators, operator_priorities, iterator, parse_end);
                return VariableDeclaration(expression);
            },
            "return" => {
                *iterator += 1;
                let mut expression = Expression{
                    expression_type: ExpressionType::Undefined,
                    left: None,
                    right: None,
                    value: None,
                    operator: None
                };
                expression.parse(tokens, operators, operator_priorities, iterator, parse_end);
                return ReturnStatement(expression);
            }
            _ => {
                let mut expression = Expression{
                    expression_type: ExpressionType::Undefined,
                    left: None,
                    right: None,
                    value: None,
                    operator: None
                };
                expression.parse(tokens, operators, operator_priorities, iterator, parse_end);
                return Statement::Expression(expression);
            }
        };
    }
}

impl Clone for Statement {
    fn clone(&self) -> Self {
        match self {
            Statement::Undefined => {
                return Statement::Undefined;
            },
            Statement::Expression(e) => {
                return Statement::Expression(e.clone());
            },
            Statement::VariableDeclaration(e) => {
                return Statement::VariableDeclaration(e.clone());
            },
            Statement::ReturnStatement(e) => {
                return Statement::ReturnStatement(e.clone());
            }
        };
    }
}

impl Dumpable for Statement {
    fn get_dump(&self) -> String {
        match self {
            Statement::Undefined => {
                return "[Undefined]".to_string();
            },
            Statement::Expression(expression) => {
                let mut result = "[expression : ".to_string();
                result += expression.get_dump().as_str();
                result += "]";
                return result;
            },
            Statement::VariableDeclaration(expression) => {
                let mut result = "[let : ".to_string();
                result += expression.get_dump().as_str();
                result += "]";
                return result;
            },
            Statement::ReturnStatement(expression) => {
                let mut result = "[return : ".to_string();
                result += expression.get_dump().as_str();
                result += "]";
                return result;
            }
        }
    }

    fn dump(&self) {
        println!("{}", self.get_dump());
    }
}

pub struct Scope {
    pub statements: Vec<Statement>,
    pub parent: Option<Rc<RefCell<Scope>>>,
    pub accessible_variables: Vec<Rc<RefCell<Variable>>>,
    pub return_value: Option<Rc<RefCell<Variable>>>
}

impl Scope {
    pub fn parse(tokens: &Vec<String>, operators: &Vec<String>, operator_priorities: &Vec<i32>, iterator: &mut i64, parse_end: i64) -> Scope {
        let mut result_statements: Vec<Statement> = vec![];
        while *iterator < parse_end{
            if tokens.get(*iterator as usize).unwrap().eq("}") {
                break;
            }
            result_statements.push(Statement::parse(tokens, operators, operator_priorities, iterator, parse_end));
            *iterator += 1;
        }
        return Scope {
            statements: result_statements,
            parent: Some(Rc::new(RefCell::new(Scope {
                statements: vec![],
                parent: None,
                accessible_variables: vec![Rc::new(RefCell::new(Variable {
                    name: Some("variable_in_parent".to_string()),
                    constant: Some(Constant::Integer(15)),
                    members: vec![]
                }))],
                return_value: None
            }))),
            accessible_variables: vec![],
            return_value: None
        };
    }
}

impl Clone for Scope {
    fn clone(&self) -> Self {
        return Scope {
            accessible_variables: self.accessible_variables.clone(),
            statements: self.statements.clone(),
            parent: self.parent.clone(),
            return_value: self.return_value.clone()
        }
    }
}

impl Dumpable for Scope {
    fn get_dump(&self) -> String {
        let mut str = "{\n".to_string();
        for s in self.statements.iter() {
            str.push_str(s.get_dump().as_str());
            str.push_str("\n");
        }
        str.push_str("---\n");
        for v in self.accessible_variables.iter() {
            str.push_str(v.deref().borrow().get_dump().as_str());
            str.push_str("\n");
        }
        str.push_str("}");

        return str;
    }

    fn dump(&self) {
        println!("{}", self.get_dump());
    }
}

pub trait Parsable {
    fn parse(&mut self, tokens: &Vec<String>, operators: &Vec<String>, operator_priorities: &Vec<i32>, iterator: &mut i64, parse_end: i64) -> u8;
}

pub trait Dumpable {
    fn get_dump(&self) -> String;
    fn dump(&self);
}

fn get_operator_priority(operators: &Vec<String>, operator_priorities: &Vec<i32>, operator: &String) -> i32 {
    let mut opsi = 0;
    for op_search in operators.iter() {
        if op_search.eq(operator) {
            return *operator_priorities.get(opsi).unwrap();
        }
        opsi += 1;
    }
    return 0;
}

fn operator_exists(operators: &Vec<String>, operator: &String) -> bool {
    for op_search in operators.iter() {
        if op_search.eq(operator) {
            return true;
        }
    }

    return false;
}