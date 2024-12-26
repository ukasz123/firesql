use std::collections::HashMap;

use firestore::{async_trait, errors::FirestoreError, FirestoreDb, FirestoreValue};
use futures::TryFutureExt as _;
use itertools::Itertools as _;

use crate::sql_parser::{FireSQLSelect, SelectProjection, Value};

#[async_trait]
pub trait SQLExecutor {
    type Error;
    async fn execute(self, select: FireSQLSelect) -> Result<Vec<Row>, Self::Error>;
}

#[async_trait]
impl<'a> SQLExecutor for &'a FirestoreDb {
    type Error = FirestoreError;

    async fn execute(self, select: FireSQLSelect) -> Result<Vec<Row>, Self::Error> {
        let (collection, projections, conditions) =
            (select.collection, select.projections, select.conditions);
        let conditions = Box::new(conditions);
        let query = self.fluent().select();

        let query = if !projections.contains(&SelectProjection::Object) {
            query.fields(projections.iter().filter_map(|field| match field {
                SelectProjection::Property(name) => Some(name),
                _ => None,
            }))
        } else {
            query
        };
        let query = query.from(collection.path.as_str());

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

        let results = query
            .query()
            .map_ok(|results| {
                results
                    .into_iter()
                    .map(|d| {
                        let columns = projections
                            .iter()
                            .map(|projection| match projection {
                                SelectProjection::ObjectId => (
                                    ":id".to_owned(),
                                    d.name.split("/").last().unwrap().to_owned(),
                                ),
                                SelectProjection::Object => {
                                    let fields = d
                                        .fields
                                        .iter()
                                        .map(|(key, value)| (key, firestore_value_to_string(value)))
                                        .collect::<HashMap<_, _>>();
                                    ("*".to_owned(), serde_json::to_string(&fields).unwrap())
                                }
                                SelectProjection::Property(property) => (
                                    property.clone(),
                                    d.fields
                                        .get(property)
                                        .map(|v| firestore_value_to_string(&v))
                                        .unwrap_or_else(|| "nil".to_owned()),
                                ),
                            })
                            .collect_vec();
                        Row(d.name, columns)
                    })
                    .collect_vec()
            })
            .await?;

        // todo: convert result documents into row
        Ok(results)
    }
}

struct ValueWrapper<'a>(&'a Value);

impl<'a> From<&'a Value> for ValueWrapper<'a> {
    fn from(value: &'a Value) -> Self {
        ValueWrapper(value)
    }
}

impl<'a> From<ValueWrapper<'a>> for FirestoreValue {
    fn from(val: ValueWrapper<'a>) -> Self {
        match &val.0 {
            Value::Number(n) => n.into(),
            Value::String(s) => s.into(),
            Value::Bool(b) => b.into(),
            Value::Reference(path) => path.into(),
        }
    }
}

fn firestore_value_to_string(v: &gcloud_sdk::google::firestore::v1::Value) -> String {
    let vt = v
        .value_type
        .as_ref()
        .expect("Value should contain its type");
    match vt {
        gcloud_sdk::google::firestore::v1::value::ValueType::NullValue(_) => "NULL".to_owned(),
        gcloud_sdk::google::firestore::v1::value::ValueType::BooleanValue(v) => format!("{v}"),
        gcloud_sdk::google::firestore::v1::value::ValueType::IntegerValue(v) => format!("{v}"),
        gcloud_sdk::google::firestore::v1::value::ValueType::DoubleValue(v) => format!("{v}"),
        gcloud_sdk::google::firestore::v1::value::ValueType::TimestampValue(timestamp) => {
            format!("{timestamp}")
        }
        gcloud_sdk::google::firestore::v1::value::ValueType::StringValue(v) => format!("{v}"),
        gcloud_sdk::google::firestore::v1::value::ValueType::BytesValue(vec) => todo!(),
        gcloud_sdk::google::firestore::v1::value::ValueType::ReferenceValue(r) => format!("#:{r}"),
        gcloud_sdk::google::firestore::v1::value::ValueType::GeoPointValue(lat_lng) => {
            format!("{}:{}", lat_lng.latitude, lat_lng.longitude)
        }
        gcloud_sdk::google::firestore::v1::value::ValueType::ArrayValue(array_value) => array_value
            .values
            .iter()
            .map(firestore_value_to_string)
            .join(", "),
        gcloud_sdk::google::firestore::v1::value::ValueType::MapValue(map_value) => map_value
            .fields
            .iter()
            .map(|(key, val)| format!("({} : {})", key, firestore_value_to_string(val)))
            .join(", "),
    }
}
// todo: make real row
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row(String, Vec<(String, String)>);

impl Row {
    pub fn id(&self) -> &str {
        &self.0
    }
    pub fn columns(&self) -> &Vec<(String, String)> {
        &self.1
    }
}
