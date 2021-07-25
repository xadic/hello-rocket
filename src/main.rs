#[macro_use]
extern crate diesel;
use std::collections::HashMap;

use rocket::{
    delete, get, post, put, routes,
    serde::{
        json::{serde_json::json, Json, Value},
        Deserialize, Serialize,
    },
    tokio::sync::Mutex,
    State,
};
use rocket_sync_db_pools::database;

// map -> mutex -> state
type PersonItems = Mutex<HashMap<usize, Person>>;
type Messages<'r> = &'r State<PersonItems>;

#[database("sqlite_path")]
struct DbConn(diesel::SqliteConnection);

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
struct Person {
    id: usize,
    name: String,
}

#[get("/")]
fn index() -> &'static str {
    "hello world!"
}

#[get("/haha")]
fn idiot() -> &'static str {
    let s = "you are idiot";
    s
}

#[get("/person/<id>")]
async fn get_person(id: usize, messages: Messages<'_>) -> Json<Person> {
    let person_map = messages.lock().await;
    if id == 0 {
        return Json(Person {
            id: 0,
            name: "_".to_string(),
        });
    }

    match person_map.get(&id) {
        None => Json(Person {
            id: 0,
            name: "".to_string(),
        }),
        Some(p) => Json(p.to_owned()),
    }
}

#[post("/person", format = "json", data = "<person>")]
async fn create_person(person: Json<Person>, messages: Messages<'_>) -> Value {
    let mut person_map = messages.lock().await;
    let new_person = person.into_inner();
    if person_map.contains_key(&new_person.id) {
        json!({"res": "err"})
    } else {
        person_map.insert(new_person.id, new_person);
        json!({"res": "ok"})
    }
}

#[put("/person/<id>", format = "json", data = "<person>")]
async fn put_person(id: usize, person: Json<Person>, messages: Messages<'_>) -> Value {
    let mut person_map = messages.lock().await;
    let new_person = person.into_inner();
    if id != new_person.id {
        return json!({"res": "err"});
    }
    if person_map.contains_key(&id) {
        person_map.insert(new_person.id, new_person);
        json!({"res": "ok"})
    } else {
        json!({"res": "err"})
    }
}

#[delete("/person/<id>")]
async fn delete_person(id: usize, messages: Messages<'_>) -> Value {
    let mut person_map = messages.lock().await;
    if person_map.contains_key(&id) {
        person_map.remove(&id);
        json!({"res": "ok"})
    } else {
        json!({"res": "err"})
    }
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rocket::build()
        .manage(PersonItems::new(HashMap::new()))
        .mount("/", routes![index])
        .mount("/cute", routes![idiot])
        .mount(
            "/rest",
            routes![get_person, create_person, put_person, delete_person],
        )
        .attach(DbConn::fairing())
        .launch()
        .await?;

    Ok(())
}
