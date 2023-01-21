fn main() {
    /* Gtfs::new will try to guess if you provide a path, a local zip file or a remote zip file.
       You can also use Gtfs::from_path, Gtfs::from_url
    */
    let gtfs = gtfs_structures::GtfsReader::default()
        .read_stop_times(false)
        .raw()
        .read("fixtures/basic")
        .expect("impossible to read gtfs");
    gtfs.print_stats();

    let routes = gtfs.routes.expect("impossible to read routes");
    let route_1 = routes.first().expect("no route");
    println!("{}: {:?}", route_1.short_name, route_1);
}
