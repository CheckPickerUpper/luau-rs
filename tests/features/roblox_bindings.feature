Feature: Letting generated Rust modules use Roblox objects

  Scenario: Generated code is valid Roblox Luau
    Given a generated Rust module with its Roblox runtime and a test world
    When Luau analysis checks the module and runtime together
    Then the combined program passes analysis without errors

  Scenario: A Rust module creates a Part and handles a click
    Given a generated Rust module with its Roblox runtime and a test world
    When the module creates a Part at position (1, 2, 3) and handles click 21
    Then the Roblox test world contains an anchored Part of size (1, 2, 3), and the module reports click 42 with add 42 and fib 34
