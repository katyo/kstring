#[cfg(feature = "diesel")]
mod test_diesel {
    use diesel::{prelude::*, result::Error, sqlite::SqliteConnection};
    use kstring::*;

    // An example struct to represent a table in the database
    #[derive(Debug, PartialEq, Eq, Queryable, Insertable)]
    #[diesel(table_name = test_table)]
    struct TestEntity {
        id: i32,
        name: KString,
    }

    // Diesel schema for testing purposes
    table! {
        test_table (id) {
            id -> Integer,
            name -> Text,
        }
    }

    fn establish_connection() -> SqliteConnection {
        let database_url = ":memory:";
        SqliteConnection::establish(database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
    }

    #[test]
    fn test_insert_and_retrieve() -> Result<(), Error> {
        use self::test_table::dsl::*;

        let mut connection = establish_connection();

        // Create a table in memory for the purpose of testing
        diesel::sql_query(
            "CREATE TABLE IF NOT EXISTS test_table (id INTEGER PRIMARY KEY, name TEXT)",
        )
        .execute(&mut connection)?;

        let new_entity = TestEntity {
            id: 1,
            name: KString::from_static("Test Name"),
        };

        diesel::insert_into(test_table)
            .values(&new_entity)
            .execute(&mut connection)?;

        let results: Vec<TestEntity> = test_table
            .filter(id.eq(1))
            .load::<TestEntity>(&mut connection)?;

        assert_eq!(vec![new_entity], results);

        Ok(())
    }
}
