use warp::{Filter, filters::query};
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
        .and_then(|client: Arc<Client>| async move {
            let query = "SELECT id, name FROM items"; //consulta sql

            match client.query(query, &[]).await {
                Ok(rows) => {
                    //mapear os items
                    let items: Vec<Item> = rows
                    .into_iter()
                    .map(|row| Item {
                        id: row.get("id"),
                        name: row.get("name"),
                    })
                    .collect();
                
                Ok(warp::reply::json(&items))
                }
                Err(err) => {
                    let error_message = format!("Failed to fetch items: {}", err);
                    Err(custom(CustomError(error_message)))
                }
            }
        });

        let delete_item = warp::delete()
        .and(warp::path!("items" / i32))
        .and(db.clone())
        .and_then(|item_id: i32, client: Arc<Client>| async move {
            let delete_query = format!("DELETE FROM items WHERE id = {}", item_id);
            match client.execute(&delete_query, &[]).await {
                Ok(rows) if rows == 1 => {
                    // A exclusão foi bem-sucedida, retornar uma resposta de sucesso
                    Ok(warp::reply::html("Item excluído com sucesso"))
                }
                _ => {
                    // Não foi possível excluir o item (talvez o ID não exista)
                    let error_message = format!("Failed to delete item with ID: {}", item_id);
                    Err(custom(CustomError(error_message)))
                }
            }
        });

    // Combinar todas as rotas
    let routes = create_item.or(get_items).or(delete_item);

    // Iniciar o servidor Warp
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;

    Ok(())
}
