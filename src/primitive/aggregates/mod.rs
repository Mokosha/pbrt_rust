mod grid;

use primitive::aggregates::grid::GridAccelerator;

enum Aggregate {
    Grid(GridAccelerator)
}
