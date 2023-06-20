#[cfg(feature = "sled-storage")]
mod sled_multi_threaded {
    use {
        futures::executor::block_on,
        gluesql::{
            prelude::{Glue, Payload, Value},
            sled_storage::SledStorage,
        },
        std::thread,
        std::sync::{Arc, RwLock},
    };

    pub async fn run() {
        let storage = SledStorage::new("/tmp/gluesql/hello_world").expect("Something went wrong!");
        let mut glue = Glue::new(storage.clone());
        let queries = "
            CREATE TABLE IF NOT EXISTS greet (name TEXT);
            DELETE FROM greet;
        ";

        glue.execute(queries).await.unwrap();

        /*
            SledStorage supports cloning, using this we can create copies of the storage for new threads;
            all we need to do is wrap it in glue again.
        */
        
        let data = Arc::new(RwLock::new(0));
        
        let insert_foo_storage = storage.clone();
        let data_clone = Arc::clone(&data);
        let insert_foo_thread = thread::spawn(move || {
            let mut _data_clone = data_clone.write().unwrap();
            
            let mut glue = Glue::new(insert_foo_storage);
            let query = "INSERT INTO greet (name) VALUES ('Foo');";

            block_on(glue.execute(query)).unwrap();
        });

        let insert_world_storage = storage.clone();
        let data_clone = Arc::clone(&data);
        let insert_world_thread = thread::spawn(move || {
            let mut _data_clone = data_clone.write().unwrap();
            
            let mut glue = Glue::new(insert_world_storage);
            let query = "INSERT INTO greet (name) VALUES ('World');";

            block_on(glue.execute(query)).unwrap();
        });

        insert_world_thread
            .join()
            .expect("Something went wrong in the world thread");

        insert_foo_thread
            .join()
            .expect("Something went wrong in the foo thread");

        let query = "SELECT name FROM greet";
        let payloads = glue.execute(query).await.unwrap();
        assert_eq!(payloads.len(), 1);

        let rows = match &payloads[0] {
            Payload::Select { rows, .. } => rows,
            _ => panic!("Unexpected result: {:?}", payloads),
        };

        let first_row = &rows[0];
        let first_value = first_row.iter().next().unwrap();
        let to_greet = match first_value {
            Value::Str(to_greet) => to_greet,
            value => panic!("Unexpected type: {:?}", value),
        };

        // Will typically output "Hello Foo!" but will sometimes output "Hello World!"; depends on which thread finished first.
        println!("Hello {}!", to_greet);
    }
}

fn main() {
    #[cfg(feature = "sled-storage")]
    futures::executor::block_on(sled_multi_threaded::run());
}
