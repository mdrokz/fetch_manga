use std::fmt::Display;

pub trait Json {
    fn serialize_array<T: Clone + Display + PartialEq>(
        &self,
        fmt: &mut std::fmt::Formatter<'_>,
        iterator: &Vec<T>,
    ) {
        let count = iterator.clone().len();
        write!(fmt, "[").expect("failed to write opening bracket");
        for (i, v) in iterator.iter().enumerate() {
            let x = format!("{}", v);

            if x.starts_with("{") {
                write!(fmt, "{}", x).expect("failed to write value");
            } else {
                write!(fmt, r#""{}""#, x).expect("failed to write value");
            }
            if i != count - 1 {
                write!(fmt, ",").expect("failed to write seperator");
            }
        }
        write!(fmt, "]").expect("failed to write closing bracket");
    }
}

#[derive(Debug, Clone)]
pub enum Value<'a, T: Display + ?Json> {
    Int(i32),
    Str(&'a str),
    String(String),
    Array(Vec<T>),
    Object(T),
}

impl<'a, T: Display + ?Json> Value<'a, T> {
    #[inline]
    fn extract_i32(&self) -> &i32 {
        if let Value::Int(x) = self {
            x
        } else {
            &0
        }
    }

    fn extract_string(&self) -> &'a str {
        if let Value::Str(x) = self {
            x
        } else {
            ""
        }
    }

    #[inline]
    fn extract_array(&self) -> &[T] {
        if let Value::Array(x) = self {
            &x[..]
        } else {
            &[]
        }
    }

    #[inline]
    fn extract(&self) -> Option<&T> {
        if let Value::Object(x) = self {
            Some(x)
        } else {
            None
        }
    }
}

impl<T: Display> Display for Value<'_, T> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Array(x) => {
                let count = x.clone().len();
                write!(fmt, "[").expect("failed to write opening bracket");
                for (i, v) in x.iter().enumerate() {
                    write!(fmt, r#"{}"#, v).expect("failed to write value");
                    if i != count - 1 {
                        write!(fmt, ",").expect("failed to write seperator");
                    }
                }
                write!(fmt, "]").expect("failed to write closing bracket");

                return Ok(());
            }
            _ => (),
        }
        write!(fmt, "formatted {}", "arguments")
    }
}

#[macro_export]
macro_rules! json {
    ($o: ident,$($v: ident => $s: path),*) => {

        #[derive(Debug,Clone)]
        struct $o<'a> {
            $(
                $v: Value<'a,$s>,
            )*
        }

        impl Json for $o<'_> {}

        impl<'a> Display for $o<'a> {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(fmt,"{{").expect("failed to write opening bracket");
                $(

                    match &self.$v {
                        Value::Array(v) => {
                            write!(fmt,"{:?}:",stringify!($v)).expect("failed to write values");
                            self.serialize_array(fmt,v);
                        },
                        Value::Object(v) => write!(fmt, "{:?}: {},",stringify!($v), v).expect("failed to write values"),
                        Value::Int(v) => write!(fmt, "{:?}: {},",stringify!($v), v).expect("failed to write values"),
                        Value::String(v) => write!(fmt, r#"{:?}: "{}","#,stringify!($v), v).expect("failed to write values"),
                        _ => {}
                    };

                )*
                write!(fmt,"}}").expect("failed to write closing bracket");

                Ok(())
            }
        }

        impl<'a> $o<'_> {
            fn serialize(&self) -> String {
                format!("{}",self).replace(",}", "}")
            }
    }
}

}
