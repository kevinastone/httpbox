extern crate urlencoded;
use self::urlencoded::QueryMap;
use std::str::FromStr;


pub fn parse_query_value<T: FromStr>(hashmap: Option<&QueryMap>, name: &str) -> Option<T> {
    hashmap.and_then(|hashmap| hashmap.get(name))
        .and_then(|vals| vals.first())
        .and_then(|val| val.parse::<T>().ok())
}


#[cfg(test)]
mod tests {

    mod parse_query_value {

        use super::super::*;
        use super::super::urlencoded::QueryMap;

        pub const TEST_QUERY_PARAM: &'static str = "seed";

        #[test]
        fn parse_parse_query_value_missing() {
            let query = QueryMap::new();

            assert_eq!(parse_query_value::<u32>(Some(&query), TEST_QUERY_PARAM),
                       None)
        }

        #[test]
        fn parse_parse_query_value_empty() {
            let mut query = QueryMap::new();
            query.insert(String::from(TEST_QUERY_PARAM), vec![]);

            assert_eq!(parse_query_value::<u32>(Some(&query), TEST_QUERY_PARAM),
                       None)
        }

        #[test]
        fn parse_parse_query_value_invalid() {
            let mut query = QueryMap::new();
            query.insert(String::from(TEST_QUERY_PARAM), vec![String::from("abcd")]);

            assert_eq!(parse_query_value::<u32>(Some(&query), TEST_QUERY_PARAM),
                       None)
        }

        #[test]
        fn parse_parse_query_value_valid() {
            let mut query = QueryMap::new();
            query.insert(String::from(TEST_QUERY_PARAM), vec![String::from("1234")]);

            assert_eq!(parse_query_value::<u32>(Some(&query), TEST_QUERY_PARAM),
                       Some(1234))
        }

    }
}
