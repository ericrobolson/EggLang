#include <iostream>
#include <vector>

class A
{
public:
    int value;
    A()
    {
        value = 1;
        std::cout << "constructor" << std::endl;
    }

    A(const A &other) : A()
    {
        std::cout << value << std::endl;
        std::cout << "copy constructor" << std::endl;
        value = other.value;
        std::cout << value << std::endl;
    }
};

int main()
{

    A a;
    a.value = 2;
    A b(a);
    return 0;
}