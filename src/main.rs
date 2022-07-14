pub mod backend;
pub mod frontend;
pub mod intermediate_representation;

fn main() {
    println!("Hello! Right now the backend is being worked on. Stay tuned for updates.");

    save_benchmarks();
}

fn save_benchmarks() {
    #[cfg(feature = "benchmark")]
    {
        use chrono::Utc;

        let dt = Utc::now();
        let timestamp: i64 = dt.timestamp();

        println!("Current timestamp is {}", timestamp);

        Benchy::save(format!("_benchmarks/run_{}.txt", timestamp).as_str());
    }
}
