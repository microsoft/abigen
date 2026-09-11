#include "expected.h"

// @abigen:start

#ifdef __cplusplus
extern "C" {
#endif

Result myclass_config(const MyClass* self, Config** out)
{
    return Result::Success;
}

Result myclass_set_config(MyClass* self, const Config* config)
{
    return Result::Success;
}

#ifdef __cplusplus
} // extern C
#endif

// @abigen:end