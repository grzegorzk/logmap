SHELL=/bin/bash

DOCKER=podman

LOGMAP_IMAGE=logmap


.PHONY: test

list:
	@ $(MAKE) -pRrq -f Makefile : 2>/dev/null \
		| grep -e "^[^[:blank:]]*:$$\|#.*recipe to execute" \
		| grep -B 1 "recipe to execute" \
		| grep -e "^[^#]*:$$" \
		| sed -e "s/\(.*\):/\1/g" \
		| sort

build:
	@ ${DOCKER} build \
		-t ${LOGMAP_IMAGE} .;

test:
	@ ${DOCKER} run \
		--entrypoint /bin/bash \
		-it \
		--rm \
		-v "${CURDIR}"/tests:/opt/logmap/tests \
		-v "${CURDIR}"/src:/opt/logmap/src \
		${LOGMAP_IMAGE} -c ' \
			cd /opt/logmap \
			&& cargo test --lib --features=tst_utils -- --test-threads=5 \
			&& cargo test --test learn_line --features=tst_utils -- --test-threads=5 \
		'

run:
	@ ${DOCKER} run \
		-it \
		--rm \
		${LOGMAP_IMAGE}
