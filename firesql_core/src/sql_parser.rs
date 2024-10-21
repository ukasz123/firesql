use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "sql_parser/sql_grammar.pest"]
pub struct FireSQLParser;

#[cfg(test)]
mod test {
    use pest::Parser;

    use super::*;

    #[test]
    fn basic_query() {
        let pairs = FireSQLParser::parse(
            Rule::select_stmt,
            r"SELECT *, company
            frOM users/USER_ID/achievements",
        );
        println!("pairs: {:#?}", pairs);
        assert!(pairs.is_ok());
    }

    #[test]
    fn single_where_query() {
        let pairs = FireSQLParser::parse(
            Rule::select_stmt,
            r#"SELECT :id
                frOM users/USER_ID/achievements
                WHERE company = "abc""#,
        );
        println!("pairs: {:#?}", pairs);
        assert!(pairs.is_ok());
    }

    #[test]
    fn multi_where_query() {
        let pairs = FireSQLParser::parse(
            Rule::select_stmt,
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
        println!("pairs: {:#?}", pairs);
        assert!(pairs.is_ok());
    }
}
