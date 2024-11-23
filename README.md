# extendr-vctrs

Turn any rust struct into a [{vctrs}](https://github.com/r-lib/vctrs) vector by implementing the `Rvctr` trait.

All `Vec<Option<T>>` can be used turned into a `vctrs_vctr` out of the box!


## Usage

This is done with the `Vctr` struct. For example we can create a custom `Point` struct an return it as a vector to R.

```rust
use extendr_api::prelude::*;
use extendr_vctrs::Vctr;

#[derive(Debug, Clone)]
pub struct Point {
    x: f64,
    y: f64,
}

/// @export
#[extendr]
fn make_points(x: Doubles, y: Doubles) -> Vctr<Vec<Option<Point>>> {
    let pnts = x.into_iter().zip(y.into_iter())
    .map(|(x, y)| {
        if x.is_na() || y.is_na() {
            None
        } else {
            let pnt = Point {
                x: x.inner(),
                y: y.inner()
            };
            Some(pnt)
        }
    }).collect::<Vec<_>>();
    Vctr::from(pnts)
}
```

This creates an object of class `extendr_vctr`. As of now, there is no automated print method so we can implement that.

```rust
/// @export
#[extendr(r_name = "format.extendr_vctr")]
fn show_points(x: Vctr<Vec<Option<Point>>>) -> Result<Strings> {
    x.show()
}
```

Be sure to add both `fn make_points` and `fn show_points` to your `extendr_module!()`.

Run `rextendr::document()` then `devtools::load_all()`

```r
x <- runif(10, -180, 180)
y <- runif(10, -90, 90)

pnts <- make_points(x, y)
pnts
#> <extendr_vctr[10]>
#>  [1] Point { x: 21.929318541660905, y: 89.07896332908422 }
#>  [2] Point { x: 46.90753617323935, y: 76.01803659461439 }
#>  [3] Point { x: 76.39162237755954, y: 48.584263189695776 }
#>  [4] Point { x: -149.61945047602057, y: -66.95100508164614 }
#>  [5] Point { x: 89.15390370413661, y: -29.644039529375732 }
#>  [6] Point { x: 114.51665648259223, y: 14.32901468127966 }
#>  [7] Point { x: -3.3372477907687426, y: 25.24383968207985 }
#>  [8] Point { x: -169.91857962682843, y: 37.487053759396076 }
#>  [9] Point { x: 99.65098458342254, y: 44.39496916718781 }
#> [10] Point { x: -172.7898272126913, y: 4.226361438632011 }
```

## Rvctr & Vctr<T>

The `Rvctr` trait defines the behavior required for any struct to be represented as a vctrs_vctr in R. The struct must be able to give us:
  - a class name
  - a character vector to be used for printing
  - the length of the vector
  - the ability to subset the vector
  - the ability extend the vector


Any object that implements the `Rvctr` trait can provided as an argument to an extendr function if it is wrapped in the `Vctr<T>` struct.

`Vctr<T>` is a struct that handles the conversion to and from R. It has implemented `TryFrom<Robj>` for all structs that have implemented the `Rvctr` trait.


## How it works

`extendr-vctrs` uses the ideas in [`geoarrow-r`](https://github.com/geoarrow/geoarrow-r).

An `extendr_vctr` is an integer vector with an attribute `extendr_ptr` which contains an external pointer to the struct.

``` r
attributes(pnts)
#> $extendr_ptr
#> <pointer: 0x1278590b0>
#>
#> $class
#> [1] "extendr_vctr" "vctrs_vctr"
```

When the object is passed into Rust as a function argument, the struct is fetched from the external pointer in the attribute.

When the object is returned to R, an integer vector is constructed with the struct in the `extendr_ptr` attribute.
