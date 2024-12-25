use std::error::Error;

use firestore::{async_trait, errors::FirestoreError, FirestoreDb, FirestoreValue};
use itertools::Itertools;

use crate::sql_parser::{FireSQLSelect, Value};

#[async_trait]
pub trait SQLExecutor<DB> {
    type Error;
    async fn execute(self, db: &DB) -> Result<Vec<Row>, Self::Error>;
}

#[async_trait]
impl SQLExecutor<FirestoreDb> for FireSQLSelect {
    type Error = FirestoreError;

    async fn execute(self, db: &FirestoreDb) -> Result<Vec<Row>, Self::Error> {
        let (collection, projections, conditions) =
            (self.collection, self.projections, self.conditions);
        let conditions = Box::new(conditions);
        let query = db.fluent().select().from(collection.path.as_str());
        let query = query.filter(|f| {
            let c = conditions.iter().map(|c| match c {
                crate::sql_parser::Condition::Not(_condition) => {
                    todo!("Condition negation not implemented")
                }
                crate::sql_parser::Condition::IsNull(field) => f.field(field).is_null(),
                crate::sql_parser::Condition::Comparison(field, compare_operations) => {
                    let field = f.field(field);
                    match compare_operations {
                        crate::sql_parser::CompareOperations::Equal(value) => {
                            field.eq(ValueWrapper::from(value))
                        }
                        crate::sql_parser::CompareOperations::NotEqual(value) => {
                            field.not_equal(ValueWrapper::from(value))
                        }
                        crate::sql_parser::CompareOperations::GreaterThan(value) => {
                            field.greater_than(ValueWrapper::from(value))
                        }
                        crate::sql_parser::CompareOperations::LessThan(value) => {
                            field.less_than(ValueWrapper::from(value))
                        }
                    }
                }
            });

            f.for_all(c)
        });

        let results = query.limit(10).query().await?;

        // todo: convert result documents into row
        Ok(results.into_iter().map(|d| Row(d.name)).collect_vec())
    }
}

struct ValueWrapper<'a>(&'a Value);

impl<'a> From<&'a Value> for ValueWrapper<'a> {
    fn from(value: &'a Value) -> Self {
        ValueWrapper(value)
    }
}

impl<'a> Into<FirestoreValue> for ValueWrapper<'a> {
    fn into(self) -> FirestoreValue {
        match &self.0 {
            Value::Number(n) => n.into(),
            Value::String(s) => s.into(),
            Value::Bool(b) => b.into(),
            Value::Reference(path) => path.into(),
        }
    }
}

// todo: make real row
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row(String);
