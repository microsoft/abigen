#pragma once

struct Config;

class MyClass {
public:
    const Config& config() const noexcept;
    void set_config(const Config& config);
};