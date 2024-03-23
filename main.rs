use actix_web::{web, App, HttpServer, Responder, HttpResponse, get, http, HttpRequest};
use actix_cors::Cors;
use rusqlite::{params, Connection, Result};
use sha2::{Digest, Sha256};

async fn hello() -> impl Responder {
    "Hello, World!"
}

async fn goodbye() -> impl Responder {
    "Goodbye, World!"
}

async fn json_response() -> impl Responder {
    let data = serde_json::json!({
        "message": "This is a JSON response",
        "status": "success"
    });

    HttpResponse::Ok().json(data)
}

async fn handle_bye() -> impl Responder {
    println!("Received GET request to /bye");
    "Handling GET request to /bye"
}

async fn create_tables_inner() -> Result<(), rusqlite::Error> {
    let conn = Connection::open("voting_machine.db")?; 
        conn.execute(
            "CREATE TABLE IF NOT EXISTS official (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL            
            )", params![],
        )?;    
        conn.execute(
            "CREATE TABLE IF NOT EXISTS voter (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                dob TEXT NOT NULL
            )", params![],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ballot (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                voter_id INTEGER NOT NULL UNIQUE,
                office_1 TEXT,
                office_2 TEXT,
                office_3 TEXT            
            )", params![],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS candidate (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                party TEXT NOT NULL, 
                office_id TEXT NOT NULL           
            )", params![],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS office (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL                        
            )", params![],
        )?;
        Ok(())
    }

async fn create_tables() -> impl Responder {
    match create_tables_inner().await {
        Ok(_) => HttpResponse::Ok().body("Tables created successfully"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}


async fn register_voter(info: web::Json<VoterInfo>) -> impl Responder {
    match sql_register_voter(&info.name, &info.dob).await {
        Ok(_) => HttpResponse::Ok().body("Voter registered successfully"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

async fn sql_register_voter(name: &str, dob: &str) -> Result<(), rusqlite::Error> {
    let conn = Connection::open("voting_machine.db")?; 
    conn.execute(
        "INSERT INTO voter (name, dob) VALUES (?1, ?2)",
        [name, dob],
    )?;
    Ok(())
    }

async fn create_ballot(info: web::Json<BallotInfo>) -> impl Responder {
        match sql_create_ballot(&info.voter_id, &info.office_1, &info.office_2, &info.office_3).await {
            Ok(_) => HttpResponse::Ok().body("Voter registered successfully"),
            Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
        }
    }

async fn sql_create_ballot(voter_id: &str, office_1: &str, office_2: &str, office_3: &str) -> Result<(), rusqlite::Error> {
    let conn = Connection::open("voting_machine.db")?; 
    conn.execute(
        "INSERT INTO ballot (id, voter_id, office_1, office_2, office_3) VALUES (null, ?, ?, ?, ?)",
        [voter_id, office_1, office_2, office_3],
    )?;
    Ok(())
    }

async fn create_official(info: web::Json<OfficialInfo>) -> impl Responder {
        match sql_create_official(&info.username, &info.password).await {
            Ok(_) => HttpResponse::Ok().body("Official registered successfully"),
            Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
        }
    }

async fn sql_create_official(username: &str, password: &str) -> Result<(), rusqlite::Error> {
    let conn = Connection::open("voting_machine.db")?; 
    let password_hash = hash_password(password);
    conn.execute("INSERT INTO official (username, password_hash) VALUES (?, ?)", params![username, password_hash])?;
        Ok(())
    }
        
    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password);
        let result = hasher.finalize();
        format!("{:x}", result)
    }
        
async fn authenticate_official(conn: &Connection, username: &str, password: &str) -> Result<bool> {
    let password_hash = hash_password(password);
    let mut stmt = conn.prepare("SELECT EXISTS(SELECT 1 FROM official WHERE username = ? AND password_hash = ?)")?;
    let exists: bool = stmt.query_row(params![username, password_hash], |row| row.get(0))?;
         Ok(exists)
    }

async fn create_candidate(info: web::Json<CandidateInfo>) -> impl Responder {
        match sql_create_candidate(&info.name, &info.office_id, &info.party).await {
            Ok(_) => HttpResponse::Ok().body("Candidate registered successfully"),
            Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
        }
    }

    async fn sql_create_candidate(name: &str, office_id: &str, party: &str) -> Result<(), rusqlite::Error> {
    let conn = Connection::open("voting_machine.db")?; 
    conn.execute(
            "INSERT INTO candidate ( id, name, party, office_id)
            VALUES (NULL, ?1, ?2, ?3)",
            [name, party, office_id],
        )?;
        Ok(())
    }
    
async fn create_office(info: web::Json<OfficeInfo>) -> impl Responder {
        match sql_create_office(&info.id, &info.name).await {
            Ok(_) => HttpResponse::Ok().body("Official registered successfully"),
            Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
        }
    }

async fn sql_create_office(id: &str, name: &str) -> Result<(), rusqlite::Error> {
    let conn = Connection::open("voting_machine.db")?; 
    conn.execute(
            "INSERT INTO office ( id, name)
            VALUES (?, ?)",
            [id, name],
        )?;
        Ok(())
    }

#[derive(serde::Deserialize)]
struct VoterInfo {
    name: String,
    dob: String,
}

#[derive(serde::Deserialize)]
struct BallotInfo {
    voter_id: String,
    office_1: String,
    office_2: String,
    office_3: String
}

#[derive(serde::Deserialize)]
struct OfficialInfo {
    username: String,
    password: String
}

#[derive(serde::Deserialize)]
struct CandidateInfo {
    name: String,
    office_id: String,
    party: String
}

#[derive(serde::Deserialize)]
struct OfficeInfo {
    id: String,
    name: String
}

async fn register() -> impl Responder {
    match create_tables_inner().await {
        Ok(_) => HttpResponse::Ok().body("Tables created successfully"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

async fn get_voter() -> impl Responder {
    match create_tables_inner().await {
        Ok(_) => HttpResponse::Ok().body("Tables created successfully"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::permissive();
        App::new().wrap(cors)
            .service(web::resource("/hello").to(hello))
            .service(web::resource("/goodbye").to(goodbye))
            .service(web::resource("/json").to(json_response))
            .service(web::resource("/bye").route(web::get().to(handle_bye)))
            .service(web::resource("/create").route(web::get().to(create_tables)))
            .service(web::resource("/register_voter").route(web::post().to(register_voter)))
            .service(web::resource("/register_official").route(web::post().to(create_official)))
            .service(web::resource("/register_office").route(web::post().to(create_office)))
            .service(web::resource("/register_candidate").route(web::post().to(create_candidate)))
            .service(web::resource("/cast_vote").route(web::post().to(create_ballot)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}