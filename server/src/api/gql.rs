use juniper::GraphQLObject;

#[derive(GraphQLObject)]
pub struct Auth {
    password: String,
}
