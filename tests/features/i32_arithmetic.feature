Feature: Matching WebAssembly integer behavior in Luau

  Scenario: Overflowing arithmetic keeps i32 wraparound
    Given a module whose arithmetic operations overflow i32
    When I evaluate it with official Luau
    Then Luau returns the WebAssembly-wrapped results
