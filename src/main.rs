#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

pub mod handlers;
pub mod models;

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![

            ],
        )
        .launch();
}

/*
 * GET /api/files/{id}
 * PUT /api/files/{id}
 * POST /api/files
 */
