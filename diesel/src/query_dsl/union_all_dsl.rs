use query_builder::{CombinableQuery, UnionAllQuery};

pub trait UnionAllDsl<U: CombinableQuery<SqlType = Self::SqlType>>: CombinableQuery {
    type Output: CombinableQuery<SqlType = Self::SqlType>;

    fn union_all(self, query: U) -> Self::Output;
}

impl<T, U> UnionAllDsl<U> for T
    where T: CombinableQuery,
          U: CombinableQuery<SqlType = T::SqlType>
{
    type Output = UnionAllQuery<T, U>;

    fn union_all(self, other: U) -> Self::Output {
        UnionAllQuery::new(self, other)
    }
}
