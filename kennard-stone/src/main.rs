use rand::Rng;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fmt::Debug;

/*
http://wiki.eigenvector.com/index.php?title=Kennardstone
Description

The KENNARDSTONE method selects a subset of samples from x which provide
uniform coverage over the data set and includes samples on the boundary
of the data set. The method begins by finding the two samples which are
farthest apart using geometric distance. To add another sample to the
selection set the algorithm selects from the remaining samples that one
which has the greatest separation distance from the selected samples.
The separation distance of a candidate sample from the selected set
is the distance from the candidate to its closest selected sample.
This most separated sample is then added to the selection set and the
process is repeated until the required number of samples, k, have
been added to the selection set. In practice this produces a very
uniformly distributed network of selected points over the data set and
includes samples along the boundary of the dataset. The method performs
efficiently because it calculates the inter-sample distances matrix only
once.

The method is implemented following the description published in R. W.
Kennard & L. A. Stone (1969): Computer Aided Design of Experiments,
Technometrics, 11:1, 137-148.
*/

// (row, col) comparison => index in upper triangle
fn cmp_index(sz: usize, row: usize, col: usize) -> usize {
    if col > row {
        (sz - 1) * row + col - 1 - (0..=row).sum::<usize>()
    } else {
        (sz - 1) * col + row - 1 - (0..=col).sum::<usize>()
    }
}

fn kennard_stone<T: Debug + Copy + PartialOrd>(n: usize, dmat: &[T]) -> Vec<usize> {
    let mut first_pair = (0, 0, dmat[0]); // row, col, distance
    let mut upper_triangle = vec![];
    let mut min_distance_index = vec![0; n];
    let mut chosen = vec![];

    // convert the full distance matrix to a list representing the upper triangle
    // because I intend to receive the distance matrix in this form later...
    //
    // find the 1st pair while loading the upper triangle list
    //   prerequisite: "two samples which are farthest apart using geometric distance"

    let mut c = 0;
    for i in 0..n {
        for j in 0..n {
            if i < j {
                let distance = dmat[c];
                println!(
                    "load pair({i},{j}) at index {}, distance = {distance:?}",
                    cmp_index(n, i, j)
                );
                upper_triangle.push(distance);
                if distance > first_pair.2 {
                    first_pair = (i, j, distance);
                }
            }
            c += 1
        }
        println!();
    }
    assert!(upper_triangle.len() > 2);

    // load the 1st pair into the selection set
    chosen.push(first_pair.0);
    chosen.push(first_pair.1);
    println!("select indices of first (max_pair) {first_pair:?}");

    // create a remaining candidates hashset (omitting the first pair)
    let mut candidates = (0..n)
        .filter(|e| !(*e == first_pair.0 || *e == first_pair.1))
        .collect::<HashSet<_>>();

    // for each candidate; record which of the initial pair it's closest to
    // i.e. min_distance_index[candidate] always holds the index into upper_triangle
    //      which represents the closest value to an item in the selected set
    // at this stage there are only 2 items in the selection set (first_pair.0, first_pair.1)
    for i in candidates.iter() {
        let i1 = cmp_index(n, *i, first_pair.0);
        let i2 = cmp_index(n, *i, first_pair.1);
        min_distance_index[*i] = match upper_triangle[i1] < upper_triangle[i2] {
            true => i1,
            false => i2,
        };
    }

    // find the index of the candidate with max (minimum distance) to the selecton set
    let mut maxi = candidates
        .iter()
        .map(|c| (c, upper_triangle[min_distance_index[*c]]))
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|t| *t.0)
        .unwrap();

    // 1. move max distance candidate to the selection set
    // 2. after a candidate is moved the min_distance_index[] for
    //    the remaining candidates may need to be updated if their distance
    //    to the new selecton set entry is less than their current minimum
    // 3. find the next candidate with max distance to the selecton set
    while candidates.len() > 1 {
        println!("select index {maxi}");
        chosen.push(candidates.take(&maxi).unwrap());

        candidates
            .iter()
            .map(|c| (c, cmp_index(n, *c, maxi)))
            .for_each(|(c, index)| {
                if upper_triangle[index] < upper_triangle[min_distance_index[*c]] {
                    min_distance_index[*c] = index
                }
            });

        maxi = candidates
            .iter()
            .map(|c| (c, upper_triangle[min_distance_index[*c]]))
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|t| *t.0)
            .unwrap();
    }
    println!("select index {maxi}");
    chosen.push(maxi);
    chosen
}

// ====================================================================================

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} n", args[0]);
        std::process::exit(0);
    }
    let n = args[1].parse::<usize>()?;

    // create a distance matrix (n x n filled with random values [1 , 999])
    let mut rng = rand::thread_rng();
    let mut dmat = vec![];
    for i in 0..n {
        for j in 0..n {
            match (i, j) {
                (x, y) if x < y => dmat.push(rng.gen_range(1.0..999.0)),
                (x, y) if x > y => dmat.push(dmat[n * y + x]),
                _ => dmat.push(0.0),
            }
        }
    }
    println!("kennard_stone selected indices = {:?}", kennard_stone(n, &dmat));
    Ok(())
}
