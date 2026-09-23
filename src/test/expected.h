#ifndef OUTPUT_TEMPLATE_H
#define OUTPUT_TEMPLATE_H

enum struct Result
{
    Success = 0,
    Unknown = 1,
    ArgumentNull = 2
};

// @todo Remove this placeholder once it's automatically generated
typedef struct XYZConfig XYZConfig;

/** Generated header - DO NOT MODIFY
 * @abigen:start
 */

#ifdef __cplusplus
extern "C" {
#endif

typedef struct XYMyClass XYMyClass;
typedef struct XYMyClassSubClass XYMyClassSubClass;

XYMyClass* x_y_myclass_new();
Result x_y_myclass_destroy(XYMyClass** self_ptr);
Result x_y_myclass_config(const XYMyClass* self_ptr, XYZConfig** out);
Result x_y_myclass_set_config(XYMyClass* self_ptr, const XYZConfig* config);
Result x_y_myclass_subclass_do_something(XYMyClassSubClass* self_ptr, int flag, int* out);

#ifdef __cplusplus
} // extern "C"
#endif

// @abigen:end

#endif