use actix_web::web;
use deadpool_postgres::{GenericClient};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

pub type AsyncVoidResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Deserialize)]
pub struct CriarPessoaDTO {
    pub apelido: String,
    pub nome: String,
    pub nascimento: String,
    pub stack: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize)]
pub struct PessoaDTO {
    pub id: String,
    pub apelido: String,
    pub nome: String,
    pub nascimento: String,
    pub stack: Option<Vec<String>>,
}

impl PessoaDTO {
    pub fn from(row: &Row) -> PessoaDTO {
        // COLUMNS: ID, APELIDO, NOME, NASCIMENTO, STACK
        let stack: Option<String> = row.get(4);
        let stack = match stack {
            None => None,
            Some(s) => Some(s.split(' ').map(|s| s.to_string()).collect()),
        };
        PessoaDTO {
            id: row.get(0),
            apelido: row.get(1),
            nome: row.get(2),
            nascimento: row.get(3),
            stack,
        }
    }
}

pub async fn db_count(conn: &deadpool_postgres::Client) -> Result<i64, Box<dyn std::error::Error>> {
    let rows = conn
        .query(
            "SELECT COUNT(1) FROM PESSOAS WHERE APELIDO NOT LIKE 'WARMUP%';",
            &[],
        )
        .await?;
    let count: i64 = rows[0].get(0);
    Ok(count)
}

pub async fn db_search(
    conn: &deadpool_postgres::Client,
    t: String,
) -> Result<Vec<PessoaDTO>, Box<dyn std::error::Error>> {
    let t = format!("%{}%", t.to_lowercase());
    let stmt = conn
        .prepare_cached(
            "
            SELECT ID, APELIDO, NOME, NASCIMENTO, STACK
            FROM PESSOAS P
            WHERE P.BUSCA_TRGM LIKE $1
            LIMIT 50;
        ",
        )
        .await?;
    let rows = conn.query(&stmt, &[&t]).await?;
    let result = rows
        .iter()
        .map(|row| PessoaDTO::from(row))
        .collect::<Vec<PessoaDTO>>();
    Ok(result)
}

pub async fn db_get_pessoa_dto(
    conn: &deadpool_postgres::Client,
    id: &String,
) -> Option<PessoaDTO> {
    match conn.query_one(
            "
            SELECT ID, APELIDO, NOME, NASCIMENTO, STACK
            FROM PESSOAS P
            WHERE P.ID = $1;
        ",
            &[&id],
        ).await {
        Ok(row) => Some(PessoaDTO::from(&row)),
        Err(_) => None,
    }
}

#[derive(Deserialize)]
pub struct ParametrosBusca {
    pub t: String,
}

/*
    https://docs.rs/tokio-postgres/0.7.7/src/tokio_postgres/error/sqlstate.rs.html#594
    /// 23505
    pub const UNIQUE_VIOLATION: SqlState = SqlState(Inner::E23505);
*/

pub async fn insert(conn: &deadpool_postgres::Client, id: &String, payload: web::Json<CriarPessoaDTO>)
        -> u8 {
    let stack = match &payload.stack {
        Some(v) => v.join(" "),
        None => "".to_string(),
    };

    let sql = "INSERT INTO pessoas values ($1, $2, $3, $4, $5)";
    let statement = conn.prepare(sql).await.unwrap();
    if let Err(_) = conn.execute(&statement, &[id, &payload.apelido, &payload.nome, &payload.nascimento, &stack]).await {
        /*if *e.code().unwrap() == tokio_postgres::error::SqlState::UNIQUE_VIOLATION {
            println!("Duplicate apelido!!!");
            return 0;
        }*/
        return 0;
    }

    1
}
