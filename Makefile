#
# Cargo helpers.
#

LIBTORCH=$(shell realpath ~/torch/libtorch)
LD_LIBRARY_PATH=$(LIBTORCH)/lib

.PHONY : run
run :
	LIBTORCH=$(LIBTORCH) \
		LD_LIBRARY_PATH=$(LD_LIBRARY_PATH) \
		cargo run

.PHONY : build
build :
	cargo build

.PHONY : format
format :
	cargo fmt --


.PHONY : lint
lint :
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- \
			-D warnings \
			-A clippy::let_and_return \
			-A clippy::redundant_clone

.PHONY : test
test :
	@cargo test

.PHONY : doc
doc :
	cargo doc

.PHONY : post
post :
	curl \
		-v \
		-d '{"text":"Hello, World!"}' \
		-H "Content-Type: application/json" \
		http://localhost:3030/generate

.PHONY : post-many
post-many :
	curl \
		-s \
		-S \
		-d '{"text":"Hello, World!"}' \
		-H "Content-Type: application/json" \
		http://localhost:3030/generate > /dev/null &
	curl \
		-s \
		-S \
		-d '{"text":"Stay at home"}' \
		-H "Content-Type: application/json" \
		http://localhost:3030/generate > /dev/null &
	curl \
		-s \
		-S \
		-d '{"text":"Wash your hands"}' \
		-H "Content-Type: application/json" \
		http://localhost:3030/generate > /dev/null &
	curl \
		-s \
		-S \
		-d '{"text":"Do not touch your face"}' \
		-H "Content-Type: application/json" \
		http://localhost:3030/generate > /dev/null &

#
# Git helpers.
#

.PHONY: create-branch
create-branch :
ifneq ($(issue),)
	git checkout -b ISSUE-$(issue)
	git push --set-upstream origin $$(git branch | grep \* | cut -d ' ' -f2)
else ifneq ($(name),)
	git checkout -b $(name)
	git push --set-upstream origin $$(git branch | grep \* | cut -d ' ' -f2)
else
	$(error must supply 'issue' or 'name' parameter)
endif

.PHONY : delete-branch
delete-branch :
	@BRANCH=`git rev-parse --abbrev-ref HEAD` \
		&& [ $$BRANCH != 'master' ] \
		&& echo "On branch $$BRANCH" \
		&& echo "Checking out master" \
		&& git checkout master \
		&& git pull \
		&& echo "Deleting branch $$BRANCH" \
		&& git branch -d $$BRANCH \
		&& git remote prune origin
