Feature: Publishing a complete Roblox project

  Scenario: Checking a project validates it without changing the workspace
    Given a project with a valid server module
    When I check the project manifest
    Then project validation succeeds
    And no Roblox output is created

  Scenario: A project tree maps modules to Roblox containers
    Given a project with a shared library and a server entrypoint
    When I compile the project
    And I compile it again after adding a file under the managed output
    Then the shared library appears in ReplicatedStorage
    And the server entrypoint appears in ServerScriptService
    And the stale managed file disappears

  Scenario: A manifest that omits the output location is explained
    Given a project manifest that omits where generated files should go
    When I check the project manifest
    Then validation names the missing output location

  Scenario: A broken update leaves the last good Roblox output intact
    Given a project with a valid server module
    When I compile the project
    And I remember the published server output
    And I replace the module with bytes that are not WebAssembly and compile again
    Then compilation explains that the module was rejected
    And the last good server output is unchanged
