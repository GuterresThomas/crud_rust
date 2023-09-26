use warp::Filter;
use tokio_postgres::{NoTls, Error, Client};
use std::sync::Arc;
use warp::reject::custom;

// Define um tipo de erro personalizado que implementa Reject
#[derive(Debug)]
struct CustomError(String);

impl warp::reject::Reject for CustomError {}

// Define uma estrutura de dados para o item
#[derive(serde::Deserialize, serde::Serialize)]
struct Item {
    id: i32,
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configurar uma conexão com o banco de dados PostgreSQL
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=1234 dbname=postgres", NoTls)
            .await?;
    tokio::spawn(connection);

    // Encapsular a conexão do banco de dados em um Arc
    let client = Arc::new(client);

    // Criar um filtro personalizado para a conexão do banco de dados
    let db = warp::any().map(move || client.clone());

    // Definir as rotas
    let create_item = warp::post()
        .and(warp::path("items"))
        .and(warp::body::json())
        .and(db.clone()) // Passar a conexão do banco de dados para o manipulador
        .and_then(|item: Item, client: Arc<Client>| async move {
            // Implemente a lógica para criar um item no banco de dados
            // e retorne uma resposta adequada
            let insert_query = format!("INSERT INTO items (name) VALUES ('{}')", item.name);

            match client.execute(&insert_query, &[]).await {
                Ok(rows) if rows == 1 => Ok(warp::reply::json(&item)),
                _ => {
                    let error_message = "Failed to insert item".to_string();
                    Err(custom(CustomError(error_message)))
                },
            }
        });

    let get_items = warp::get()
        .and(warp::path("items"))
        .and(db.clone()) // Passar a conexão do banco de dados para o manipulador
        .map(|client: Arc<Client>| {
            // Implemente a lógica para buscar itens no banco de dados
            // e retorne uma resposta adequada
            // Aqui você pode executar uma consulta SELECT no banco de dados
            // e retornar os resultados como uma resposta JSON
            let items = vec![Item {
                id: 1,
                name: "Item 1".to_string(),
            }];
            warp::reply::json(&items)
        });

    // Combinar todas as rotas
    let routes = create_item.or(get_items);

    // Iniciar o servidor Warp
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;

    Ok(())
}
