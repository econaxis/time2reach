use gtfs_structures::Gtfs;

/// prints some stats about the GTFS given as a cli argument
fn main() {
    let file_path = std::env::args()
        .nth(1)
        .expect("you should put the path of the file to load");

    println!("reading file {}", &file_path);
    let gtfs = Gtfs::new(&file_path);

    match gtfs {
        Ok(g) => {
            g.print_stats();
        }
        Err(e) => eprintln!("error: {:?}", e),
    }
}
