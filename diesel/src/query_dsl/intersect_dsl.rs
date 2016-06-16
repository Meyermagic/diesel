use query_builder::{CombinableQuery, IntersectQuery};

pub trait IntersectDsl<U: CombinableQuery<SqlType = Self::SqlType>>: CombinableQuery {
    type Output: CombinableQuery<SqlType = Self::SqlType>;

    fn intersect(self, query: U) -> Self::Output;
}

impl<T, U> IntersectDsl<U> for T
    where T: CombinableQuery,
          U: CombinableQuery<SqlType = T::SqlType>
{
    type Output = IntersectQuery<T, U>;

    fn intersect(self, other: U) -> Self::Output {
        IntersectQuery::new(self, other)
    }
}
