--!strict
-- runtime/roblox.luau
--
-- The Roblox binding layer for luau-rs wasm modules.
--
-- A luau-rs module is instantiated with an imports table:
--     local module = require(path.to.module)
--     local runtime = require(path.to.roblox_runtime).new(roblox_globals)
--     local exports = module(runtime.imports)
--     runtime.bind_memory(exports.memory)
--
-- Rust code receives Roblox objects as integer handles (0 is null). The
-- runtime keeps a handle -> Instance registry; imported functions convert
-- handles back into real Instances before touching the Roblox API. String
-- arguments arrive as pointers into the module's linear memory and are
-- decoded by this runtime after bind_memory.
--
-- The covered property surface is deliberately small and explicit: Name and
-- ClassName are strings, Anchored is a number, Size is a Vector3, and
-- CanCollide is available for missing-property detection. An unknown property
-- is an error instead of an accidental dynamic capability.
--
-- The constructor takes the Roblox globals explicitly so the same module can
-- run against the real platform or against a mock environment in tests.

export type RobloxEnvironment = {
    game: any,
    workspace: any,
    Instance: {
        new: (className: string) -> any,
    },
    Vector3: {
        new: (x: number, y: number, z: number) -> any,
    },
    print: (...any) -> (),
}

export type RobloxRuntime = {
    imports: { [string]: { [string]: any } },
    bind_memory: (memory: buffer) -> (),
    bind_exports: (exports: { [string]: any }) -> (),
    handle_for: (instance: any) -> number,
    instance_at: (handle: number) -> any,
}

local RobloxRuntime = {}
RobloxRuntime.__index = RobloxRuntime

-- The first handle value; 0 is reserved for null.
local FIRST_HANDLE: number = 1
-- The null handle, used for optional instances that are absent.
local NULL_HANDLE: number = 0

function RobloxRuntime.new(env: RobloxEnvironment): RobloxRuntime
    local self = setmetatable({}, RobloxRuntime) :: RobloxRuntime
    self.env = env
    self.memory = nil
    self.exports = nil
    self.instances = {}
    self.next_handle = FIRST_HANDLE

    local imports = { roblox = {} }
    local roblox_imports: { [string]: any } = imports.roblox

    roblox_imports.roblox_print = function(message_ptr: number)
        env.print(self:_read_string(message_ptr))
    end

    roblox_imports.roblox_get_service = function(name_ptr: number): number
        local service_name = self:_read_string(name_ptr)
        local service = env.game:GetService(service_name)
        if service == nil then
            error("Roblox service capability is not covered: " .. service_name)
        end
        return self:handle_for(service)
    end

    roblox_imports.roblox_instance_new = function(class_ptr: number): number
        local class_name = self:_read_string(class_ptr)
        local instance = env.Instance.new(class_name)
        return self:handle_for(instance)
    end

    roblox_imports.roblox_set_parent = function(handle: number, parent_handle: number)
        local instance = self:_require_instance(handle)
        if parent_handle == NULL_HANDLE then
            instance.Parent = nil
            return
        end
        instance.Parent = self:_require_instance(parent_handle)
    end

    roblox_imports.roblox_set_property = function(handle: number, property_ptr: number, value: number)
        local instance = self:_require_instance(handle)
        local property_name = self:_read_string(property_ptr)
        self:_require_writable_property(property_name, "number")
        instance[property_name] = value
    end

    roblox_imports.roblox_get_property = function(handle: number, property_ptr: number): number
        local instance = self:_require_instance(handle)
        local property_name = self:_read_string(property_ptr)
        self:_require_supported_property(property_name)
        local value = instance[property_name]
        if typeof(value) ~= "number" then
            error("Roblox property is not a number: " .. property_name)
        end
        return value
    end

    roblox_imports.roblox_set_string_property = function(
        handle: number,
        property_ptr: number,
        value_ptr: number
    )
        local instance = self:_require_instance(handle)
        local property_name = self:_read_string(property_ptr)
        self:_require_writable_property(property_name, "string")
        instance[property_name] = self:_read_string(value_ptr)
    end

    roblox_imports.roblox_get_property_kind = function(handle: number, property_ptr: number): number
        local instance = self:_require_instance(handle)
        local property_name = self:_read_string(property_ptr)
        self:_require_supported_property(property_name)
        local value = instance[property_name]
        if value == nil then
            return 0
        elseif typeof(value) == "number" then
            return 1
        elseif typeof(value) == "string" then
            return 2
        elseif typeof(value) == "Vector3" then
            return 3
        end
        return 4
    end

    roblox_imports.roblox_get_string_property = function(
        handle: number,
        property_ptr: number,
        output_ptr: number,
        output_capacity: number
    ): number
        local instance = self:_require_instance(handle)
        local property_name = self:_read_string(property_ptr)
        self:_require_supported_property(property_name)
        local value = instance[property_name]
        if typeof(value) ~= "string" then
            error("Roblox property is not a string: " .. property_name)
        end
        if #value > output_capacity then
            return -1
        end
        for index = 1, #value do
            buffer.writeu8(self.memory, output_ptr + index - 1, string.byte(value, index))
        end
        return #value
    end

    roblox_imports.roblox_set_vector3 = function(
        handle: number,
        property_ptr: number,
        x: number,
        y: number,
        z: number
    )
        local instance = self:_require_instance(handle)
        local property_name = self:_read_string(property_ptr)
        self:_require_writable_property(property_name, "Vector3")
        instance[property_name] = env.Vector3.new(x, y, z)
    end

    roblox_imports.roblox_destroy = function(handle: number)
        local instance = self:_require_instance(handle)
        instance:Destroy()
        self.instances[handle] = nil
    end

    roblox_imports.roblox_connect = function(
        handle: number,
        event_ptr: number,
        callback_index: number
    )
        local instance = self:_require_instance(handle)
        local event_name = self:_read_string(event_ptr)
        local event = instance[event_name]
        if event == nil or event.Connect == nil then
            error("Roblox event capability is not covered: " .. event_name)
        end
        event:Connect(function(...)
            local callback = self.exports.functions[callback_index + 1]
            if callback == nil then
                error("Roblox callback export is missing: " .. tostring(callback_index))
            end
            return callback(...)
        end)
    end

    self.imports = imports
    return self
end

-- Decodes a null-terminated string from the module's linear memory.
function RobloxRuntime:_read_string(pointer: number): string
    local memory = self.memory
    if memory == nil then
        return ""
    end
    local characters = {}
    local index = 0
    while true do
        local byte = buffer.readu8(memory, pointer + index)
        if byte == 0 then
            break
        end
        characters[index + 1] = string.char(byte)
        index += 1
    end
    return table.concat(characters)
end

function RobloxRuntime:_require_instance(handle: number): any
    local instance = self:instance_at(handle)
    if instance == nil then
        error("Roblox instance handle is not live: " .. tostring(handle))
    end
    return instance
end

function RobloxRuntime:_require_supported_property(property_name: string)
    if property_name ~= "Name"
        and property_name ~= "ClassName"
        and property_name ~= "Anchored"
        and property_name ~= "Size"
        and property_name ~= "CanCollide"
    then
        error("Roblox property capability is not covered: " .. property_name)
    end
end

function RobloxRuntime:_require_writable_property(property_name: string, expected_type: string)
    if property_name == "Anchored" and expected_type == "number" then
        return
    elseif property_name == "Name" and expected_type == "string" then
        return
    elseif property_name == "Size" and expected_type == "Vector3" then
        return
    end
    error("Roblox writable property capability is not covered: " .. property_name)
end

-- Resolves a handle to a live instance, or nil when the handle is absent.
function RobloxRuntime:instance_at(handle: number): any
    return self.instances[handle]
end

-- Returns the handle for an instance, registering it on first use.
function RobloxRuntime:handle_for(instance: any): number
    if instance == nil then
        return NULL_HANDLE
    end
    local handle = self.next_handle
    self.next_handle += 1
    self.instances[handle] = instance
    return handle
end

-- Binds the module's linear memory so string decoding works.
function RobloxRuntime:bind_memory(memory: buffer)
    self.memory = memory
end

-- Binds the module's exported surface so imports can invoke Rust callbacks.
function RobloxRuntime:bind_exports(exports: { [string]: any })
    self.exports = exports
end

return RobloxRuntime
