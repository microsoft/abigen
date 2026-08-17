/**
 * Header goes here
 */

#include <memory>

namespace ns {
    class MyClass {
    public:
        MyClass(int arg1);
        ~MyClass();
        
        const void* pub_method(int arg1, float arg2);

    private:
        int priv_method(char* arg1, uint8_t* arg2);
    };
}