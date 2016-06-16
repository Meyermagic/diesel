use query_builder::{CombinableQuery, ExceptQuery};

pub trait ExceptDsl<U: CombinableQuery<SqlType = Self::SqlType>>: CombinableQuery {
    type Output: CombinableQuery<SqlType = Self::SqlType>;

    fn except(self, query: U) -> Self::Output;
}

impl<T, U> ExceptDsl<U> for T
    where T: CombinableQuery,
          U: CombinableQuery<SqlType = T::SqlType>
{
    type Output = ExceptQuery<T, U>;

    fn except(self, other: U) -> Self::Output {
        ExceptQuery::new(self, other)
    }
}
