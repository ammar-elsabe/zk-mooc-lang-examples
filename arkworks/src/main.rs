mod alloc;
mod circuit;
//use circuit::*;
mod cmp;

fn main() {}
//
//#[derive(Debug)]
//enum MainError {
//    ProcessingError(String),
//    ShouldHavePassed,
//    ShouldHaveFailed,
//}
//
//impl std::fmt::Display for MainError {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        match self {
//            MainError::ProcessingError(s) => write!(f, "Processing error: {}", s),
//            MainError::ShouldHavePassed => write!(f, "Should have passed but failed"),
//            MainError::ShouldHaveFailed => write!(f, "Should have failed but passed"),
//        }
//    }
//}
//
//impl Error for MainError {}
//
//fn main() -> Result<(), MainError> {
//
//    // Check that it rejects a solution with a repeated number in a row.
//    let solution = [
//        [1, 1, 4, 8, 6, 5, 2, 3, 7],
//        [7, 3, 5, 4, 1, 2, 9, 6, 8],
//        [8, 6, 2, 3, 9, 7, 1, 4, 5],
//        [9, 2, 1, 7, 4, 8, 3, 5, 6],
//        [6, 7, 8, 5, 3, 1, 4, 2, 9],
//        [4, 5, 3, 9, 2, 6, 8, 7, 1],
//        [3, 8, 9, 6, 5, 4, 7, 1, 2],
//        [2, 4, 6, 1, 7, 9, 5, 8, 3],
//        [5, 1, 7, 2, 8, 3, 6, 9, 4],
//    ];
//
//    let Ok(proof) = Groth16::<Bls12_381>::prove(
//        &pk,
//        SudokuCircuit {
//            puzzle,
//            solution: Some(solution),
//        },
//        rng,
//    ) else {
//        return Err(MainError::ProcessingError(
//            "Failed to generate proof".to_string(),
//        ));
//    };
//
//    let mut serialized = vec![0; proof.serialized_size(ark_serialize::Compress::No)];
//    let Ok(_) = proof.serialize_uncompressed(&mut serialized[..]) else {
//        return Err(MainError::ProcessingError(
//            "Failed to serialize proof".to_string(),
//        ));
//    };
//
//    // println!("proof: {:?}", proof.serialized_size());
//    // println!("proof: {:?}", serialized);
//
//    let Ok(pr) =
//        <Groth16<Bls12_381> as SNARK<BlsFr>>::Proof::deserialize_uncompressed(&serialized[..])
//    else {
//        return Err(MainError::ProcessingError(
//            "Failed to deserialize proof".to_string(),
//        ));
//    };
//    assert_eq!(proof, pr);
//
//    let mut serialized = vec![0; pk.serialized_size(ark_serialize::Compress::No)];
//    let Ok(_) = pk.serialize_uncompressed(&mut serialized[..]) else {
//        return Err(MainError::ProcessingError(
//            "Failed to serialize proving key".to_string(),
//        ));
//    };
//
//    // println!("pk-size: {:?}", pk.serialized_size());
//    // println!("pk: {:?}", serialized);
//    let Ok(p) =
//        <Groth16<Bls12_381> as SNARK<BlsFr>>::ProvingKey::deserialize_uncompressed(&serialized[..])
//    else {
//        return Err(MainError::ProcessingError(
//            "Failed to deserialize proving key".to_string(),
//        ));
//    };
//    assert_eq!(pk, p);
//
//    let mut serialized = vec![0; vk.serialized_size(ark_serialize::Compress::No)];
//    let Ok(_) = vk.serialize_uncompressed(&mut serialized[..]) else {
//        return Err(MainError::ProcessingError(
//            "Failed to serialize verifying key".to_string(),
//        ));
//    };
//
//    // println!("vk-size: {:?}", vk.serialized_size());
//    // println!("vk: {:?}", serialized);
//
//    let Ok(v) = <Groth16<Bls12_381> as SNARK<BlsFr>>::VerifyingKey::deserialize_uncompressed(
//        &serialized[..],
//    ) else {
//        return Err(MainError::ProcessingError(
//            "Failed to deserialize verifying key".to_string(),
//        ));
//    };
//    assert_eq!(vk, v);
//
//    match Groth16::<Bls12_381>::verify(
//        &vk,
//        puzzle
//            .as_flattened()
//            .iter()
//            .map(|cell| BlsFr::from(*cell))
//            .collect::<Vec<_>>()
//            .as_slice(),
//        &proof,
//    ) {
//        Ok(false) => {}
//        Ok(true) => return Err(MainError::ShouldHaveFailed),
//        Err(_) => {
//            return Err(MainError::ProcessingError(
//                "Failed to verify proof".to_string(),
//            ))
//        }
//    };
//
//    match Groth16::<Bls12_381>::verify(
//        &v,
//        puzzle
//            .as_flattened()
//            .iter()
//            .map(|cell| BlsFr::from(*cell))
//            .collect::<Vec<_>>()
//            .as_slice(),
//        &pr,
//    ) {
//        Ok(false) => {}
//        Ok(true) => return Err(MainError::ShouldHaveFailed),
//        Err(_) => {
//            return Err(MainError::ProcessingError(
//                "Failed to verify proof".to_string(),
//            ))
//        }
//    }
//
//    Ok(())
//}
