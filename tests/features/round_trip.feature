Feature: Keeping Rust module behavior after translation to Luau

  Scenario: Rust exports survive translation into Luau
    Given the compiled Rust hello module
    When I translate it to Luau
    Then callers can use add, fib, and double_at
    And the translated module owns linear memory

  Scenario: The generated module stays stable for review
    Given the compiled Rust hello module
    When I translate it to Luau
    Then the generated Luau matches the committed output snapshot

  Scenario: The translated module is accepted by Luau analysis
    Given the compiled Rust hello module
    When I translate it to Luau
    And Luau analysis checks the translated module
    Then the generated module passes analysis without errors

  Scenario: The translated module keeps its exported behavior
    Given the compiled Rust hello module
    When I translate it to Luau
    And I run the translated module with official Luau
    Then add returns 42, fib returns 34, and memory doubles 7 to 14
