Feature: Deciding which WebAssembly modules can become Luau

  Scenario: Random bytes never become a Luau module
    Given input bytes that are not a WebAssembly module
    When I ask the compiler to read the WebAssembly bytes
    Then it identifies a malformed module

  Scenario: A module asking the host for memory is refused
    Given a WebAssembly module that asks the host to provide memory
    When I ask the compiler to read the WebAssembly bytes
    Then it explains that imported memory is unsupported

  Scenario: A module with no code has a valid empty public surface
    Given an empty WebAssembly module
    When I ask the compiler to read the WebAssembly bytes
    Then it returns no functions and no exports
