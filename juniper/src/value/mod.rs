use crate::ast::{InputValue, ToInputValue};
use crate::parser::Spanning;
mod object;
mod scalar;

pub use self::object::Object;

pub use self::scalar::{
    DefaultScalarValue, ParseScalarResult, ParseScalarValue, ScalarRefValue, ScalarValue,
};

/// Serializable value returned from query and field execution.
///
/// Used by the execution engine and resolvers to build up the response
/// structure. Similar to the `Json` type found in the serialize crate.
///
/// It is also similar to the `InputValue` type, but can not contain enum
/// values or variables. Also, lists and objects do not contain any location
/// information since they are generated by resolving fields and values rather
/// than parsing a source query.
#[derive(Debug, PartialEq, Clone)]
#[allow(missing_docs)]
pub enum Value<S = DefaultScalarValue> {
    Null,
    Scalar(S),
    List(Vec<Value<S>>),
    Object(Object<S>),
}

impl<S> Value<S>
where
    S: ScalarValue,
{
    // CONSTRUCTORS

    /// Construct a null value.
    pub fn null() -> Self {
        Value::Null
    }

    /// Construct an integer value.
    #[deprecated(since = "0.11.0", note = "Use `Value::scalar` instead")]
    pub fn int(i: i32) -> Self {
        Self::scalar(i)
    }

    /// Construct a floating point value.
    #[deprecated(since = "0.11.0", note = "Use `Value::scalar` instead")]
    pub fn float(f: f64) -> Self {
        Self::scalar(f)
    }

    /// Construct a string value.
    #[deprecated(since = "0.11.0", note = "Use `Value::scalar` instead")]
    pub fn string(s: &str) -> Self {
        Self::scalar(s.to_owned())
    }

    /// Construct a boolean value.
    #[deprecated(since = "0.11.0", note = "Use `Value::scalar` instead")]
    pub fn boolean(b: bool) -> Self {
        Self::scalar(b)
    }

    /// Construct a list value.
    pub fn list(l: Vec<Self>) -> Self {
        Value::List(l)
    }

    /// Construct an object value.
    pub fn object(o: Object<S>) -> Self {
        Value::Object(o)
    }

    /// Construct a scalar value
    pub fn scalar<T>(s: T) -> Self
    where
        T: Into<S>,
    {
        Value::Scalar(s.into())
    }

    // DISCRIMINATORS

    /// Does this value represent null?
    pub fn is_null(&self) -> bool {
        match *self {
            Value::Null => true,
            _ => false,
        }
    }

    /// View the underlying scalar value if present
    pub fn as_scalar_value<'a, T>(&'a self) -> Option<&'a T>
    where
        &'a S: Into<Option<&'a T>>,
    {
        match *self {
            Value::Scalar(ref s) => s.into(),
            _ => None,
        }
    }

    /// View the underlying float value, if present.
    #[deprecated(since = "0.11.0", note = "Use `Value::as_scalar_value` instead")]
    pub fn as_float_value(&self) -> Option<f64>
    where
        for<'a> &'a S: ScalarRefValue<'a>,
    {
        self.as_scalar_value::<f64>().cloned()
    }

    /// View the underlying object value, if present.
    pub fn as_object_value(&self) -> Option<&Object<S>> {
        match *self {
            Value::Object(ref o) => Some(o),
            _ => None,
        }
    }

    pub fn into_object(self) -> Option<Object<S>> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Mutable view into the underlying object value, if present.
    pub fn as_mut_object_value(&mut self) -> Option<&mut Object<S>> {
        match *self {
            Value::Object(ref mut o) => Some(o),
            _ => None,
        }
    }

    /// View the underlying list value, if present.
    pub fn as_list_value(&self) -> Option<&Vec<Self>> {
        match *self {
            Value::List(ref l) => Some(l),
            _ => None,
        }
    }

    /// View the underlying scalar value, if present
    pub fn as_scalar(&self) -> Option<&S> {
        match *self {
            Value::Scalar(ref s) => Some(s),
            _ => None,
        }
    }

    /// View the underlying string value, if present.
    #[deprecated(since = "0.11.0", note = "Use `Value::as_scalar_value` instead")]
    pub fn as_string_value<'a>(&'a self) -> Option<&'a str>
    where
        Option<&'a String>: From<&'a S>,
    {
        self.as_scalar_value::<String>().map(|s| s as &str)
    }
}

impl<S: ScalarValue> ToInputValue<S> for Value<S> {
    fn to_input_value(&self) -> InputValue<S> {
        match *self {
            Value::Null => InputValue::Null,
            Value::Scalar(ref s) => InputValue::Scalar(s.clone()),
            Value::List(ref l) => InputValue::List(
                l.iter()
                    .map(|x| Spanning::unlocated(x.to_input_value()))
                    .collect(),
            ),
            Value::Object(ref o) => InputValue::Object(
                o.iter()
                    .map(|&(ref k, ref v)| {
                        (
                            Spanning::unlocated(k.clone()),
                            Spanning::unlocated(v.to_input_value()),
                        )
                    })
                    .collect(),
            ),
        }
    }
}

impl<S, T> From<Option<T>> for Value<S>
where
    S: ScalarValue,
    Value<S>: From<T>,
{
    fn from(v: Option<T>) -> Value<S> {
        match v {
            Some(v) => v.into(),
            None => Value::null(),
        }
    }
}

impl<'a, S> From<&'a str> for Value<S>
where
    S: ScalarValue,
{
    fn from(s: &'a str) -> Self {
        Value::scalar(s.to_owned())
    }
}

impl<S> From<String> for Value<S>
where
    S: ScalarValue,
{
    fn from(s: String) -> Self {
        Value::scalar(s)
    }
}

impl<S> From<i32> for Value<S>
where
    S: ScalarValue,
{
    fn from(i: i32) -> Self {
        Value::scalar(i)
    }
}

impl<S> From<f64> for Value<S>
where
    S: ScalarValue,
{
    fn from(f: f64) -> Self {
        Value::scalar(f)
    }
}

impl<S> From<bool> for Value<S>
where
    S: ScalarValue,
{
    fn from(b: bool) -> Self {
        Value::scalar(b)
    }
}

/// Construct JSON-like values by using JSON syntax
///
/// This macro can be used to create `Value` instances using a JSON syntax.
/// Value objects are used mostly when creating custom errors from fields.
///
/// Here are some examples; the resulting JSON will look just like what you
/// passed in.
/// ```rust
/// extern crate juniper;
/// # use juniper::{Value, DefaultScalarValue, graphql_value};
/// # type V = Value<DefaultScalarValue>;
///
/// # fn main() {
/// # let _: V =
/// graphql_value!(1234);
/// # let _: V =
/// graphql_value!("test");
/// # let _: V =
/// graphql_value!([ 1234, "test", true ]);
/// # let _: V =
/// graphql_value!({ "key": "value", "foo": 1234 });
/// # }
/// ```
#[macro_export]
macro_rules! graphql_value {
    ([ $($arg:tt),* $(,)* ]) => {
        $crate::Value::list(vec![
            $( graphql_value!($arg), )*
        ])
    };
    ({ $($key:tt : $val:tt ),* $(,)* }) => {
        $crate::Value::object(vec![
            $( ($key, graphql_value!($val)), )*
        ].into_iter().collect())
    };
    (None) => ($crate::Value::null());
    ($e:expr) => ($crate::Value::from($e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_macro_string() {
        let s: Value<DefaultScalarValue> = graphql_value!("test");
        assert_eq!(s, Value::scalar("test"));
    }

    #[test]
    fn value_macro_int() {
        let s: Value<DefaultScalarValue> = graphql_value!(123);
        assert_eq!(s, Value::scalar(123));
    }

    #[test]
    fn value_macro_float() {
        let s: Value<DefaultScalarValue> = graphql_value!(123.5);
        assert_eq!(s, Value::scalar(123.5));
    }

    #[test]
    fn value_macro_boolean() {
        let s: Value<DefaultScalarValue> = graphql_value!(false);
        assert_eq!(s, Value::scalar(false));
    }

    #[test]
    fn value_macro_option() {
        let s: Value<DefaultScalarValue> = graphql_value!(Some("test"));
        assert_eq!(s, Value::scalar("test"));
        let s: Value<DefaultScalarValue> = graphql_value!(None);
        assert_eq!(s, Value::null());
    }

    #[test]
    fn value_macro_list() {
        let s: Value<DefaultScalarValue> = graphql_value!([123, "Test", false]);
        assert_eq!(
            s,
            Value::list(vec![
                Value::scalar(123),
                Value::scalar("Test"),
                Value::scalar(false),
            ])
        );
        let s: Value<DefaultScalarValue> = graphql_value!([123, [456], 789]);
        assert_eq!(
            s,
            Value::list(vec![
                Value::scalar(123),
                Value::list(vec![Value::scalar(456)]),
                Value::scalar(789),
            ])
        );
    }

    #[test]
    fn value_macro_object() {
        let s: Value<DefaultScalarValue> = graphql_value!({ "key": 123, "next": true });
        assert_eq!(
            s,
            Value::object(
                vec![("key", Value::scalar(123)), ("next", Value::scalar(true))]
                    .into_iter()
                    .collect(),
            )
        );
    }
}
