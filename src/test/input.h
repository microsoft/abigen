#pragma once

struct Config;

class MyClass {
public:
    class SubClass {
    public:
        int do_something();
    };
    
    const Config& config() const noexcept;
    void set_config(const Config& config);
};