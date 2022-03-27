COLOR ?= always # Valid COLOR options: {always, auto, never}
CARGO = cargo --color $(COLOR)
TARGET = target/wasm32-unknown-unknown
DEBUG = $(TARGET)/debug
RELEASE = $(TARGET)/release
KEYDIR ?= .keys

NAME = "party_state_actor.wasm"
SIGN_NAME = "party_state_actor_signed.wasm"

.PHONY: all bench build check clean doc test update keys keys-account keys-module

all: build

bench:
	@$(CARGO) bench

build:
	@$(CARGO) build --target wasm32-unknown-unknown --features nitro
	wascap sign $(DEBUG)/$(NAME) $(DEBUG)/$(SIGN_NAME) -i $(KEYDIR)/account.nk -u $(KEYDIR)/module.nk -g -l -z  -c tea:tpm -c tea:layer1 -c tea:keyvalue -c tea:env -c tea:ipfs -c tea:intercom -c tea:crypto -c tea:nitro -c tea:vmh-provider -n "TEA Party Contract Actor"

check:
	@$(CARGO) check

clean:
	@$(CARGO) clean

doc:
	@$(CARGO) doc

test: 
	@$(CARGO) test --features nitro -- --nocapture

update:
	@$(CARGO) update

release:
	@$(CARGO) build --release --target wasm32-unknown-unknown
	wascap sign $(RELEASE)/$(NAME) $(RELEASE)/$(SIGN_NAME) -i $(KEYDIR)/account.nk -u $(KEYDIR)/module.nk -g -l -z -c tea:tpm -c tea:layer1 -c tea:keyvalue -c tea:env -c tea:ipfs -c tea:intercom -c tea:crypto -c tea:nitro -c tea:vmh-provider -c tea:replica -c tea:tokenstate -n "TEA Party Contract Actor"

keys: keys-account
keys: keys-module

keys-account:
	@mkdir -p $(KEYDIR)
	nk gen account > $(KEYDIR)/account.txt
	awk '/Seed/{ print $$2 }' $(KEYDIR)/account.txt > $(KEYDIR)/account.nk

keys-module:
	@mkdir -p $(KEYDIR)
	nk gen module > $(KEYDIR)/module.txt
	awk '/Seed/{ print $$2 }' $(KEYDIR)/module.txt > $(KEYDIR)/module.nk
