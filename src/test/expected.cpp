#include "expected.h"

// @abigen:start

#ifdef __cplusplus
extern "C" {
#endif

Result x_y_myclass_subclass_do_something(XYMyClassSubClass* self, int* out)
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

Result x_y_myclass_config(const XYMyClass* self, XYZConfig** out)
{
    if (self == nullptr) {
        return Result::ArgumentNull;
    }
    if (out == nullptr) {
        return Result::ArgumentNull;
    }
    *out = XYZConfig*{};
    try {
        *out = self->config();
    }
    catch (...) {
        return Result::Unknown;
    }
    return Result::Success;
}

Result x_y_myclass_set_config(XYMyClass* self, const XYZConfig* config)
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