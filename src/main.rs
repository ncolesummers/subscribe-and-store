// src/main.rs

mod mqtt;
mod sql;

fn main() {
    // Configure and run your program here
    mqtt::mqtt::connect();
    mqtt::mqtt::subscribe("topic_name");

    sql::sql::establish_connection();
    sql::sql::execute_query("SELECT * FROM table_name");

    // Additional configuration and running logic
}
