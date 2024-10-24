mod sql_grammar;


use pest::Parser;
use sql_grammar::*;

struct FireSQLParser;

impl FireSQLParser {
    pub fn parse(stmt: &str) -> Result<FireSQLSelect, ParseError> {
        let parsed = FireSQLGrammarParser::parse(Rule::select_stmt, stmt)
            .map_err(ParseError::GrammarError)?
            .next()
            .expect("select statement present");
        let select_stmt = match parsed.as_rule() {
            Rule::select_stmt => {
                let mut select_inner = parsed.into_inner();

                let projections = select_inner.next().expect("select projections expected");
                let projections = parse_projections(projections);

                let tables = select_inner.next().expect("select tables expected");
                let collection = parse_collection(tables)?;

                let conditions = select_inner.next();
                let conditions = conditions
                    .map(|conditions| {
                        use itertools::*;
                        let conditions = conditions.into_inner();
                        conditions
                            .map(|condition| match condition.as_rule() {
                                Rule::is_null => {
                                    Ok(Condition::IsNull(condition.into_inner().to_string()))
                                }
                                Rule::is_not_null => Ok(Condition::Not(Box::new(
                                    Condition::IsNull(condition.into_inner().to_string()),
                                ))),
                                Rule::comparison => {
                                    let mut comparison_inner = condition.into_inner();
                                    let property_name =
                                        comparison_inner.next().expect("property expected");
                                    let operator =
                                        comparison_inner.next().expect("operator expected");
                                    let value = comparison_inner.next().expect("value expected");
                                    let inner_value =
                                        value.into_inner().next().expect("inner value expected");
                                    let value = match inner_value.as_rule() {
                                        Rule::number => {
                                            Ok(Value::Number(inner_value.as_str().parse().unwrap()))
                                        }
                                        Rule::string => Ok(Value::String(
                                            inner_value
                                                .into_inner()
                                                .next()
                                                .expect("inner_string expected")
                                                .as_str()
                                                .to_owned(),
                                        )),
                                        Rule::reference => {
                                            Ok(Value::Reference(inner_value.as_str().to_owned()))
                                        }
                                        Rule::bool => {
                                            Ok(Value::Bool(inner_value.as_str().parse().unwrap()))
                                        }

                                        rule => Err(ParseError::UnexpectedItem(format!(
                                            "rule {:?} - {}",
                                            rule,
                                            inner_value.as_str()
                                        ))),
                                    }?;
                                    let operation = match operator.as_str() {
                                        "=" => Ok(CompareOperations::Equal(value)),
                                        "!=" => Ok(CompareOperations::NotEqual(value)),
                                        ">" => Ok(CompareOperations::GreaterThan(value)),
                                        "<" => Ok(CompareOperations::LessThan(value)),
                                        _ => Err(ParseError::UnexpectedItem(
                                            operator.as_str().to_owned(),
                                        )),
                                    }?;
                                    Ok(Condition::Comparison(
                                        property_name.as_str().to_owned(),
                                        operation,
                                    ))
                                }
                                rule => Err(ParseError::UnexpectedItem(format!(
                                    "rule {:?} - {}",
                                    rule,
                                    condition.as_str()
                                ))),
                            })
                            .process_results(|c| c.collect_vec())
                    })
                    .unwrap_or(Ok(vec![]))?;

                Ok(FireSQLSelect {
                    projections,
                    collection,
                    conditions,
                })
            }
            _ => Err(ParseError::UnexpectedItem(parsed.as_str().to_string())),
        }?;

        Ok(select_stmt)
    }
}

fn parse_collection(tables: pest::iterators::Pair<'_, Rule>) -> Result<Collection, ParseError> {
    let raw_path = tables.as_str();
    let path = if tables.into_inner().len() % 2 != 1 {
        Err(ParseError::InvalidCollectionPath(format!(
            "Invalid collection path {}",
            raw_path
        )))
    } else {
        Ok(raw_path.to_owned())
    }?;
    let collection = Collection { path };
    Ok(collection)
}

fn parse_projections(projections: pest::iterators::Pair<'_, Rule>) -> Vec<SelectProjection> {
    let projections = projections
        .into_inner()
        .map(|proj| {
            let proj = proj
                .into_inner()
                .next()
                .expect("select projection expected");
            match proj.as_rule() {
                Rule::id_projection => SelectProjection::ObjectId,
                Rule::object_projection => SelectProjection::Object,
                Rule::ident => SelectProjection::Property(proj.as_str().to_owned()),
                _ => unreachable!(),
            }
        })
        .collect::<Vec<_>>();
    projections
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    GrammarError(pest::error::Error<Rule>),
    UnexpectedItem(String),
    InvalidCollectionPath(String),
}

#[derive(Debug, PartialEq)]
struct FireSQLSelect {
    projections: Vec<SelectProjection>,
    collection: Collection,
    conditions: Vec<Condition>,
}

#[derive(Debug, PartialEq)]
enum SelectProjection {
    ObjectId,
    Object,
    Property(String),
}

#[derive(Debug, PartialEq)]
struct Collection {
    path: String,
}

#[derive(Debug, PartialEq)]
enum Condition {
    Not(Box<Condition>),
    IsNull(String),
    Comparison(String, CompareOperations),
}

#[derive(Debug, PartialEq)]
enum CompareOperations {
    Equal(Value),
    NotEqual(Value),
    GreaterThan(Value),
    LessThan(Value),
}

#[derive(Debug, PartialEq)]
enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Reference(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_simple_statement() {
        let result = FireSQLParser::parse(
            r"SELECT *, company
        frOM users/USER_ID/achievements",
        );

        assert_eq!(
            result,
            Ok(FireSQLSelect {
                projections: vec![
                    SelectProjection::Object,
                    SelectProjection::Property("company".to_owned()),
                ],
                collection: Collection {
                    path: "users/USER_ID/achievements".to_owned(),
                },
                conditions: vec![],
            }),
        )
    }

    #[test]
    fn parse_statement_with_conditions() {
        let result = FireSQLParser::parse(
            r#"SELECT :id
                frOM users/USER_ID/achievements
                WHERE name = "abc"
                and name != 4
                and value > 42
                and done = true
                AND completed is not null
                and completion is null
                "#,
        );
        assert!(result.is_ok());
    }
}
