use async_graphql::ID;

pub fn try_parse_graphql_ids_to_i64_vec_option(ids: &[ID]) -> Option<Vec<i64>> {
    ids.iter()
        .map(|id| id.0.parse::<i64>().ok())
        .collect::<Option<Vec<i64>>>()
}
