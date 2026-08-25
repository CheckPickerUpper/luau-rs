Feature: Building WebAssembly modules for Roblox

  Scenario: A server entrypoint is written from a valid module
    Given the committed Rust hello WebAssembly module
    When I build it as a server entrypoint
    Then the server script is written under ServerScriptService
    And the server script uses strict Luau
    And the server script exposes an instantiate factory

  Scenario: A non-WebAssembly file is rejected
    Given a file containing invalid WebAssembly bytes
    When I build it as a server entrypoint
    Then the build fails because the input was rejected

  Scenario: A client module uses its Roblox client path
    Given the committed Rust hello WebAssembly module
    When I build it as a client module at "game/main"
    Then the client script is written under StarterPlayerScripts
