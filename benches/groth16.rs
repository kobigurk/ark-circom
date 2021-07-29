use criterion::{criterion_group, criterion_main, Criterion};

use ark_circom::{CircomBuilder, CircomConfig, R1CStoQAPCircom};
use ark_std::{
    UniformRand,
    rand::thread_rng,
};

use ark_bn254::{Bn254, Fr};
use ark_groth16::{
    create_proof_with_qap_and_matrices as prove, generate_random_parameters_with_qap, prepare_verifying_key, verify_proof,
};
use ark_relations::r1cs::{OptimizationGoal, ConstraintSystem, ConstraintSynthesizer};
use std::ops::Deref;
use ark_groth16::r1cs_to_qap::R1CStoQAP;

fn groth(c: &mut Criterion) {
    let cfg = CircomConfig::<Bn254>::new(
        "./test-vectors/complex-circuit.wasm",
        "./test-vectors/complex-circuit.r1cs",
    ).unwrap();

    let mut builder = CircomBuilder::new(cfg);
    builder.push_input("a", 3);

    // create an empty instance for setting it up
    let circom = builder.setup();

    let mut rng = thread_rng();
    let params = generate_random_parameters_with_qap::<Bn254, _, _, R1CStoQAPCircom>(circom, &mut rng).unwrap();

    let circom = builder.build().unwrap();

    let inputs = circom.get_public_inputs().unwrap();

    let cs = ConstraintSystem::new_ref();

    // Set the optimization goal
    cs.set_optimization_goal(OptimizationGoal::Constraints);

    // Synthesize the circuit.
    circom.generate_constraints(cs.clone()).unwrap();
    debug_assert!(cs.is_satisfied().unwrap());

    cs.finalize();

    let prover = cs;

    let matrices = prover.to_matrices().unwrap();
    let num_inputs = prover.num_instance_variables();
    let num_constraints = prover.num_constraints();

    let cs = prover.borrow().unwrap();
    let prover = cs.deref();

    let full_assignment = [
        prover.instance_assignment.as_slice(),
        prover.witness_assignment.as_slice(),
    ]
        .concat();

    let r = Fr::rand(&mut rng);
    let s = Fr::rand(&mut rng);

    let proof = prove::<_, R1CStoQAPCircom>(&params, r, s, &matrices, num_inputs, num_constraints, &full_assignment).unwrap();
    let pvk = prepare_verifying_key(&params.vk);
    let verified = verify_proof(&pvk, &proof, &inputs).unwrap();
    assert!(verified);

    c.bench_function("groth proof", |b| b.iter(|| {
        prove::<_, R1CStoQAPCircom>(&params, r, s, &matrices, num_inputs, num_constraints, &full_assignment).unwrap();
    }));
}

criterion_group!(benches, groth);
criterion_main!(benches);
