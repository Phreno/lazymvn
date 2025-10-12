# Demo Project for LazyMVN

This workspace hosts a minimal Spring Boot application split across two Maven modules so you can exercise LazyMVN against commands that resemble a real-world workflow.

## Layout
- `pom.xml` – aggregator POM inheriting from `spring-boot-starter-parent` and wiring modules.
- `library/` – reusable Spring bean (`GreetingService`) plus focused unit tests.
- `app/` – Boot entrypoint, REST controller, profile-aware configuration, and MVC tests.

## Useful Commands
Run these from the `demo-project/` directory:

- `mvn clean install` – build both modules and run their tests.
- `mvn -pl app spring-boot:run` – start the REST service; hit `http://localhost:8080/greet?name=Agent`.
- `mvn -pl app spring-boot:run -Pdev` – boot with the `dev` profile to see alternate greeting text.
- `mvn -pl library test` – execute only the shared library’s unit suite.

## Pairing with LazyMVN
1. Compile LazyMVN (`cargo build`) from the repository root if you haven’t already.
2. Launch the TUI from the repo root: `target/debug/lazymvn --project demo-project`.
3. Select the `app` module to drive Boot-specific targets (`package`, `spring-boot:run`, profiles, etc.).
4. Switch to the `library` module to run or watch focused unit tests.
