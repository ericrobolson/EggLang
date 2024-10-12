

wc-gen: FORCE
	cd wc-gen && cargo run

wc-test:
	cd wc-gen && cargo test

cpp: FORCE
	cd cppoutput && make

run-cpp: cpp
	cd cppoutput && ./main

FORCE:
