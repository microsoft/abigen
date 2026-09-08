#ifndef OUTPUT_TEMPLATE_H
#define OUTPUT_TEMPLATE_H

struct MyClass;
struct Config;
enum struct Result
{
    Success = 0,
    Unknown = 1,
    ArgumentNull = 2
};

// @abigen:start

#ifdef __cplusplus
extern "C" {
#endif

Result myclass_config(const MyClass* self, Config** out);

#ifdef __cplusplus
} // extern C
#endif

// @abigen:end

#endif