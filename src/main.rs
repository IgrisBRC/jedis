#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

mod server;

fn main() {
    let mut file_path = std::env::current_dir().expect("Couldn't get current directory");
    file_path.push("dump.rdb");

    server::run(
        "127.0.0.1",
        6379,
        3,
        128,
        file_path
    );
}
