Feature: Manifest-backed project compilation

  Scenario: Compile nested modules into Roblox paths
    Given a manifest project with nested wasm modules
    When I compile the manifest project
    Then the shared module is published under ReplicatedStorage
    And the server entrypoint is published under ServerScriptService
