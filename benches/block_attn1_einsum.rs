use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ezkl_lib::circuit::poly::PolyOp;
use ezkl_lib::circuit::*;
use ezkl_lib::execute::create_proof_circuit_kzg;
use ezkl_lib::pfsys::TranscriptType;
use ezkl_lib::pfsys::{create_keys, gen_srs};
use ezkl_lib::tensor::*;
use halo2_proofs::poly::kzg::commitment::KZGCommitmentScheme;
use halo2_proofs::poly::kzg::strategy::SingleStrategy;
use halo2_proofs::{
    arithmetic::Field,
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Circuit, ConstraintSystem, Error},
};
use halo2curves::bn256::{Bn256, Fr};
use rand::rngs::OsRng;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

static mut LEN: usize = 4;
const K: usize = 24;

#[derive(Clone)]
struct MyCircuit {
    inputs: [ValTensor<Fr>; 2],
    _marker: PhantomData<Fr>,
}

impl Circuit<Fr> for MyCircuit {
    type Config = BaseConfig<Fr>;
    type FloorPlanner = SimpleFloorPlanner;
    type Params = ();

    fn without_witnesses(&self) -> Self {
        self.clone()
    }

    fn configure(cs: &mut ConstraintSystem<Fr>) -> Self::Config {
        let len = unsafe { LEN };

        let a = VarTensor::new_advice(cs, K, 1 * 4 * len * len);

        let b = VarTensor::new_advice(cs, K, 1 * 4 * len * len);

        let output = VarTensor::new_advice(cs, K, 1 * 4 * len * len);

        Self::Config::configure(cs, &[a, b], &output, CheckMode::UNSAFE, 0)
    }

    fn synthesize(
        &self,
        mut config: Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "",
            |mut region| {
                config
                    .layout(
                        Arc::new(Mutex::new(Some(&mut region))),
                        &self.inputs,
                        &mut 0,
                        Box::new(PolyOp::Einsum {
                            equation: "abcd,abdc->abcc".to_string(),
                        }),
                    )
                    .unwrap();
                Ok(())
            },
        )?;
        Ok(())
    }
}

fn runmatmul(c: &mut Criterion) {
    let mut group = c.benchmark_group("block_attn1_einsum");
    let params = gen_srs::<KZGCommitmentScheme<_>>(24);
    for &len in [64, 80, 96, 112, 128].iter() {
        unsafe {
            LEN = len;
        };

        let mut a = Tensor::from((0..1 * 4 * len * 16).map(|_| Value::known(Fr::random(OsRng))));
        a.reshape(&[1, 4, len, 16]);

        // parameters
        let mut b = Tensor::from((0..1 * 4 * 16 * len).map(|_| Value::known(Fr::random(OsRng))));
        b.reshape(&[1, 4, 16, len]);

        let circuit = MyCircuit {
            inputs: [ValTensor::from(a), ValTensor::from(b)],
            _marker: PhantomData,
        };

        group.throughput(Throughput::Elements(len as u64));
        group.bench_with_input(BenchmarkId::new("pk", len), &len, |b, &_| {
            b.iter(|| {
                create_keys::<KZGCommitmentScheme<Bn256>, Fr, MyCircuit>(&circuit, &params)
                    .unwrap();
            });
        });

        let pk =
            create_keys::<KZGCommitmentScheme<Bn256>, Fr, MyCircuit>(&circuit, &params).unwrap();

        group.throughput(Throughput::Elements(len as u64));
        group.bench_with_input(BenchmarkId::new("prove", len), &len, |b, &_| {
            b.iter(|| {
                let prover = create_proof_circuit_kzg(
                    circuit.clone(),
                    &params,
                    vec![],
                    &pk,
                    TranscriptType::Blake,
                    SingleStrategy::new(&params),
                    CheckMode::UNSAFE,
                );
                prover.unwrap();
            });
        });
    }
    group.finish();
}

criterion_group! {
  name = benches;
  config = Criterion::default().sample_size(10).with_plots();
  targets = runmatmul
}
criterion_main!(benches);
