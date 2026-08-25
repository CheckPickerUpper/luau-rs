Feature: Decoding WebAssembly modules

  Scenario: Malformed bytes are rejected
    Given bytes that are not a WebAssembly module
    When I decode the WebAssembly bytes
    Then decoding reports a malformed module

  Scenario: Imported memory is rejected
    Given a WebAssembly module that imports memory
    When I decode the WebAssembly bytes
    Then decoding reports an unsupported memory import

  Scenario: An empty module decodes to an empty surface
    Given an empty WebAssembly module
    When I decode the WebAssembly bytes
    Then decoding returns no functions and no exports
