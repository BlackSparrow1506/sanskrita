// value.rs — runtime values (वेगः engine), slices 1–6.
//
// Lists, maps and objects are shared and mutable, so they live behind
// Rc<RefCell<…>> — assigning a सूची to another name refers to the SAME list,
// exactly as in the Python reference.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{Method, Param, Stmt};
use crate::bigint::BigInt;
use crate::decimal::Decimal;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub parent: Option<Rc<Class>>,
    pub methods: HashMap<String, Rc<Function>>,
}

impl Class {
    /// Look a method up through the inheritance chain.
    pub fn find_method(&self, name: &str) -> Option<Rc<Function>> {
        if let Some(m) = self.methods.get(name) {
            return Some(m.clone());
        }
        match &self.parent {
            Some(p) => p.find_method(name),
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct Instance {
    pub class: Rc<Class>,
    pub fields: RefCell<HashMap<String, Value>>,
}

/// Map keys: text or whole numbers (matching the reference).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Str(String),
    Int(String), // canonical decimal digits of the integer
}

#[derive(Debug, Clone)]
pub enum Value {
    /// पूर्णाङ्कः — arbitrary-precision integer
    Int(BigInt),
    /// दशमांशः — exact decimal
    Dec(Decimal),
    Str(String),
    Bool(bool),
    Nil,
    Func(Rc<Function>),
    /// सूची — shared, mutable list
    List(Rc<RefCell<Vec<Value>>>),
    /// कोशः — shared, mutable map (insertion-ordered like the reference's dict)
    Map(Rc<RefCell<MapData>>),
    Class(Rc<Class>),
    Object(Rc<Instance>),
    /// A method bound to its object: `वस्तु.विधि`
    Bound(Rc<Instance>, Rc<Function>),
}

/// Insertion-ordered map: Python dicts preserve insertion order, and our
/// कुञ्जिकाः() output must match, so we keep the order explicitly.
#[derive(Debug, Default)]
pub struct MapData {
    pub order: Vec<Key>,
    pub items: HashMap<Key, Value>,
}

impl MapData {
    pub fn get(&self, k: &Key) -> Option<&Value> {
        self.items.get(k)
    }

    pub fn insert(&mut self, k: Key, v: Value) {
        if !self.items.contains_key(&k) {
            self.order.push(k.clone());
        }
        self.items.insert(k, v);
    }

    pub fn remove(&mut self, k: &Key) -> Option<Value> {
        if let Some(v) = self.items.remove(k) {
            self.order.retain(|o| o != k);
            Some(v)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl Value {
    pub fn int(v: i64) -> Value {
        Value::Int(BigInt::from_i64(v))
    }

    pub fn list(items: Vec<Value>) -> Value {
        Value::List(Rc::new(RefCell::new(items)))
    }

    /// Numeric view for arithmetic: integers promote to decimals when mixed.
    pub fn as_decimal(&self) -> Option<Decimal> {
        match self {
            Value::Int(b) => Some(Decimal::from_bigint(b.clone())),
            Value::Dec(d) => Some(d.clone()),
            _ => None,
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Int(_) | Value::Dec(_))
    }

    /// Convert to a map key (text or whole number only).
    pub fn as_key(&self) -> Option<Key> {
        match self {
            Value::Str(s) => Some(Key::Str(s.clone())),
            Value::Int(b) => Some(Key::Int(b.to_string_signed())),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "पूर्णाङ्कः",
            Value::Dec(_) => "दशमांशः",
            Value::Str(_) => "वाक्यम्",
            Value::Bool(_) => "सत्यासत्यम्",
            Value::Nil => "शून्यम्",
            Value::Func(_) | Value::Bound(..) => "विधिः",
            Value::List(_) => "सूची",
            Value::Map(_) => "कोशः",
            Value::Class(_) => "वर्गः",
            Value::Object(_) => "वस्तु",
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            _ if self.is_number() && other.is_number() => {
                match (self.as_decimal(), other.as_decimal()) {
                    (Some(a), Some(b)) => a.eq_value(&b),
                    _ => false,
                }
            }
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Func(a), Value::Func(b)) => Rc::ptr_eq(a, b),
            (Value::List(a), Value::List(b)) => {
                if Rc::ptr_eq(a, b) {
                    return true;
                }
                let (a, b) = (a.borrow(), b.borrow());
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x == y)
            }
            (Value::Map(a), Value::Map(b)) => {
                if Rc::ptr_eq(a, b) {
                    return true;
                }
                let (a, b) = (a.borrow(), b.borrow());
                a.len() == b.len()
                    && a.order.iter().all(|k| match (a.get(k), b.get(k)) {
                        (Some(x), Some(y)) => x == y,
                        _ => false,
                    })
            }
            (Value::Class(a), Value::Class(b)) => Rc::ptr_eq(a, b),
            (Value::Object(a), Value::Object(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
