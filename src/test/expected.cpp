#include "expected.h"

// @abigen:start

#ifdef __cplusplus
extern "C" {
#endif

Result x_y_myclass_subclass_do_something(x_y_MyClass_SubClass* self, int* out)
{
    if (self == nullptr) {
        return Result::ArgumentNull;
    }
    if (out == nullptr) {
        return Result::ArgumentNull;
    }
    *out = int{};
    try {
        *out = self->do_something();
    }
    catch (...) {
        return Result::Unknown;
    }
    return Result::Success;
}

Result x_y_myclass_config(const x_y_MyClass* self, z::Config** out)
{
    if (self == nullptr) {
        return Result::ArgumentNull;
    }
    if (out == nullptr) {
        return Result::ArgumentNull;
    }
    *out = z::Config*{};
    try {
        *out = self->config();
    }
    catch (...) {
        return Result::Unknown;
    }
    return Result::Success;
}

Result x_y_myclass_set_config(x_y_MyClass* self, const z::Config* config)
{
    if (self == nullptr) {
        return Result::ArgumentNull;
    }
    if (config == nullptr) {
        return Result::ArgumentNull;
    }

    try {
        self->set_config(*config);
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