#include "expected.h"

// @abigen:start

#ifdef __cplusplus
extern "C" {
#endif

Result myclass_config(const MyClass* self, Config** out)
{
    if (self == nullptr) {
        return Result::ArgumentNull;
    }
    if (out == nullptr) {
        return Result::ArgumentNull;
    }
    *out = nullptr;
    try {
        *out = self->config();
    }
    catch (...) {
        return Result::Unknown;
    }
    return Result::Success;
}

Result myclass_set_config(MyClass* self, const Config* config)
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
} // extern C
#endif

// @abigen:end