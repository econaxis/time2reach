# [GTFS](https://gtfs.org/) Model ![crates.io](https://img.shields.io/crates/v/gtfs-structures.svg) [![](https://docs.rs/gtfs-structures/badge.svg)](https://docs.rs/gtfs-structures)

The [General Transit Feed Specification](https://gtfs.org/) (GTFS) is a commonly used model to represent public transit data.

This crates brings [serde](https://serde.rs) structures of this model and helpers to read GTFS archives.

## Using

This crates has 2 main entry-points.

### Gtfs
The most common one is to create a `gtfs_structures::Gtfs`:

```rust
// Gtfs::new will try to guess if you provide a path, a local zip file or a remote zip file.
// You can also use Gtfs::from_path or Gtfs::from_url
let gtfs = gtfs_structures::Gtfs::new("path_of_a_zip_or_directory_or_url")?;
println!("there are {} stops in the gtfs", gtfs.stops.len());

// This structure is the easiest to use as the collections are `HashMap`,
// thus you can access an object by its id.
let route_1 = gtfs.routes.get("1").expect("no route 1");
println!("{}: {:?}", route_1.short_name, route_1);
```

### RawGtfs

If you want a lower level model, you can use `gtfs_structures::RawGtfs`:

```rust
let raw_gtfs = RawGtfs::new("fixtures/basic").expect("impossible to read gtfs");
for stop in raw_gtfs.stops.expect("impossible to read stops.txt") {
    println!("stop: {}", stop.name);
}
```

Instead of easy to use `HashMap`, each collection is a `Result` with an error if something went wrong during the reading.

This makes it possible for example for a [GTFS validator](https://github.com/etalab/transport-validator/) to display better error messages.

### Feature 'read-url'

By default the feature 'read-url' is activated. It makes it possible to read a Gtfs from an url.

```rust
let gtfs = gtfs_structures::Gtfs::new("http://www.metromobilite.fr/data/Horaires/SEM-GTFS.zip")?;
```

Or you can use the explicit constructor:
```rust
let gtfs = gtfs_structures::Gtfs::from_url("http://www.metromobilite.fr/data/Horaires/SEM-GTFS.zip")?;
```

If you don't want the dependency to `reqwest`, you can remove this feature.

## Building

You need an up to date rust tool-chain (commonly installed with [rustup](https://rustup.rs/)).

Building is done with:

`cargo build`

You can also run the unit tests:

`cargo test`

And run the examples by giving their names:

`cargo run --example gtfs_reading`

# Alternative

If you are interested in transit data, you can also use the really nice crate [transit_model](https://github.com/CanalTP/transit_model) that can also handle GTFS data.

The price to pay is a steeper learning curve (and a documentation that could be improved :roll_eyes:), but this crate provides very nice ergonomics to handle transit data and lots of utilities like data format conversions, datasets merge, ...
