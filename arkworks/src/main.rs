use ark_ff::PrimeField;
use ark_r1cs_std::{
    prelude::{AllocVar, Boolean, EqGadget},
    uint8::UInt8,
};
use ark_relations::r1cs::{ConstraintSystem, SynthesisError};
use cmp::CmpGadget;

mod alloc;
mod cmp;

pub struct Puzzle<const N: usize, ConstraintF: PrimeField>([[UInt8<ConstraintF>; N]; N]);
pub struct Solution<const N: usize, ConstraintF: PrimeField>([[UInt8<ConstraintF>; N]; N]);

fn no_duplicates<'a, T, ConstraintF: PrimeField>(cells: T) -> Result<(), SynthesisError>
where
    T: Iterator<Item = &'a UInt8<ConstraintF>> + Clone,
{
    //// cloning an iterator just copies internal state, not the elements
    for (i, cell) in cells.clone().enumerate() {
        for prior_cell in cells.clone().take(i) {
            cell.is_neq(prior_cell)?.enforce_equal(&Boolean::TRUE)?;
        }
    }
    Ok(())
}

fn check_rows<const N: usize, ConstraintF: PrimeField>(
    solution: &Solution<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    for row in &solution.0 {
        no_duplicates(row.iter())?;
    }
    Ok(())
}

fn check_cols<const N: usize, ConstraintF: PrimeField>(
    solution: &Solution<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    for col in (0..N).map(|idx| solution.0.iter().map(move |row| &row[idx])) {
        no_duplicates(col)?;
    }
    Ok(())
}

fn check_subgrids<const N: usize, ConstraintF: PrimeField>(
    solution: &Solution<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    for i in 0..N {
        for j in 0..N {
            if i % 3 == 0 && j % 3 == 0 {
                let subgrid = solution.0[i..(i + 3)]
                    .iter()
                    .flat_map(|row| &row[j..(j + 3)]);
                no_duplicates(subgrid)?;
            }
        }
    }
    Ok(())
}

fn check_puzzle_matches_solution<const N: usize, ConstraintF: PrimeField>(
    puzzle: &Puzzle<N, ConstraintF>,
    solution: &Solution<N, ConstraintF>,
) -> Result<(), SynthesisError> {
    for (p_row, s_row) in puzzle.0.iter().zip(&solution.0) {
        for (p, s) in p_row.iter().zip(s_row) {
            // Ensure that the solution `s` is in the range [1, N]
            s.is_leq(&UInt8::constant(N as u8))?
                .and(&s.is_geq(&UInt8::constant(1))?)?
                .enforce_equal(&Boolean::TRUE)?;

            // Ensure that either the puzzle slot is 0, or that
            // the slot matches equivalent slot in the solution
            (p.is_eq(s)?.or(&p.is_eq(&UInt8::constant(0))?)?).enforce_equal(&Boolean::TRUE)?;
        }
    }
    Ok(())
}

fn check_helper<const N: usize, ConstraintF: PrimeField>(
    puzzle: &[[u8; N]; N],
    solution: &[[u8; N]; N],
) -> Result<(), SynthesisError> {
    let cs = ConstraintSystem::<ConstraintF>::new_ref();
    let puzzle_var = Puzzle::new_input(cs.clone(), || Ok(puzzle))?;
    let solution_var = Solution::new_witness(cs.clone(), || Ok(solution))?;
    check_puzzle_matches_solution(&puzzle_var, &solution_var)?;
    check_rows(&solution_var)?;
    check_cols(&solution_var)?;
    check_subgrids(&solution_var)?;
    match cs.is_satisfied() {
        Ok(true) => Ok(()),
        _ => Err(SynthesisError::Unsatisfiable),
    }
}

fn main() {
    use ark_bls12_381::Fq as F;
    // Check that it accepts a valid solution.
    let puzzle = [
        [0, 0, 0, 8, 6, 0, 2, 3, 0],
        [7, 0, 5, 0, 0, 0, 9, 0, 8],
        [0, 6, 0, 3, 0, 7, 0, 4, 0],
        [0, 2, 0, 7, 0, 8, 0, 5, 0],
        [0, 7, 8, 5, 0, 0, 0, 0, 0],
        [4, 0, 0, 9, 0, 6, 0, 7, 0],
        [3, 0, 9, 0, 5, 0, 7, 0, 2],
        [0, 4, 0, 1, 0, 9, 0, 8, 0],
        [5, 0, 7, 0, 8, 0, 0, 9, 4],
    ];
    let solution = [
        [1, 9, 4, 8, 6, 5, 2, 3, 7],
        [7, 3, 5, 4, 1, 2, 9, 6, 8],
        [8, 6, 2, 3, 9, 7, 1, 4, 5],
        [9, 2, 1, 7, 4, 8, 3, 5, 6],
        [6, 7, 8, 5, 3, 1, 4, 2, 9],
        [4, 5, 3, 9, 2, 6, 8, 7, 1],
        [3, 8, 9, 6, 5, 4, 7, 1, 2],
        [2, 4, 6, 1, 7, 9, 5, 8, 3],
        [5, 1, 7, 2, 8, 3, 6, 9, 4],
    ];
    check_helper::<9, F>(&puzzle, &solution).unwrap();

    // Check that it rejects a solution with a repeated number in a row.
    let puzzle = [
        [0, 0, 0, 8, 6, 0, 2, 3, 0],
        [7, 0, 5, 0, 0, 0, 9, 0, 8],
        [0, 6, 0, 3, 0, 7, 0, 4, 0],
        [0, 2, 0, 7, 0, 8, 0, 5, 0],
        [0, 7, 8, 5, 0, 0, 0, 0, 0],
        [4, 0, 0, 9, 0, 6, 0, 7, 0],
        [3, 0, 9, 0, 5, 0, 7, 0, 2],
        [0, 4, 0, 1, 0, 9, 0, 8, 0],
        [5, 0, 7, 0, 8, 0, 0, 9, 4],
    ];
    let solution = [
        [1, 1, 4, 8, 6, 5, 2, 3, 7],
        [7, 3, 5, 4, 1, 2, 9, 6, 8],
        [8, 6, 2, 3, 9, 7, 1, 4, 5],
        [9, 2, 1, 7, 4, 8, 3, 5, 6],
        [6, 7, 8, 5, 3, 1, 4, 2, 9],
        [4, 5, 3, 9, 2, 6, 8, 7, 1],
        [3, 8, 9, 6, 5, 4, 7, 1, 2],
        [2, 4, 6, 1, 7, 9, 5, 8, 3],
        [5, 1, 7, 2, 8, 3, 6, 9, 4],
    ];
    check_helper::<9, F>(&puzzle, &solution).unwrap_err();
}
