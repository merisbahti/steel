use crate::env::Env;
use crate::parser::Expr;
use crate::rerrs::RucketErr;
use crate::tokens::Token::*;
// use std::any::Any;
use std::any::Any;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::rc::Rc;
use RucketVal::*;

use std::convert::TryFrom;

pub trait CustomType {
    fn box_clone(&self) -> Box<dyn CustomType>;
    fn as_any(&self) -> Box<dyn Any>;
    fn name(&self) -> String {
        (std::any::type_name::<Self>()).to_string()
    }
    fn new_rucket_val(&self) -> RucketVal;
}

impl Clone for Box<dyn CustomType> {
    fn clone(&self) -> Box<dyn CustomType> {
        self.box_clone()
    }
}

#[macro_export]
macro_rules! unwrap {
    ($x:expr, $body:ty) => {{
        if let crate::rvals::RucketVal::Custom(v) = $x {
            let left_type = (*v).as_any();
            let left = left_type.downcast_ref::<$body>();
            // left.map
            left.map(|x| x.clone())
                .ok_or_else(|| crate::rerrs::RucketErr::ConversionError("blah".to_string()))
        } else {
            Err(crate::rerrs::RucketErr::ConversionError("blah".to_string()))
        }
    }};
}

#[derive(Clone)]
pub enum RucketVal {
    BoolV(bool),
    NumV(f64),
    ListV(Vec<RucketVal>),
    Void,
    StringV(String),
    FuncV(fn(Vec<RucketVal>) -> Result<RucketVal, RucketErr>),
    LambdaV(RucketLambda),
    SymbolV(String),
    Custom(Box<dyn CustomType>),
}

// sometimes you want to just
// return an expression
impl TryFrom<Expr> for RucketVal {
    type Error = RucketErr;
    fn try_from(e: Expr) -> Result<Self, Self::Error> {
        match e {
            Expr::Atom(a) => match a {
                OpenParen => Err(RucketErr::UnexpectedToken("(".to_string())),
                CloseParen => Err(RucketErr::UnexpectedToken(")".to_string())),
                QuoteTick => Err(RucketErr::UnexpectedToken("'".to_string())),
                BooleanLiteral(x) => Ok(BoolV(x)),
                Identifier(x) => Ok(SymbolV(x)),
                NumberLiteral(x) => Ok(NumV(x)),
                StringLiteral(x) => Ok(StringV(x)),
            },
            Expr::ListVal(lst) => {
                let items: Result<Vec<Self>, Self::Error> =
                    lst.into_iter().map(Self::try_from).collect();
                Ok(ListV(items?))
            }
        }
    }
}

// TODO add tests
impl PartialEq for RucketVal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (BoolV(l), BoolV(r)) => l == r,
            (NumV(l), NumV(r)) => l == r,
            (StringV(l), StringV(r)) => l == r,
            (ListV(l), ListV(r)) => l == r,
            // (Custom(l), Custom(r)) => {
            // let left_type = (*l).as_any();
            // let left = left_type.downcast_ref::<usize>();
            // let right_type = (*r).as_any();
            // let right = right_type.downcast_ref::<usize>();
            // let left: Option<&usize> = type_check!(l, usize);
            // let right: Option<&usize> = type_check!(r, usize);
            // left_type == right_type;
            // match (left, right) {
            // (Some(lt), Some(rt)) => lt == rt,
            // (_, _) => false,
            // }
            // }
            (l, r) => {
                let left = unwrap!(l, usize);
                let right = unwrap!(r, usize);
                match (left, right) {
                    (Ok(l), Ok(r)) => l == r,
                    (_, _) => false,
                }
            }
        }
    }
}

// TODO add tests
impl PartialOrd for RucketVal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (NumV(n), NumV(o)) => n.partial_cmp(o),
            (StringV(s), StringV(o)) => s.partial_cmp(o),
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone)]
/// struct representing data required to describe a scheme function
pub struct RucketLambda {
    /// symbols representing the arguments to the function
    params_exp: Vec<String>,
    /// body of the function with identifiers yet to be bound
    body_exp: Expr,
    /// parent environment that created this Lambda.
    /// the actual environment with correct bindingsis built at runtime
    /// once the function is called
    parent_env: Rc<RefCell<Env>>,
}
impl RucketLambda {
    pub fn new(
        params_exp: Vec<String>,
        body_exp: Expr,
        parent_env: Rc<RefCell<Env>>,
    ) -> RucketLambda {
        RucketLambda {
            params_exp,
            body_exp,
            parent_env,
        }
    }
    /// symbols representing the arguments to the function
    pub fn params_exp(&self) -> &[String] {
        &self.params_exp
    }
    /// body of the function with identifiers yet to be bound
    pub fn body_exp(&self) -> Expr {
        self.body_exp.clone()
    }
    /// parent environment that created this Lambda.
    ///
    /// The actual environment with correct bindings is built at runtime
    /// once the function is called
    pub fn parent_env(&self) -> &Rc<RefCell<Env>> {
        &self.parent_env
    }
}

impl fmt::Display for RucketVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // at the top level, print a ' if we are
        // trying to print a symbol or list
        match self {
            SymbolV(_) | ListV(_) => write!(f, "'")?,
            _ => (),
        };
        display_helper(self, f)
    }
}

/// this function recursively prints lists without prepending the `'`
/// at the beginning
fn display_helper(val: &RucketVal, f: &mut fmt::Formatter) -> fmt::Result {
    match val {
        BoolV(b) => write!(f, "#{}", b),
        NumV(x) => write!(f, "{}", x),
        StringV(s) => write!(f, "\"{}\"", s),
        FuncV(_) => write!(f, "Function"),
        LambdaV(_) => write!(f, "Lambda Function"),
        Void => write!(f, "Void"),
        SymbolV(s) => write!(f, "{}", s),
        ListV(lst) => {
            let mut iter = lst.iter();
            write!(f, "(")?;
            if let Some(last) = iter.next_back() {
                for item in iter {
                    display_helper(item, f)?;
                    write!(f, " ")?;
                }
                display_helper(last, f)?;
            }
            write!(f, ")")
        }
        Custom(x) => write!(f, "Custom Type: {}", x.name()),
    }
}

#[test]
fn display_test() {
    use crate::tokens::Token;
    assert_eq!(RucketVal::BoolV(false).to_string(), "#false");
    assert_eq!(RucketVal::NumV(1.0).to_string(), "1");
    assert_eq!(
        RucketVal::FuncV(|_args: Vec<RucketVal>| -> Result<RucketVal, RucketErr> {
            Ok(RucketVal::ListV(vec![]))
        })
        .to_string(),
        "Function"
    );
    assert_eq!(
        RucketVal::LambdaV(RucketLambda::new(
            vec!["arg1".to_owned()],
            Expr::Atom(Token::NumberLiteral(1.0)),
            Rc::new(RefCell::new(crate::env::Env::default_env())),
        ))
        .to_string(),
        "Lambda Function"
    );
    assert_eq!(RucketVal::SymbolV("foo".to_string()).to_string(), "'foo");
}

#[test]
fn display_list_test() {
    use crate::tokens::Token;
    assert_eq!(ListV(vec![]).to_string(), "'()");
    assert_eq!(
        ListV(vec![
            BoolV(false),
            NumV(1.0),
            LambdaV(RucketLambda::new(
                vec!["arg1".to_owned()],
                Expr::Atom(Token::NumberLiteral(1.0)),
                Rc::new(RefCell::new(crate::env::Env::default_env())),
            ))
        ])
        .to_string(),
        "'(#false 1 Lambda Function)"
    );
    assert_eq!(
        ListV(vec![
            ListV(vec![NumV(1.0), ListV(vec!(NumV(2.0), NumV(3.0)))]),
            ListV(vec![NumV(4.0), NumV(5.0)]),
            NumV(6.0),
            ListV(vec![NumV(7.0)])
        ])
        .to_string(),
        "'((1 (2 3)) (4 5) 6 (7))"
    );
}
