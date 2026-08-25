Feature: Integrating generated modules with Roblox bindings

  Scenario: The binding driver passes Luau analysis
    Given the translated fixture and Roblox runtime
    When I ask official Luau analysis to validate the binding driver
    Then the analyzer accepts the binding driver

  Scenario: The binding driver runs against the Roblox mock
    Given the translated fixture and Roblox runtime
    When I run the binding driver with official Luau
    Then the binding driver completes successfully
