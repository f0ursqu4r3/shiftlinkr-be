macro_rules! string_enum {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => $str_val:literal
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant
            ),*
        }

        // SQLx Type implementation
        impl sqlx::Type<sqlx::Postgres> for $name {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
            }
        }

        // SQLx Encode implementation
        impl<'q> sqlx::Encode<'q, sqlx::Postgres> for $name {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer,
            ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                let s = match self {
                    $(Self::$variant => $str_val),*
                };
                <&str as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&s, buf)
            }
        }

        // SQLx Decode implementation
        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for $name {
            fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
                let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
                match s.as_str() {
                    $($str_val => Ok(Self::$variant)),*,
                    _ => Err(format!("Invalid {}: {}", stringify!($name), s).into()),
                }
            }
        }

        // Display implementation
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$variant => write!(f, $str_val)),*
                }
            }
        }

        // FromStr implementation
        impl std::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    $($str_val => Ok(Self::$variant)),*,
                    _ => Err(format!("Invalid {}: {}", stringify!($name), s)),
                }
            }
        }
    };
}

pub(crate) use string_enum;
