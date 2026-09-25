#include "expected.h"
#include "input.h"

// @abigen:start

#ifdef __cplusplus
extern "C" {
#endif

XYMyClass* x_y_myclass_new()
{
    XYMyClass* out = (XYMyClass*) new x::y::MyClass();
    return out;
}

Result x_y_myclass_destroy(XYMyClass** self_ptr)
{
    if (self_ptr == nullptr) { return Result::ArgumentNull; }
    if (*self_ptr == nullptr) { return Result::ArgumentNull; }
    x::y::MyClass* self = *(x::y::MyClass**) self_ptr;
    
    try {
        delete self;
    } catch (...) {
        return Result::Unknown;
    }

    *self_ptr = nullptr;
    
    return Result::Success;
}

Result x_y_myclass_config(const XYMyClass* self_ptr, XYZConfig** out)
{
    if (self_ptr == nullptr) {
        return Result::ArgumentNull;
    }
    if (out == nullptr) {
        return Result::ArgumentNull;
    }
    const x::y::MyClass& self = *(const x::y::MyClass*)self_ptr;
    *out = nullptr;
    try {
        *out = (XYZConfig*)&self.config();
    }
    catch (...) {
        return Result::Unknown;
    }
    return Result::Success;
}

Result x_y_myclass_set_config(XYMyClass* self_ptr, const XYZConfig* config)
{
    if (self_ptr == nullptr) {
        return Result::ArgumentNull;
    }
    if (config == nullptr) {
        return Result::ArgumentNull;
    }
    x::y::MyClass& self = *(x::y::MyClass*)self_ptr;
    try {
        self.set_config(*(const x::y::z::Config*)config);
    }
    catch (...) {
        return Result::Unknown;
    }
    
    return Result::Success;
}

Result x_y_myclass_subclass_do_something(XYMyClassSubClass* self_ptr, int flag, int* out)
{
    if (self_ptr == nullptr) {
        return Result::ArgumentNull;
    }
    if (out == nullptr) {
        return Result::ArgumentNull;
    }
    x::y::MyClass::SubClass& self = *(x::y::MyClass::SubClass*)self_ptr;
    *out = int{};
    try {
        *out = (int)self.do_something((bool)flag);
    }
    catch (...) {
        return Result::Unknown;
    }
    return Result::Success;
}

#ifdef __cplusplus
} // extern "C"
#endif

// @abigen:end