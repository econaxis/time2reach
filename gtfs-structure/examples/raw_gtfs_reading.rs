use gtfs_structures::RawGtfs;

fn main() {
    /* Gtfs::new will try to guess if you provide a path, a local zip file or a remote zip file.
       You can also use RawGtfs::from_path, RawGtfs::from_url
    */
    let raw_gtfs = RawGtfs::new("fixtures/basic").expect("impossible to read gtfs");

    raw_gtfs.print_stats();

    for stop in raw_gtfs.stops.expect("impossible to read stops.txt") {
        println!("stop: {}", stop.name);
    }
}
