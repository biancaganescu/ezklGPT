#! /usr/bin/bash

mkdir logs

python3 gen.py | tee logs/gen_logs.txt

RUST_LOG=debug cargo run --bin ezkl --release forward --data input.json --model network.onnx --output output.json  -K=26 -S=2 -B=9 | tee logs/forward_log.txt 

RUST_LOG=debug cargo run --bin ezkl --release setup -D output.json -M network.onnx --params-path=../../kzg26.params --vk-path=vk.key --pk-path=pk.key --circuit-params-path=circuit.params -K=26 -S=2 -B=9  | tee logs/setup_log.txt

RUST_LOG=debug cargo run --bin ezkl --release  prove -M network.onnx -D output.json --pk-path=pk.key --proof-path=model.proof --params-path=../../kzg26.params --circuit-params-path=circuit.params | tee logs/prove_log.txt

RUST_LOG=debug cargo run --bin ezkl --release verify --proof-path=model.proof --circuit-params-path=circuit.params --vk-path=vk.key --params-path=../../kzg26.params | tee logs/verify_log.txt

ls -l | tee logs/sizes_log.txt