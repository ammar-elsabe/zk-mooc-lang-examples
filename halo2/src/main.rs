use std::{borrow::Borrow, marker::PhantomData};

use halo2_proofs::{
    arithmetic::Field,
    circuit::{AssignedCell, Chip, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Assigned, Circuit, Column, Error, Expression, Instance},
    poly::Rotation,
};

#[derive(Clone)]
struct NeqChip<F: Field> {
    config: NeqConfig,
    _marker: PhantomData<F>,
}

#[derive(Clone, Debug)]
struct NeqConfig {
    col: [Column<Advice>; 2],
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
    fn new(config: NeqConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn load_private(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<F>,
    ) -> Result<AssignedCell<F, F>, Error> {
        let config = self.config();

        layouter.assign_region(
            || "load private",
            |mut region| region.assign_advice(|| "private input", config.col[0], 0, || value),
        )
    }

    fn configure(
        meta: &mut halo2_proofs::plonk::ConstraintSystem<F>,
        advice: [Column<Advice>; 2],
    ) -> NeqConfig {
        for col in advice {
            meta.enable_equality(col);
        }

        meta.create_gate("neq", |meta| {
            let a = meta.query_advice(advice[0], Rotation::cur());
            let b = meta.query_advice(advice[0], Rotation::next());
            let inv = meta.query_advice(advice[1], Rotation::cur());
            let diff = a - b;
            vec![diff * inv - Expression::Constant(F::ONE)]
        });

        NeqConfig { col: advice }
    }

    fn neq(&self, mut layouter: impl Layouter<F>, a: Value<F>, b: Value<F>) -> Result<(), Error> {
        let a = self.load_private(layouter.namespace(|| "a"), a)?;
        let b = self.load_private(layouter.namespace(|| "b"), b)?;

        layouter.assign_region(
            || "neq",
            |mut region| {
                region.assign_advice(|| "a", self.config.col[0], 0, || a.value().cloned())?;
                region.assign_advice(|| "b", self.config.col[0], 1, || b.value().cloned())?;
                let diff = a.value().cloned() - b.value().cloned();
                // If diff equals zero, inversion fails.
                //let inv_ = diff.invert();
                let inv = diff.and_then(|v| match v.invert() {
                    inv if inv.is_some().into() => Value::known(inv.unwrap()),
                    _ => Value::unknown(),
                });

                region.assign_advice(|| "inverse", self.config.col[1], 0, move || inv)?;
                Ok(())
            },
        )
    }
}

// This gadget is built on top of the previously implemented NeqChip.
struct NoDuplicatesGadget<F: Field> {
    // Our gadget reuses a NeqChip instance.
    neq_chip: NeqChip<F>,
}

impl<F: Field> NoDuplicatesGadget<F> {
    /// Constructs the gadget with an instance of the NeqChip.
    pub fn new(neq_chip: NeqChip<F>) -> Self {
        Self { neq_chip }
    }

    /// Given a sorted slice of values, iterates over each consecutive pair and
    /// calls the NeqChip’s `neq` method to enforce a non‑equality constraint.
    ///
    /// Because the array is assumed to be sorted, if any duplicates exist they
    /// will appear as consecutive equal values.
    pub fn nodup(&self, mut layouter: impl Layouter<F>, values: &[Value<F>]) -> Result<(), Error> {
        // We must perform the neq check for every adjacent pair.
        for (i, a_val) in values.iter().enumerate() {
            for b_val in values.iter().take(i) {
                // Create a namespace (region) for each pair. You could also group
                // them into one larger region if desired.
                self.neq_chip.neq(
                    layouter.namespace(|| format!("neq pair {}", i)),
                    *a_val,
                    *b_val,
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Default)]
struct SudokuCircuit<const N: usize, F: Field> {
    // The puzzle is public
    //pub puzzle: Value<[[F; N]; N]>,
    // The solution is private
    solution: Value<[[F; N]; N]>,
}

#[derive(Clone)]
struct SudokuConfig<const N: usize, F: Field> {
    // The rows for the Sudoku circuit
    advice: [Column<Advice>; N],
    // public rows
    public: [Column<Instance>; N],
    /// neq chips used for the gadget
    chips: [NeqChip<F>; 3],
}

impl<const N: usize, F: Field> Circuit<F> for SudokuCircuit<N, F> {
    type Config = SudokuConfig<N, F>;

    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Default::default()
    }

    fn configure(meta: &mut halo2_proofs::plonk::ConstraintSystem<F>) -> Self::Config {
        SudokuConfig {
            advice: std::array::from_fn(|_| meta.advice_column()),
            public: std::array::from_fn(|_| meta.instance_column()),
            chips: std::array::from_fn(|_| {
                let advice = std::array::from_fn(|_| meta.advice_column());
                let config = NeqChip::configure(meta, advice);
                NeqChip::new(config)
            }),
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let [no_dup_rows, no_dup_cols, no_dup_nonet] = config.chips;
        let gadget = NoDuplicatesGadget::new(no_dup_rows);
        self.solution.map(|solution| {
            for (i, row) in solution.iter().enumerate() {
                layouter.assign_region(
                    || format!("row {i}"),
                    |mut region| {
                        for (j, cell) in row.iter().enumerate() {
                            region.assign_advice(
                                || "cell",
                                config.advice[i],
                                j,
                                || Value::known(*cell),
                            )?;
                        }
                        Ok(())
                    },
                );
            }
        });
        todo!()
    }
}

fn main() {
    println!("Hello, world!");
}
