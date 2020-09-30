ORG=ismacaulay
APP=procrast-cli

image:
	docker build -t $(ORG)/$(APP) -f Dockerfile .

run:
	docker run --rm -it \
		-v $(shell pwd)/src:/app/src \
		-v $(shell pwd)/Cargo.toml:/app/Cargo.toml \
		-v $(shell pwd)/Cargo.lock:/app/Cargo.lock \
		-v $(shell pwd)/.docker:/app/target \
		--network="host" \
		$(ORG)/$(APP) /bin/bash

run2:
	docker run --rm -it \
		-v $(shell pwd)/src:/app/src \
		-v $(shell pwd)/Cargo.toml:/app/Cargo.toml \
		-v $(shell pwd)/Cargo.lock:/app/Cargo.lock \
		-v $(shell pwd)/.docker2:/app/target \
		--network="host" \
		$(ORG)/$(APP) /bin/bash
