// Copyright 2020 Alex Dukhno
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use kernel::SystemResult;
use protocol::results::{QueryError, QueryEvent, QueryResult};
use sqlparser::ast::{Assignment, ObjectName};
use std::sync::{Arc, Mutex};
use storage::{backend::BackendStorage, frontend::FrontendStorage, OperationOnTableError};

pub(crate) struct UpdateCommand<'q, P: BackendStorage> {
    raw_sql_query: &'q str,
    name: ObjectName,
    assignments: Vec<Assignment>,
    storage: Arc<Mutex<FrontendStorage<P>>>,
}

impl<P: BackendStorage> UpdateCommand<'_, P> {
    pub(crate) fn new(
        raw_sql_query: &'_ str,
        name: ObjectName,
        assignments: Vec<Assignment>,
        storage: Arc<Mutex<FrontendStorage<P>>>,
    ) -> UpdateCommand<P> {
        UpdateCommand {
            raw_sql_query,
            name,
            assignments,
            storage,
        }
    }

    pub(crate) fn execute(&mut self) -> SystemResult<QueryResult> {
        let schema_name = self.name.0[0].to_string();
        let table_name = self.name.0[1].to_string();

        let to_update: Vec<(String, String)> = self
            .assignments
            .iter()
            .map(|item| {
                let sqlparser::ast::Assignment { id, value } = &item;
                let sqlparser::ast::Ident { value: column, .. } = id;

                let value = match value {
                    sqlparser::ast::Expr::Value(sqlparser::ast::Value::Number(val)) => val.to_owned(),
                    sqlparser::ast::Expr::Value(sqlparser::ast::Value::SingleQuotedString(v)) => v.to_string(),
                    sqlparser::ast::Expr::UnaryOp { op, expr } => match (op, &**expr) {
                        (
                            sqlparser::ast::UnaryOperator::Minus,
                            sqlparser::ast::Expr::Value(sqlparser::ast::Value::Number(v)),
                        ) => "-".to_owned() + v.as_str(),
                        (op, expr) => unimplemented!("{:?} {:?} is not currently supported", op, expr),
                    },
                    expr => unimplemented!("{:?} is not currently supported", expr),
                };

                (column.to_owned(), value)
            })
            .collect();

        match (self.storage.lock().unwrap()).update_all(&schema_name, &table_name, to_update)? {
            Ok(records_number) => Ok(Ok(QueryEvent::RecordsUpdated(records_number))),
            Err(OperationOnTableError::SchemaDoesNotExist) => Ok(Err(QueryError::schema_does_not_exist(schema_name))),
            Err(OperationOnTableError::TableDoesNotExist) => Ok(Err(QueryError::table_does_not_exist(
                schema_name + "." + table_name.as_str(),
            ))),
            Err(OperationOnTableError::ColumnDoesNotExist(non_existing_columns)) => {
                Ok(Err(QueryError::column_does_not_exist(non_existing_columns)))
            }
            _ => Ok(Err(QueryError::not_supported_operation(self.raw_sql_query.to_owned()))),
        }
    }
}
