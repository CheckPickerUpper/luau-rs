Feature: Delivering Rust modules into Roblox

  Scenario: A server module becomes a strict Roblox script
    Given a valid Rust module compiled to WebAssembly
    When I build it as a server entrypoint
    Then Roblox receives a server script under ServerScriptService
    And the server script is strict Luau
    And callers can instantiate the generated module

  Scenario: A corrupt input does not produce a Roblox script
    Given a file that claims to be WebAssembly but contains invalid bytes
    When I build it as a server entrypoint
    Then the build explains that the module was rejected

  Scenario: A client module lands in its Roblox client container
    Given a valid Rust module compiled to WebAssembly
    When I build it as a client module at "game/main"
    Then Roblox receives the client script at StarterPlayerScripts/game/main
