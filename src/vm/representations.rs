use std::fmt;
use std::borrow::Borrow;
use std::ops::Deref;
use std::convert::TryFrom;
use regex::{Regex};
use vm::types::{Value, TraitIdentifier, QualifiedContractIdentifier};
use vm::errors::{RuntimeErrorType};

pub const MAX_STRING_LEN: u8 = 128;

macro_rules! guarded_string {
    ($Name:ident, $Label:literal, $Regex:expr) => {
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $Name (String);
        impl TryFrom<String> for $Name {
            type Error = RuntimeErrorType;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                if value.len() > (MAX_STRING_LEN as usize) {
                    return Err(RuntimeErrorType::BadNameValue($Label, value))
                }
                // TODO: use lazy static ?
                let regex_check = $Regex
                    .expect("FAIL: Bad static regex.");
                if regex_check.is_match(&value) {
                    Ok(Self(value))
                } else {
                    Err(RuntimeErrorType::BadNameValue($Label, value))
                }
            }
        }

        impl $Name {
            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn len(&self) -> u8 {
                u8::try_from(self.as_str().len()).unwrap()
            }
        }

        impl Deref for $Name {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl Borrow<str> for $Name {
            fn borrow(&self) -> &str {
                self.as_str()
            }
        }

        impl Into<String> for $Name {
            fn into(self) -> String {
                self.0
            }
        }

        #[cfg(test)]
        impl From<&'_ str> for $Name {
            fn from(value: &str) -> Self {
                Self::try_from(value.to_string()).unwrap()
            }
        }
    }
}

guarded_string!(ClarityName, "ClarityName", Regex::new("^[a-zA-Z]([a-zA-Z0-9]|[-_!?+<>=/*])*$|^[-+=/*]$|^[<>]=?$"));
guarded_string!(ContractName, "ContractName", Regex::new("^[a-zA-Z]([a-zA-Z0-9]|[-_])*$|^__transient$"));
guarded_string!(UrlString, "UrlString", Regex::new(r#"^[a-zA-Z0-9._~:/?#\[\]@!$&'()*+,;%=-]*$"#));

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum PreSymbolicExpressionType {
    AtomValue(Value),
    Atom(ClarityName),
    List(Box<[PreSymbolicExpression]>),
    SugaredContractIdentifier(ContractName),
    SugaredFieldIdentifier(ContractName, ClarityName),
    FieldIdentifier(TraitIdentifier),
    TraitReference(ClarityName),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PreSymbolicExpression {
    pub pre_expr: PreSymbolicExpressionType,
    pub id: u64,

    #[cfg(feature = "developer-mode")]
    pub span: Span,
}

impl PreSymbolicExpression {
    #[cfg(feature = "developer-mode")]
    fn cons() -> PreSymbolicExpression {
        PreSymbolicExpression {
            id: 0,
            span: Span::zero(),
            pre_expr: PreSymbolicExpressionType::AtomValue(Value::Bool(false))
        }
    }
    #[cfg(not(feature = "developer-mode"))]
    fn cons() -> PreSymbolicExpression {
        PreSymbolicExpression {
            id: 0,
            pre_expr: PreSymbolicExpressionType::AtomValue(Value::Bool(false))
        }
    }

    #[cfg(feature = "developer-mode")]
    pub fn set_span(&mut self, start_line: u32, start_column: u32, end_line: u32, end_column: u32) {
        self.span = Span {
            start_line,
            start_column,
            end_line,
            end_column
        }
    }

    #[cfg(not(feature = "developer-mode"))]
    pub fn set_span(&mut self, _start_line: u32, _start_column: u32, _end_line: u32, _end_column: u32) {
    }

    pub fn sugared_contract_identifier(val: ContractName) -> PreSymbolicExpression {
        PreSymbolicExpression {
            pre_expr: PreSymbolicExpressionType::SugaredContractIdentifier(val),
            .. PreSymbolicExpression::cons()
        }
    }

    pub fn sugared_field_identifier(contract_name: ContractName, name: ClarityName) -> PreSymbolicExpression {
        PreSymbolicExpression {
            pre_expr: PreSymbolicExpressionType::SugaredFieldIdentifier(contract_name, name),
            .. PreSymbolicExpression::cons()
        }
    }

    pub fn atom_value(val: Value) -> PreSymbolicExpression {
        PreSymbolicExpression {
            pre_expr: PreSymbolicExpressionType::AtomValue(val),
            .. PreSymbolicExpression::cons()
        }
    }

    pub fn atom(val: ClarityName) -> PreSymbolicExpression {
        PreSymbolicExpression {
            pre_expr: PreSymbolicExpressionType::Atom(val),
            .. PreSymbolicExpression::cons()
        }
    }

    pub fn trait_reference(val: ClarityName) -> PreSymbolicExpression {
        PreSymbolicExpression {
            pre_expr: PreSymbolicExpressionType::TraitReference(val),
            .. PreSymbolicExpression::cons()
        }
    }

    pub fn field_identifier(val: TraitIdentifier) -> PreSymbolicExpression {
        PreSymbolicExpression {
            pre_expr: PreSymbolicExpressionType::FieldIdentifier(val),
            .. PreSymbolicExpression::cons()
        }
    }

    pub fn list(val: Box<[PreSymbolicExpression]>) -> PreSymbolicExpression {
        PreSymbolicExpression {
            pre_expr: PreSymbolicExpressionType::List(val),
            .. PreSymbolicExpression::cons()
        }
    }

    pub fn match_trait_reference(&self) -> Option<&ClarityName> {
        if let PreSymbolicExpressionType::TraitReference(ref value) = self.pre_expr {
            Some(value)
        } else {
            None
        }
    }

    pub fn match_atom_value(&self) -> Option<&Value> {
        if let PreSymbolicExpressionType::AtomValue(ref value) = self.pre_expr {
            Some(value)
        } else {
            None
        }
    }

    pub fn match_atom(&self) -> Option<&ClarityName> {
        if let PreSymbolicExpressionType::Atom(ref value) = self.pre_expr {
            Some(value)
        } else {
            None
        }
    }

    pub fn match_list(&self) -> Option<&[PreSymbolicExpression]> {
        if let PreSymbolicExpressionType::List(ref list) = self.pre_expr {
            Some(list)
        } else {
            None
        }
    }

    pub fn match_field_identifier(&self) -> Option<&TraitIdentifier> {
        if let PreSymbolicExpressionType::FieldIdentifier(ref value) = self.pre_expr {
            Some(value)
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum SymbolicExpressionType {
    AtomValue(Value),
    Atom(ClarityName),
    List(Box<[SymbolicExpression]>),
    LiteralValue(Value),
    Field(TraitIdentifier),
    TraitReference(ClarityName, TraitDefinition),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TraitDefinition {
    Defined(TraitIdentifier),
    Imported(TraitIdentifier)
}

pub fn depth_traverse<F,T,E>(expr: &SymbolicExpression, mut visit: F) -> Result<T, E>
where F: FnMut(&SymbolicExpression) -> Result<T, E> {
    let mut stack = vec![];
    let mut last = None;
    stack.push(expr);
    while let Some(current) = stack.pop() {
        last = Some(visit(current)?);
        if let Some(list) = current.match_list() {
            for item in list.iter() {
                stack.push(item);
            }
        }
    }

    Ok(last.unwrap())
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SymbolicExpression {
    pub expr: SymbolicExpressionType,
    // this id field is used by compiler passes to store information in
    //  maps.
    // first pass       -> fill out unique ids
    // ...typing passes -> store information in hashmap according to id.
    // 
    // this is a fairly standard technique in compiler passes
    pub id: u64,

    #[cfg(feature = "developer-mode")]
    pub span: Span,
}

impl SymbolicExpression {
    #[cfg(feature = "developer-mode")]
    fn cons() -> SymbolicExpression {
        SymbolicExpression {
            id: 0,
            span: Span::zero(),
            expr: SymbolicExpressionType::AtomValue(Value::Bool(false))
        }
    }
    #[cfg(not(feature = "developer-mode"))]
    fn cons() -> SymbolicExpression {
        SymbolicExpression {
            id: 0,
            expr: SymbolicExpressionType::AtomValue(Value::Bool(false))
        }
    }

    #[cfg(feature = "developer-mode")]
    pub fn set_span(&mut self, start_line: u32, start_column: u32, end_line: u32, end_column: u32) {
        self.span = Span {
            start_line,
            start_column,
            end_line,
            end_column
        }
    }

    #[cfg(not(feature = "developer-mode"))]
    pub fn set_span(&mut self, _start_line: u32, _start_column: u32, _end_line: u32, _end_column: u32) {
    }
    
    pub fn atom_value(val: Value) -> SymbolicExpression {
        SymbolicExpression {
            expr: SymbolicExpressionType::AtomValue(val),
            .. SymbolicExpression::cons()
        }
    }

    pub fn atom(val: ClarityName) -> SymbolicExpression {
        SymbolicExpression {
            expr: SymbolicExpressionType::Atom(val),
            .. SymbolicExpression::cons()
        }
    }

    pub fn literal_value(val: Value) -> SymbolicExpression {
        SymbolicExpression {
            expr: SymbolicExpressionType::LiteralValue(val),
            .. SymbolicExpression::cons()
        }
    }

    pub fn list(val: Box<[SymbolicExpression]>) -> SymbolicExpression {
        SymbolicExpression {
            expr: SymbolicExpressionType::List(val),
            .. SymbolicExpression::cons()
        }
    }

    pub fn trait_reference(val: ClarityName, trait_definition: TraitDefinition) -> SymbolicExpression {
        SymbolicExpression {
            expr: SymbolicExpressionType::TraitReference(val, trait_definition),
            .. SymbolicExpression::cons()
        }
    }

    pub fn field(val: TraitIdentifier) -> SymbolicExpression {
        SymbolicExpression {
            expr: SymbolicExpressionType::Field(val),
            .. SymbolicExpression::cons()
        }
    }

    // These match functions are used to simplify calling code
    //   areas a lot. There is a frequent code pattern where
    //   a block _expects_ specific symbolic expressions, leading
    //   to a lot of very verbose `if let x = {` expressions. 

    pub fn match_list(&self) -> Option<&[SymbolicExpression]> {
        if let SymbolicExpressionType::List(ref list) = self.expr {
            Some(list)
        } else {
            None
        }
    }

    pub fn match_atom(&self) -> Option<&ClarityName> {
        if let SymbolicExpressionType::Atom(ref value) = self.expr {
            Some(value)
        } else {
            None
        }
    }

    pub fn match_atom_value(&self) -> Option<&Value> {
        if let SymbolicExpressionType::AtomValue(ref value) = self.expr {
            Some(value)
        } else {
            None
        }
    }

    pub fn match_literal_value(&self) -> Option<&Value> {
        if let SymbolicExpressionType::LiteralValue(ref value) = self.expr {
            Some(value)
        } else {
            None
        }
    }

    pub fn match_trait_reference(&self) -> Option<&ClarityName> {
        if let SymbolicExpressionType::TraitReference(ref value, _) = self.expr {
            Some(value)
        } else {
            None
        }
    }

    pub fn match_field(&self) -> Option<&TraitIdentifier> {
        if let SymbolicExpressionType::Field(ref value) = self.expr {
            Some(value)
        } else {
            None
        }
    }
}

impl fmt::Display for SymbolicExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.expr {
            SymbolicExpressionType::List(ref list) => {
                write!(f, "(")?;
                for item in list.iter() {
                    write!(f, " {}", item)?;
                }
                write!(f, " )")?;
            },
            SymbolicExpressionType::Atom(ref value) => {
                write!(f, "{}", &**value)?;
            },
            SymbolicExpressionType::AtomValue(ref value) | SymbolicExpressionType::LiteralValue(ref value) => {
                write!(f, "{}", value)?;
            },
            SymbolicExpressionType::TraitReference(ref value, _) => { write!(f, "<{}>", &**value)?; },
            SymbolicExpressionType::Field(ref value) => { write!(f, "<{}>", value)?; },
        };
        
        Ok(())
    }
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32
}

impl Span {
    pub fn zero() -> Span {
        Span {
            start_line: 0,
            start_column: 0,
            end_line: 0,
            end_column: 0
        }
    }
}