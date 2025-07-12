use heapexchange::{
    CORS,
    create_answer,
    create_question,
    delete_answer,
    delete_question,
    read_answers,
    read_questions,
};

#[macro_use] extern crate rocket;

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![
            create_question,
            read_questions,
            delete_question,
            create_answer,
            read_answers,
            delete_answer
        ])
        .attach(CORS)
}
