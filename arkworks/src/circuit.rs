use crate::cmp::CmpGadget;
use ark_ff::PrimeField;
use ark_r1cs_std::{
    prelude::{AllocVar, Boolean, EqGadget},
    uint8::UInt8,
};
use ark_relations::r1cs::{ConstraintSynthesizer, SynthesisError};

pub struct Puzzle<const N: usize, ConstraintF: PrimeField>(pub [[UInt8<ConstraintF>; N]; N]);
pub struct Solution<const N: usize, ConstraintF: PrimeField>(pub [[UInt8<ConstraintF>; N]; N]);

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

impl<const N: usize, ConstraintF: PrimeField> Solution<N, ConstraintF> {
    fn check_rows(&self) -> Result<(), SynthesisError> {
        for row in &self.0 {
            no_duplicates(row.iter())?;
        }
        Ok(())
    }

    fn check_cols(&self) -> Result<(), SynthesisError> {
        for col in (0..N).map(|idx| self.0.iter().map(move |row| &row[idx])) {
            no_duplicates(col)?;
        }
        Ok(())
    }

    fn check_subgrids(&self) -> Result<(), SynthesisError> {
        for i in 0..N {
            for j in 0..N {
                if i % 3 == 0 && j % 3 == 0 {
                    let subgrid = self.0[i..(i + 3)].iter().flat_map(|row| &row[j..(j + 3)]);
                    no_duplicates(subgrid)?;
                }
            }
        }
        Ok(())
    }
}

pub struct SudokuCircuit<const N: usize> {
    // The puzzle is public
    pub puzzle: [[u8; N]; N],
    // The solution is private
    pub solution: Option<[[u8; N]; N]>,
}

impl<const N: usize> SudokuCircuit<N> {
    fn check_puzzle_matches_solution<ConstraintF: PrimeField>(
        puzzle: &Puzzle<N, ConstraintF>,
        solution: &Solution<N, ConstraintF>,
    ) -> Result<(), SynthesisError> {
        for (p_row, s_row) in puzzle.0.iter().zip(&solution.0) {
            for (p, s) in p_row.iter().zip(s_row) {
                // Ensure that the solution `s` is in the range [1, N]
                (s.is_leq(&UInt8::constant(N as u8))? & (&s.is_geq(&UInt8::constant(1))?))
                    .enforce_equal(&Boolean::TRUE)?;

                // Ensure that either the puzzle slot is 0, or that
                // the slot matches equivalent slot in the solution
                (p.is_eq(s)? | (&p.is_eq(&UInt8::constant(0))?)).enforce_equal(&Boolean::TRUE)?;
            }
        }
        Ok(())
    }
}

impl<const N: usize, ConstraintF: PrimeField> ConstraintSynthesizer<ConstraintF>
    for SudokuCircuit<N>
{
    fn generate_constraints(
        self,
        cs: ark_relations::r1cs::ConstraintSystemRef<ConstraintF>,
    ) -> ark_relations::r1cs::Result<()> {
        let puzzle_var = Puzzle::new_input(cs.clone(), || Ok(self.puzzle))?;
        let solution_var = Solution::new_witness(cs.clone(), || {
            self.solution.ok_or(SynthesisError::AssignmentMissing)
        })?;
        Self::check_puzzle_matches_solution(&puzzle_var, &solution_var)?;
        solution_var.check_rows()?;
        solution_var.check_cols()?;
        solution_var.check_subgrids()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::{Bls12_381, Fr as BlsFr};
    use ark_groth16::*;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use ark_snark::SNARK;
    use rand::prelude::*;

    #[test]
    fn valid_solution_accepted() {
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

        let circuit = SudokuCircuit::<9> {
            puzzle,
            solution: None,
        };

        let rng = &mut thread_rng();

        // generate the setup parameters
        let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(circuit, rng)
            .map_err(|err| format!("Failed to generate setup parameters: {err}").to_string())
            .unwrap();

        // Check that it accepts a valid solution.
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

        let proof = Groth16::<Bls12_381>::prove(
            &pk,
            SudokuCircuit {
                puzzle,
                solution: Some(solution),
            },
            rng,
        )
        .map_err(|_| "Failed to generate proof".to_string())
        .unwrap();

        match Groth16::<Bls12_381>::verify(
            &vk,
            puzzle
                .into_iter()
                .flatten()
                .flat_map(|cell| (0..8).map(move |b| (cell >> b) & 1))
                .map(BlsFr::from)
                .collect::<Vec<_>>()
                .as_slice(),
            &proof,
        ) {
            Ok(true) => {}
            Ok(false) => panic!("Proof rejected but should have been accepted"),
            Err(err) => panic!("Failed to verify proof with vk: {err}"),
        };
    }

    #[test]
    fn invalid_solution_rejected() {
        let puzzle = [
            [5, 3, 0, 0, 7, 0, 0, 0, 0],
            [6, 0, 0, 1, 9, 5, 0, 0, 0],
            [0, 9, 8, 0, 0, 0, 0, 6, 0],
            [8, 0, 0, 0, 6, 0, 0, 0, 3],
            [4, 0, 0, 8, 0, 3, 0, 0, 1],
            [7, 0, 0, 0, 2, 0, 0, 0, 6],
            [0, 6, 0, 0, 0, 0, 2, 8, 0],
            [0, 0, 0, 4, 1, 9, 0, 0, 5],
            [0, 0, 0, 0, 8, 0, 0, 7, 9],
        ];

        let fake_puzzle = [
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

        let circuit = SudokuCircuit::<9> {
            puzzle: fake_puzzle,
            solution: None,
        };

        let rng = &mut thread_rng();

        // generate the setup parameters
        let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(circuit, rng)
            .map_err(|err| format!("Failed to generate setup parameters: {err}").to_string())
            .unwrap();

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

        // Check that it rejects an invalid proof:
        let proof = Groth16::<Bls12_381>::prove(
            &pk,
            SudokuCircuit {
                puzzle: fake_puzzle,
                solution: Some(solution),
            },
            rng,
        )
        .map_err(|_| "Failed to generate proof".to_string())
        ////// lol i wish
        //.and_then(|proof| {
        //    let mut serialized = vec![0; proof.serialized_size(ark_serialize::Compress::No)];
        //    proof
        //        .serialize_uncompressed(&mut serialized[..])
        //        .map_err(|_| "Failed to serialize proof".to_string())?;
        //    for byte in serialized.iter_mut() {
        //        // randomly flip a bit
        //        let prob = rng.gen_range(0.0..1.0);
        //        if rng.gen_bool(prob) {
        //            *byte ^= 1u8 << rng.gen_range(0..8);
        //        }
        //    }
        //    <Groth16<Bls12_381> as SNARK<BlsFr>>::Proof::deserialize_compressed(&serialized[..])
        //        .map_err(|err| format!("Failed to deserialize proof: {err}").to_string())
        //})
        .unwrap();

        //// The proof is valid, but not for the correct puzzle.
        match Groth16::<Bls12_381>::verify(
            &vk,
            puzzle
                .into_iter()
                .flatten()
                .flat_map(|cell| (0..8).map(move |b| (cell >> b) & 1))
                .map(BlsFr::from)
                .collect::<Vec<_>>()
                .as_slice(),
            &proof,
        ) {
            Ok(false) => {}
            Ok(true) => panic!("Proof accepted but should have been rejected"),
            Err(err) => panic!("Failed to verify proof with vk: {err}"),
        };
    }

    #[test]
    fn serde() {
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

        let circuit = SudokuCircuit::<9> {
            puzzle,
            solution: None,
        };

        let rng = &mut thread_rng();

        // generate the setup parameters
        let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(circuit, rng)
            .map_err(|err| format!("Failed to generate setup parameters: {err}").to_string())
            .unwrap();

        // Check that it accepts a valid solution.
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

        let proof = Groth16::<Bls12_381>::prove(
            &pk,
            SudokuCircuit {
                puzzle,
                solution: Some(solution),
            },
            rng,
        )
        .map_err(|_| "Failed to generate proof".to_string())
        .unwrap();

        let mut serialized = vec![0; proof.serialized_size(ark_serialize::Compress::No)];
        proof
            .serialize_uncompressed(&mut serialized[..])
            .map_err(|err| format!("Failed to serialize proof: {err}").to_string())
            .unwrap();

        //println!("proof: {:?}", proof.serialized_size());
        //println!("proof: {:?}", serialized);

        let pr =
            <Groth16<Bls12_381> as SNARK<BlsFr>>::Proof::deserialize_uncompressed(&serialized[..])
                .map_err(|err| format!("Failed to deserialize proof: {err}").to_string())
                .unwrap();

        assert_eq!(proof, pr);

        let mut serialized = vec![0; pk.serialized_size(ark_serialize::Compress::No)];
        pk.serialize_uncompressed(&mut serialized[..])
            .map_err(|err| format!("Failed to serialize proving key: {err}").to_string())
            .unwrap();

        // println!("pk-size: {:?}", pk.serialized_size());
        // println!("pk: {:?}", serialized);
        let p = <Groth16<Bls12_381> as SNARK<BlsFr>>::ProvingKey::deserialize_uncompressed(
            &serialized[..],
        )
        .map_err(|err| format!("Failed to deserialize proving key: {err}").to_string())
        .unwrap();

        assert_eq!(pk, p);

        let mut serialized = vec![0; vk.serialized_size(ark_serialize::Compress::No)];
        vk.serialize_uncompressed(&mut serialized[..])
            .map_err(|err| format!("Failed to serialize verifiying key: {err}").to_string())
            .unwrap();

        //println!(
        //    "vk-size: {:?}",
        //    vk.serialized_size(ark_serialize::Compress::No)
        //);
        //println!("vk: {:?}", serialized);

        let v = <Groth16<Bls12_381> as SNARK<BlsFr>>::VerifyingKey::deserialize_uncompressed(
            &serialized[..],
        )
        .map_err(|err| format!("Failed to deserialize verifiying key: {err}").to_string())
        .unwrap();
        assert_eq!(vk, v);

        match Groth16::<Bls12_381>::verify(
            &vk,
            puzzle
                .into_iter()
                .flatten()
                .flat_map(|cell| (0..8).map(move |b| (cell >> b) & 1))
                .map(BlsFr::from)
                .collect::<Vec<_>>()
                .as_slice(),
            &proof,
        ) {
            Ok(true) => {}
            Ok(false) => panic!("Proof rejected but should have been accepted"),
            Err(err) => panic!("Failed to verify proof with vk: {err}"),
        };

        match Groth16::<Bls12_381>::verify(
            &v,
            puzzle
                .into_iter()
                .flatten()
                .flat_map(|cell| (0..8).map(move |b| (cell >> b) & 1))
                .map(BlsFr::from)
                .collect::<Vec<_>>()
                .as_slice(),
            &proof,
        ) {
            Ok(true) => {}
            Ok(false) => panic!("Proof rejected but should have been accepted"),
            Err(err) => panic!("Failed to verify proof with vk: {err}"),
        };
    }
}
