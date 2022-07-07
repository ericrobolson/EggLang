pub mod error;
pub mod location;
pub mod tokenizer;

use benchy::Benchy;

fn main() {
    {
        Benchy::time("Print world");
        println!("Hello, world!");
    }
    save_benchmarks();
}

fn save_benchmarks() {
    use chrono::Utc;

    let dt = Utc::now();
    let timestamp: i64 = dt.timestamp();

    println!("Current timestamp is {}", timestamp);

    Benchy::save(format!("_benchmarks/run_{}.txt", timestamp).as_str());
}
