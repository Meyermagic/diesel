use backend::Backend;
use result::QueryResult;
use super::{Query, CombinableQuery, QueryBuilder, QueryFragment, BuildQueryResult};

#[derive(Debug)]
pub struct UnionAllQuery<L, R> {
    left: L,
    right: R,
}

impl<L, R> UnionAllQuery<L, R> {
    pub fn new(left: L, right: R) -> Self {
        UnionAllQuery {
            left: left,
            right: right,
        }
    }
}

impl<L, R> Query for UnionAllQuery<L, R>
    where L: CombinableQuery,
          R: CombinableQuery<SqlType = L::SqlType>
{
    type SqlType = <L as Query>::SqlType;
}

impl<L, R> CombinableQuery for UnionAllQuery<L, R> where UnionAllQuery<L, R>: Query {}

impl<L, R, DB> QueryFragment<DB> for UnionAllQuery<L, R>
    where DB: Backend,
          L: QueryFragment<DB>,
          R: QueryFragment<DB>
{
    fn to_sql(&self, out: &mut DB::QueryBuilder) -> BuildQueryResult {
        try!(self.left.to_sql(out));
        out.push_sql(" UNION ALL ");
        try!(self.right.to_sql(out));
        Ok(())
    }

    fn collect_binds(&self, out: &mut DB::BindCollector) -> QueryResult<()> {
        try!(self.left.collect_binds(out));
        try!(self.right.collect_binds(out));
        Ok(())
    }

    fn is_safe_to_cache_prepared(&self) -> bool {
        self.left.is_safe_to_cache_prepared() && self.right.is_safe_to_cache_prepared()
    }
}

impl_query_id!(UnionAllQuery<L, R>);
