Feature: Managing manifest-backed projects

  Scenario: Checking a valid project does not create output
    Given a valid project with one server module
    When I check the manifest project
    Then checking succeeds
    And no build output is created

  Scenario: Compiling nested modules publishes Roblox paths and removes stale files
    Given a valid project with shared and server modules
    When I compile the manifest project
    And I add a stale managed file and compile the manifest project again
    Then the shared module is published under ReplicatedStorage
    And the server entrypoint is published under ServerScriptService
    And the stale managed file is removed

  Scenario: A missing output root is reported
    Given a project manifest without an output root
    When I check the manifest project
    Then checking fails because output_root is missing

  Scenario: A failed recompilation preserves the last good output
    Given a valid project with one server module
    When I compile the manifest project
    And I remember the published server output
    And I replace the server module with invalid bytes and compile the manifest project again
    Then compilation fails because the module was rejected
    And the remembered server output is unchanged
