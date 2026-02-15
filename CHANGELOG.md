# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

### Features

- _(api)_ Add version, created_at, updated_at to User responses - ([fc5679f](https://github.com///commit/fc5679f7da1220111f4005b8bf68d3d97bcb37e6))
- _(api)_ Add gRPC UserService and refactor to SystemService - ([a8446e5](https://github.com///commit/a8446e5736547571bb44647e9eff78b51e6b0a1b))
- _(api)_ Add gRPC error mapping from core ErrorKind to tonic Status - ([9972ff5](https://github.com///commit/9972ff55be568238975ab5378ad84796a1c7afdc))
- _(api)_ Basic GRPC server - ([9b0a415](https://github.com///commit/9b0a415a488d78d8282ad8dd83af527a08a6f405))
- _(api)_ Api framework - ([143797a](https://github.com///commit/143797a367f7ad6604186a90ff697b37f71b70f6))
- _(api,core)_ User CRUD api - ([f59e936](https://github.com///commit/f59e9360d5563f0b9bb4b0fff18bc9854a7e2226))
- _(api,core)_ Adding an api route to core services - ([b93e48b](https://github.com///commit/b93e48b072d6afc7cc957a39f42608d446ff3198))
- _(api,core,database)_ Add age field to User with user_info storage - ([3ad7974](https://github.com///commit/3ad7974f9b5a759bd01b6720e8d649c381de0b1a))
- _(api,core,database)_ Add UUID token field to users - ([c0262a3](https://github.com///commit/c0262a3ce43b76b9ef3778c32a55efc28a7cd6b1))
- _(api,core,database)_ Implement user CRUD use cases - ([cada057](https://github.com///commit/cada057686d050032e958d39efafda0e03846e4f))
- _(cli)_ Command arguments framework - ([b848ee9](https://github.com///commit/b848ee954a89981e40a0c4bfcc6f48c5680a6090))
- _(cli)_ Config framework - ([6cd97b3](https://github.com///commit/6cd97b38216d85d03b6de92ee3b9d1186f22a1a2))
- _(cli,core,database)_ Switch to Entity First Workflow - ([0e93303](https://github.com///commit/0e93303fb579ae1b0b5593749d3e000e6f788be1))
- _(cli,frontend)_ Add user API endpoint and wire frontend middleware - ([60201c0](https://github.com///commit/60201c0ef35c4bacc1a06e55301019b970211cf9))
- _(core)_ Add transaction helper functions with auto commit/rollback - ([111d402](https://github.com///commit/111d40215aad31f50f8fd5cf413d9ac03ad8875a))
- _(core,database)_ CoreServices framework - ([53233f3](https://github.com///commit/53233f3c3068ea5f20eeebda39d605d9b8e99cb5))
- _(core,database)_ Added update for User service - ([f8a97cc](https://github.com///commit/f8a97cce4974e4e35069ca37e39fd7d2b1870954))
- _(core,database)_ Beginning of User service - ([9e56714](https://github.com///commit/9e56714ff27569074de7e8a4ec3e8b7b0e1a9aa9))
- _(core,database)_ Beginning of repository and transactions - ([209b468](https://github.com///commit/209b4688c874120a782ae650792f69b4e50e0a4a))
- _(database)_ Database framework - ([e040116](https://github.com///commit/e0401167d2e6a1243a845aa4982fb74e5b1a3207))
- _(deps)_ Update renovatebot/github-action action to v46.1.0 - ([dcf6939](https://github.com///commit/dcf693917820b5152af8fd5babd76378bbcab9eb))
- _(deps)_ Update renovatebot/github-action action to v40.3.6 - ([2c15e36](https://github.com///commit/2c15e36170e8319f2d41d85403b626c9e7cbaead))
- _(utils)_ Add Token::generate() for random token creation - ([febfedd](https://github.com///commit/febfedd007154bcff8a100c19ebbcff70ce51a7a))
- _(utils)_ Generalize Token to support u64 and u128 backing types - ([95c7fd9](https://github.com///commit/95c7fd92c87dcae5bdb97ac37f10fd4360e14a81))
- _(utils)_ Add typed, prefixed token identifiers - ([09e3626](https://github.com///commit/09e3626b224d50a571e7023690909971b4fa6fbc))
- Adding Dioxus frontend - ([0da56c8](https://github.com///commit/0da56c8df48866cc04188c4cf06ed2b1c67b42f9))
- Filling in basic CLI for user CRUD operations - ([be45937](https://github.com///commit/be45937a2e67115b7736ca4581d176f77d7e8e1c))
- Exploring entity-first vs migrations - ([779ecb8](https://github.com///commit/779ecb854733f537dcd73363bf097806baf533e2))

### Bug Fixes

- _(cli)_ Need to initialize logging first - ([0d4cde5](https://github.com///commit/0d4cde5f4a99c77058b1d8a0a457b4f2a31cf644))

### Refactor

- _(api)_ Simplify HTTP server setup with axum::serve - ([1ae04e0](https://github.com///commit/1ae04e05174ae617e79487b30994a2be27b1a02a))
- _(api,core)_ Move API infrastructure errors to api crate - ([6c7e1c7](https://github.com///commit/6c7e1c79b7e1ccd7171fb6baa774da72c9c7eec4))
- _(api,core)_ Extract shared MockUserService to core test_support - ([11b0a34](https://github.com///commit/11b0a347f2cb7315f982749a58e1e8e2c298bf7c))
- _(api,core,database)_ Adopt Email and Age newtypes in domain models - ([20768a5](https://github.com///commit/20768a5e1b458454a731d32cfbb88f3d4cdb8cdd))
- _(api,core,database)_ Improve error handling, add newtypes, and consolidate test infrastructure - ([d2e63c7](https://github.com///commit/d2e63c777018d3672a293e6da538a5dea25098a1))
- _(api,core,database)_ Improve code quality and reduce duplication - ([8fc0806](https://github.com///commit/8fc08064417560d8ee041e14d68cfe8b22663684))
- _(cli,core,frontend)_ Wire frontend to core services - ([0d3d287](https://github.com///commit/0d3d287a90500f2345a753af27e3423b7fd22eee))
- _(core)_ Restructure modules - extract repositories from services - ([330f94f](https://github.com///commit/330f94fd7a43d44ccf9272a71ea0c743ceead832))
- _(core)_ Extend transaction macros to support multiple services - ([fb52be1](https://github.com///commit/fb52be15607133a511092d8b4889344263dd036e))
- _(core)_ Improve module organization and add NewUser model - ([9737a45](https://github.com///commit/9737a4597562a390a947ab4c8415ef3b6fddb4bb))
- _(core,api)_ Move UserService to services module - ([cb3aa60](https://github.com///commit/cb3aa60909ee04559bf523254afa8747533a0663))
- _(core,api,database)_ Rename User::test to User::fake - ([4dc683a](https://github.com///commit/4dc683a88a96121c4d0010917baceb7a7586edaa))
- _(core,database)_ Remove user_info table and consolidate age into users - ([18db51c](https://github.com///commit/18db51ce98a0159bcf7d734d2fde80b340e64de0))
- _(core,database)_ Use derive_builder for RepositoryService construction - ([c52ccc7](https://github.com///commit/c52ccc7853898a43f73616c54cec7c70040f4214))
- _(core,database)_ Rename UserService to UserRepository - ([fec1eb0](https://github.com///commit/fec1eb0fa2cb6567c7965be283155d128f5945a1))
- _(core,database)_ Improve delete_user with optimistic locking - ([04abbd4](https://github.com///commit/04abbd41db81d25f4e3eebe126c8167d8aaa5a70))
- _(core,database,api)_ Use idiomatic combinators and flatten module structure - ([7e71ec5](https://github.com///commit/7e71ec5a0c6998bd2a1940191e48011af9764790))
- _(database)_ Replace mock database tests with SQLite in-memory - ([51b84f4](https://github.com///commit/51b84f4edd178acb370b829205e64ef194ce1d2a))
- Use find_with_related for list_users - ([cf64468](https://github.com///commit/cf64468f60ca413703090a582719405efcfa122d))

### Stying

- Formatting - ([951d27a](https://github.com///commit/951d27aac2cac973b9e5ddf28e67b630ae0cad15))

### Testing

- _(api)_ Add comprehensive tests for gRPC services - ([bf934c0](https://github.com///commit/bf934c05fb26e366d927553320fb21c96445a611))
- _(api)_ Add HTTP endpoint tests for user routes - ([8d83dbf](https://github.com///commit/8d83dbf821cf57e92469afd93b641ee31fba8b4c))
- _(core,database)_ Add unit tests for user use cases and adapter - ([32b3b44](https://github.com///commit/32b3b442ba7c6605f5a411443cbc0745fed73e0a))

### Miscellaneous Tasks

- _(config)_ Migrate config .renovaterc.json5 ([#3](https://github.com/szinn/hex-play/issues/3)) - ([3a7de6d](https://github.com///commit/3a7de6d5a6a5b17243c7788c517d98c4484acbd5))
- Don't need to turn bacon-ls logging off in env - ([77ddd08](https://github.com///commit/77ddd08e2503a733c6d245e0c8f36eadf1892737))
- Update crates - ([15ee446](https://github.com///commit/15ee446cbbad107dffcddadfcd03962f68240568))
