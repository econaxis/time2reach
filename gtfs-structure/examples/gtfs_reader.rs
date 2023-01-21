fn main() {
    /* Gtfs::new will try to guess if you provide a path, a local zip file or a remote zip file.
       You can also use Gtfs::from_path, Gtfs::from_url
    */
    let gtfs = gtfs_structures::GtfsReader::default()
        .read_stop_times(false)
        .read("fixtures/basic")
        .expect("impossible to read gtfs");
    gtfs.print_stats();

    println!("there are {} stops in the gtfs", gtfs.stops.len());

    let route_1 = gtfs.routes.get("1").expect("no route 1");
    println!("{}: {:?}", route_1.short_name, route_1);
}
