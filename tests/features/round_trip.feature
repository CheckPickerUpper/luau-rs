Feature: Translating Rust modules into executable Luau

  Scenario: Generated Luau exposes the fixture surface
    Given the committed Rust hello WebAssembly module
    When I translate it to Luau
    Then the generated Luau exposes the add fib and double_at functions
    And the generated Luau declares linear memory

  Scenario: Generated Luau remains byte-for-byte stable
    Given the committed Rust hello WebAssembly module
    When I translate it to Luau
    Then the generated Luau matches the committed snapshot

  Scenario: Generated Luau passes official analysis
    Given the committed Rust hello WebAssembly module
    When I translate it to Luau
    And I ask official Luau analysis to validate it
    Then the analyzer accepts the generated module

  Scenario: Generated Luau executes its exported behavior
    Given the committed Rust hello WebAssembly module
    When I translate it to Luau
    And I run the generated module with official Luau
    Then official Luau reports the expected exported results
