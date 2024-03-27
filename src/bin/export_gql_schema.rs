use async_graphql::{EmptySubscription, Schema};
use palabras::gql::studies::{MutationRoot, QueryRoot};

// Assuming you have defined your QueryRoot and other types
async fn export_schema() -> async_graphql::Result<()> {
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish();

    // Generate the schema in SDL format
    let sdl = schema.sdl();

    // Write the SDL to a file
    std::fs::write("schema.graphql", sdl)?;

    Ok(())
}

fn main() {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(export_schema())
        .unwrap();
}
