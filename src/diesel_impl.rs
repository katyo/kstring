#[cfg(not(feature = "unsafe"))]
use super::StdString;
use super::{HeapStr, SecureString, String, StringCow, StringRef};
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow, Result as DeResult},
    expression::AsExpression,
    serialize::{Output, Result as SerResult, ToSql},
    sql_types::Text,
};

#[derive(AsExpression, FromSqlRow)]
#[diesel(foreign_derive)]
#[diesel(sql_type = Text)]
#[allow(dead_code)]
struct StringProxy<B>(String<B>);

#[cfg(feature = "unsafe")]
impl<B, ST, DB> FromSql<ST, DB> for String<B>
where
    B: HeapStr,
    DB: Backend,
    *const str: FromSql<ST, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> DeResult<Self> {
        let str_ptr = <*const str as FromSql<ST, DB>>::from_sql(bytes)?;
        if !str_ptr.is_null() {
            // SAFETY: We just checked that `str_ptr` is not null, and `from_sql()` should return
            // a valid pointer to an `str`.
            let string = unsafe { &*str_ptr };
            Ok(Self::from_ref(string))
        } else {
            Ok(Default::default())
        }
    }
}

#[cfg(not(feature = "unsafe"))]
impl<B, ST, DB> FromSql<ST, DB> for String<B>
where
    B: HeapStr,
    DB: Backend,
    StdString: FromSql<ST, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> DeResult<Self> {
        <super::StdString as FromSql<ST, DB>>::from_sql(bytes).map(Self::from_string)
    }
}

impl<B, DB> ToSql<Text, DB> for String<B>
where
    B: HeapStr,
    DB: Backend,
    str: ToSql<Text, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> SerResult {
        self.as_str().to_sql(out)
    }
}

#[derive(AsExpression, FromSqlRow)]
#[diesel(foreign_derive)]
#[diesel(sql_type = Text)]
#[allow(dead_code)]
struct SecureStringProxy<B: HeapStr>(SecureString<B>);

impl<B, ST, DB> FromSql<ST, DB> for SecureString<B>
where
    B: HeapStr,
    DB: Backend,
    String<B>: FromSql<ST, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> DeResult<Self> {
        <String<B> as FromSql<ST, DB>>::from_sql(bytes).map(From::from)
    }
}

impl<B, DB> ToSql<Text, DB> for SecureString<B>
where
    B: HeapStr,
    DB: Backend,
    str: ToSql<Text, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> SerResult {
        self.as_str().to_sql(out)
    }
}

#[derive(AsExpression, FromSqlRow)]
#[diesel(foreign_derive)]
#[diesel(sql_type = Text)]
#[allow(dead_code)]
struct StringCowProxy<'s, B>(StringCow<'s, B>);

#[cfg(feature = "unsafe")]
impl<B, ST, DB> FromSql<ST, DB> for StringCow<'_, B>
where
    B: HeapStr,
    DB: Backend,
    *const str: FromSql<ST, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> DeResult<Self> {
        String::from_sql(bytes).map(From::from)
    }
}

#[cfg(not(feature = "unsafe"))]
impl<B, ST, DB> FromSql<ST, DB> for StringCow<'_, B>
where
    B: HeapStr,
    DB: Backend,
    StdString: FromSql<ST, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> DeResult<Self> {
        String::from_sql(bytes).map(From::from)
    }
}

impl<B, DB> ToSql<Text, DB> for StringCow<'_, B>
where
    B: HeapStr,
    DB: Backend,
    str: ToSql<Text, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> SerResult {
        self.as_str().to_sql(out)
    }
}

#[derive(AsExpression)]
#[diesel(foreign_derive)]
#[diesel(sql_type = Text)]
#[allow(dead_code)]
struct StringRefProxy<'s>(StringRef<'s>);

impl<DB> ToSql<Text, DB> for StringRef<'_>
where
    DB: Backend,
    str: ToSql<Text, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> SerResult {
        self.as_str().to_sql(out)
    }
}
