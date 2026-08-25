Feature: Preserving WebAssembly integer arithmetic

  Scenario: Overflowing i32 operations wrap like WebAssembly
    Given a module with overflowing i32 arithmetic exports
    When I run the translated arithmetic module with official Luau
    Then official Luau reports every wrapping result as correct
