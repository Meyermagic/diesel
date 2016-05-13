#[macro_export]
macro_rules! AsChangeset {
    // Provide a default value for treat_none_as_null if not provided
    (
        ($table_name:ident)
        $($body:tt)*
    ) => {
        AsChangeset! {
            ($table_name, treat_none_as_null="false")
            $($body)*
        }
    };

    // Strip meta items, pub (if present) and struct from definition
    (
        $args:tt
        $(#[$ignore:meta])*
        $(pub)* struct $($body:tt)*
    ) => {
        AsChangeset! {
            $args
            $($body)*
        }
    };

    // Handle struct with lifetimes
    (
        ($table_name:ident, treat_none_as_null=$treat_none_as_null:expr)
        $struct_name:ident <$($lifetime:tt),*>
        $body:tt $(;)*
    ) => {
        __diesel_parse_struct_body! {
            (
                struct_name = $struct_name,
                table_name = $table_name,
                treat_none_as_null = $treat_none_as_null,
                struct_ty = $struct_name<$($lifetime),*>,
                lifetimes = ($($lifetime),*),
            ),
            callback = AsChangeset,
            body = $body,
        }
    };

    // Handle struct with no lifetimes. We pass a dummy lifetime to reduce
    // the amount of branching later.
    (
        ($table_name:ident, treat_none_as_null=$treat_none_as_null:expr)
        $struct_name:ident
        $body:tt $(;)*
    ) => {
        __diesel_parse_struct_body! {
            (
                struct_name = $struct_name,
                table_name = $table_name,
                treat_none_as_null = $treat_none_as_null,
                struct_ty = $struct_name,
                lifetimes = ('a),
            ),
            callback = AsChangeset,
            body = $body,
        }
    };

    // Receive parsed fields of struct from `__diesel_parse_struct_body`
    (
        (
            struct_name = $struct_name:ident,
            table_name = $table_name:ident,
            treat_none_as_null = $treat_none_as_null:expr,
            $($headers:tt)*
        ),
        fields = $fields:tt,
    ) => {
        AsChangeset_construct_changeset_ty! {
            (
                fields = $fields,
                struct_name = $struct_name,
                table_name = $table_name,
                treat_none_as_null = $treat_none_as_null,
                $($headers)*
            ),
            table_name = $table_name,
            treat_none_as_null = $treat_none_as_null,
            fields = $fields,
            changeset_ty = (),
        }
    };

    // Receive changeset ty when tuple struct from `AsChangeset_construct_changeset_ty`
    (
        (
            fields = [$({
                column_name: $column_name:ident,
                field_ty: $field_ty:ty,
                field_kind: $field_kind:ident,
            })+],
            struct_name = $struct_name:ident,
            $($headers:tt)*
        ),
        changeset_ty = $changeset_ty:ty,
    ) => {
        AsChangeset! {
            $($headers)*
            self_to_columns = $struct_name($(ref $column_name),+),
            columns = ($($column_name, $field_kind),+),
            field_names = [],
            changeset_ty = $changeset_ty,
        }
    };

    // Receive changeset ty when named struct from `AsChangeset_construct_changeset_ty`
    (
        (
            fields = [$({
                field_name: $field_name:ident,
                column_name: $column_name:ident,
                field_ty: $field_ty:ty,
                field_kind: $field_kind:ident,
            })+],
            struct_name = $struct_name:ident,
            $($headers:tt)*
        ),
        changeset_ty = $changeset_ty:ty,
    ) => {
        AsChangeset! {
            $($headers)*
            self_to_columns = $struct_name { $($field_name: ref $column_name),+ },
            columns = ($($column_name, $field_ty, $field_kind),+),
            field_names = [$($field_name)+],
            changeset_ty = $changeset_ty,
        }
    };

    // Construct final impl
    (
        table_name = $table_name:ident,
        treat_none_as_null = $treat_none_as_null:expr,
        struct_ty = $struct_ty:ty,
        lifetimes = ($($lifetime:tt),*),
        self_to_columns = $self_to_columns:pat,
        columns = ($($column_name:ident, $field_kind:ident),+),
        field_names = $field_names:tt,
        changeset_ty = $changeset_ty:ty,
    ) => {
        __diesel_parse_as_item! {
            impl<$($lifetime: 'update,)* 'update> $crate::query_builder::AsChangeset
                for &'update $struct_ty
            {
                type Target = $table_name::table;
                type Changeset = $changeset_ty;

                #[allow(non_shorthand_field_patterns)]
                fn as_changeset(self) -> Self::Changeset {
                    let $self_to_columns = *self;
                    ($(
                        AsChangeset_column_expr!(
                            $table_name::$column_name,
                            $column_name,
                            none_as_null = $treat_none_as_null,
                            field_kind = $field_kind,
                        )
                    ,)+)
                }
            }
        }
        __diesel_call_if_includes_id! {
            search = $field_names,
            callback = AsChangeset_sqlite_save_changes_impl,
            args = (
                table_name = $table_name,
                struct_ty = $struct_ty,
                lifetimes = ($($lifetime),*),
            ),
        }
        // AsChangeset_postgres_save_changes_impl! {
        //     table_name = $table_name,
        //     struct_ty = $struct_ty,
        //     lifetimes = ($($lifetime),*),
        // }
    };
}

#[doc(hidden)]
#[macro_export]
AsChangeset_construct_changeset_ty! {
    // Handle option field when treat none as null is false
    (
        $headers:tt,
        table_name = $table_name:ident,
        treat_none_as_null = "false",
        fields = [{
            $(field_name: $field_name:ident,)*
            column_name: $column_name:ident,
            field_ty: Option<$field_ty:ty>,
            field_kind: option,
        } $($tail:tt)*],
        changeset_ty = ($($changeset_ty:ty,)*),
    ) => {
        AsChangeset_construct_changeset_ty! {
            $headers,
            table_name = $table_name,
            treat_none_as_null = "false",
            fields = [$($tail)*],
            changeset_ty = ($($changeset_ty,)*
                Option<$crate::expression::predicates::Eq<
                    $table_name::$column_name,
                    $crate::expression::bound::Bound<
                        <$table_name::$column_name as $crate::expression::Expression>::SqlType,
                        &'update $field_ty,
                    >,
                >>,
            ),
        }
    };

    // Handle normal field or option when treat none as null is true
    (
        $headers:tt,
        table_name = $table_name:ident,
        treat_none_as_null = $treat_none_as_null:expr,
        fields = [{
            $(field_name: $field_name:ident,)*
            column_name: $column_name:ident,
            field_ty: $field_ty:ty,
            $($ignore:tt)*
        } $($tail:tt)*],
        changeset_ty = ($($changeset_ty:ty,)*),
    ) => {
        AsChangeset_construct_changeset_ty! {
            $headers,
            table_name = $table_name,
            treat_none_as_null = $treat_none_as_null,
            fields = [$($tail)*],
            changeset_ty = ($($changeset_ty,)*
                $crate::expression::predicates::Eq<
                    $table_name::$column_name,
                    $crate::expression::bound::Bound<
                        <$table_name::$column_name as $crate::expression::Expression>::SqlType,
                        &'update $field_ty,
                    >,
                >,
            ),
        }
    };

    // Finished parsing
    (
        $headers:tt,
        table_name = $table_name:ident,
        treat_none_as_null = $treat_none_as_null:expr,
        fields = [],
        changeset_ty = $changeset_ty:ty,
    ) => {
        AsChangeset!($headers, changeset_ty = $changeset_ty,);
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! AsChangeset_column_expr {
    // When none_as_null is false, we don't update fields which aren't present
    (
        $column:expr,
        $field_access:expr,
        none_as_null = "false",
        field_kind = option,
    ) => {
        $field_access.as_ref().map(|f| $column.eq(f))
    };

    // If none_as_null is true, or the field kind isn't option, assign blindly
    (
        $column:expr,
        $field_access:expr,
        $($args:tt)*
    ) => {
        $column.eq($field_access)
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "sqlite"))]
macro_rules! AsChangeset_sqlite_save_changes_impl {
    ($($args:tt)*) => {}
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "sqlite")]
macro_rules! AsChangeset_sqlite_save_changes_impl {
    (
        table_name = $table_name:ident,
        struct_ty = $struct_ty:ty,
        lifetimes = ($($lifetime:tt),+),
    ) => { __diesel_parse_as_item! {
        impl<$($lifetime),+> SaveChangesDsl<
            $crate::sqlite::SqliteConnection,
            $table_name::SqlType,
        > for $struct_ty {
            fn save_changes<T>(
                &self,
                conn: &$crate::sqlite::SqliteConnection
            ) -> $crate::QueryResult<T> where
                T: Queryable<$table_name::SqlType, $crate::sqlite::Sqlite>,
            {
                let target = $table_name::table.primary_key().eq(&self.id);
                try!($crate::update($table_name::table.filter(target))
                    .set(self)
                    .execute(conn));
                $table_name::table.find(&self.id).first(conn)
            }
        }
    }};
}
        // AsChangeset_postgres_save_changes_impl! {
        //     table_name = $table_name,
        //     struct_ty = $struct_ty,
        //     lifetimes = ($($lifetime),*),
        // }