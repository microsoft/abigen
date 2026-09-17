#ifndef OUTPUT_TEMPLATE_H
#define OUTPUT_TEMPLATE_H

enum struct Result
{
    Success = 0,
    Unknown = 1,
    ArgumentNull = 2
};

/** Generated header - DO NOT MODIFY
 * @abigen:start
 */

#ifdef __cplusplus
extern "C" {
#endif

typedef struct XYMyClass XYMyClass;
typedef struct XYMyClassSubClass XYMyClassSubClass;

Result x_y_myclass_subclass_do_something(XYMyClassSubClass* self, int* out);
Result x_y_myclass_config(const XYMyClass* self, z::Config** out);
Result x_y_myclass_set_config(XYMyClass* self, const z::Config* config);

#ifdef __cplusplus
} // extern "C"
#endif

// @abigen:end

#endif