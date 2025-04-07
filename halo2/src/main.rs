use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::Field,
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{Advice, Column, Error},
    poly::Rotation,
};

struct NeqChip<F: Field> {
    config: NeqConfig,
    _marker: PhantomData<F>,
}

#[derive(Clone, Debug)]
struct NeqConfig {
    col: Column<Advice>,
}

impl<F: Field> Chip<F> for NeqChip<F> {
    type Config = NeqConfig;

    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

/// instructions
impl<F: Field> NeqChip<F> {
    fn load_private(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<F>,
    ) -> Result<AssignedCell<F, F>, Error> {
        let config = self.config();

        layouter.assign_region(
            || "load private",
            |mut region| region.assign_advice(|| "private input", config.col, 0, || value),
        )
    }

    fn configure(
        meta: &mut halo2_proofs::plonk::ConstraintSystem<F>,
        advice: Column<Advice>,
    ) -> NeqConfig {
        meta.enable_equality(advice);

        meta.create_gate("neq", |meta| {
            let a = meta.query_advice(advice, Rotation::cur());
            let b = meta.query_advice(advice, Rotation::next());

            vec![a - b]
        });

        NeqConfig { col: advice }
    }
}

fn main() {
    println!("Hello, world!");
}
