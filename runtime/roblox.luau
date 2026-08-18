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
    self.instances = {}
    self.next_handle = FIRST_HANDLE

    local imports = { roblox = {} }
    local roblox_imports: { [string]: any } = imports.roblox

    roblox_imports.roblox_print = function(message_ptr: number)
        env.print(self:_read_string(message_ptr))
    end

    roblox_imports.roblox_get_service = function(name_ptr: number): number
        local service_name = self:_read_string(name_ptr)
        return self:handle_for(env.game:GetService(service_name))
    end

    roblox_imports.roblox_instance_new = function(class_ptr: number, parent_handle: number): number
        local class_name = self:_read_string(class_ptr)
        local instance = env.Instance.new(class_name)
        if parent_handle ~= NULL_HANDLE then
            local parent = self:instance_at(parent_handle)
            if parent ~= nil then
                instance.Parent = parent
            end
        end
        return self:handle_for(instance)
    end

    roblox_imports.roblox_set_property = function(handle: number, property_ptr: number, value: number)
        local instance = self:instance_at(handle)
        local property_name = self:_read_string(property_ptr)
        if instance ~= nil then
            instance[property_name] = value
        end
    end

    roblox_imports.roblox_get_property = function(handle: number, property_ptr: number): number
        local instance = self:instance_at(handle)
        local property_name = self:_read_string(property_ptr)
        if instance == nil then
            return 0
        end
        return instance[property_name] or 0
    end

    roblox_imports.roblox_set_vector3 = function(
        handle: number,
        property_ptr: number,
        x: number,
        y: number,
        z: number
    )
        local instance = self:instance_at(handle)
        local property_name = self:_read_string(property_ptr)
        if instance ~= nil then
            instance[property_name] = env.Vector3.new(x, y, z)
        end
    end

    roblox_imports.roblox_destroy = function(handle: number)
        local instance = self:instance_at(handle)
        if instance ~= nil then
            instance:Destroy()
        end
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

return RobloxRuntime
