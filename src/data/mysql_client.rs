pub mod mysql_impl {
    use crate::data::{DatabaseInterface, Short};
    use mysql::*;
    use mysql::prelude::*;
    use std::fmt::Result;
    use crate::api::{ApiOperationStatus, EditRequest};

    #[derive(Debug, Clone)]
    pub struct MysqlDB {
        pub connection: Pool
    }

    impl MysqlDB {
        pub fn new(username: &String, password: &String, database: &String) -> Self {
            let url = format!("mysql://{}:{}@192.168.1.100:3306/{}", username, password, database);
            let opts = mysql::Opts::from_url(url.as_str()).unwrap();
            let mut pool = match Pool::new(opts) {
                Ok(p) => {
                    p
                }
                Err(e) => {
                    panic!("Error: {}", e);
                }
            };
            MysqlDB {
                connection: pool
            }
        }

    }

    impl DatabaseInterface for MysqlDB {

        fn list_of_all(&mut self) -> Option<Vec<Short>> {
            let mut connection = self.connection.get_conn();
            if connection.is_err() {
                return None;
            }
            let result = connection.unwrap().query_map("select * from shorts", |(hash, url, until, view)| {
                Short {
                    hash,
                    url,
                    until,
                    view
                }
            });

            match result {
                Ok(data) => {
                    Some(data)
                }
                Err(_) => { None }
            }
        }

        fn is_hash_exist(&self, hash: &String) -> bool {
            let mut connection = self.connection.get_conn();
            if connection.is_err() {
                return false;
            }

            let result: Option<bool> = connection.unwrap()
                .query_first(format!("select exists(select hash from shorts where hash='{}')", hash))
                .unwrap();

            result.unwrap()
        }

        fn add(&mut self, short: Short) -> ApiOperationStatus {
            let mut connection = self.connection.get_conn();
            if connection.is_err() {
                return ApiOperationStatus::ConnectionError;
            }

            if self.is_hash_exist(&short.hash) {
                return ApiOperationStatus::DuplicatedHashError;
            }

            return match connection.unwrap().exec_drop(
                "insert into shorts (hash, url, until) values (:hash, :url, :until)",
                params! {
                    "hash" => &short.hash,
                    "url" => short.url,
                    "until" => short.until
                }
            ) {
                Ok(_) => { ApiOperationStatus::Inserted }
                Err(_) => { ApiOperationStatus::InsertError }
            }
        }

        fn edit(&mut self, hash: String, url: String, until: f64) -> ApiOperationStatus {
            let mut connection = self.connection.get_conn();
            if connection.is_err() {
                return ApiOperationStatus::ConnectionError;
            }

            if self.is_hash_exist(&hash) {
                match connection.unwrap()
                    .exec_drop("update shorts set url=:url, until=:until where hash=:hash",
                    params! {
                        "url" => url.trim(),
                        "until" => until,
                        "hash" => hash
                    }) {
                    Ok(_) => { return ApiOperationStatus::Edited }
                    Err(e) => {}
                }
            }

            ApiOperationStatus::EditError
        }

        fn delete(&mut self, hash: String) -> ApiOperationStatus {
            let mut connection = self.connection.get_conn();
            if connection.is_err() {
                return ApiOperationStatus::ConnectionError;
            }

            if self.is_hash_exist(&hash) {
                match connection.unwrap().exec_drop("delete from shorts where hash =:hash",
                params! {
                    "hash" => hash
                }) {
                    Ok(_) => { return ApiOperationStatus::Deleted; }
                    Err(_) => {}
                }
            }
            return ApiOperationStatus::DeleteError
        }
    }

}