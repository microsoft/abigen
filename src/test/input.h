#pragma once

namespace x::y
{
    namespace z
    {
        struct Config
        {
            float a;
            float b;
            float c;
        };
    }

    class MyClass {
    public:
        class SubClass {
        public:
            bool do_something(bool flag);
        };

    public:
        MyClass() noexcept;

        const z::Config& config() const noexcept;
        void set_config(const z::Config& config);
    };
}