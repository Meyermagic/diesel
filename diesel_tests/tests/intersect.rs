use super::schema::*;
use diesel::*;
use schema_dsl::*;

#[test]
fn union_set_returns_set() {
    use schema::users::dsl::*;

    let connection = connection();
    connection.execute("INSERT INTO users (name) VALUES ('A'), ('B'), ('C'), ('D')")
        .unwrap();

    let expected_data = vec![
        "Tess".to_string(),
        "Sean".to_string(),
    ];
    let query = users.select(name);
    let union = query.union(query);
    let actual_data: Vec<String> = union
        .load(&connection)
        .unwrap();
    assert_eq!(expected_data, actual_data);
}


