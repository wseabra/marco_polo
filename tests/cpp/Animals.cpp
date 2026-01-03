class Animal {
public:
    std::string name;

    Animal(std::string name) : name(name) {}

    void speak() {
        // Animal speaks
    }
};

class Dog : public Animal {
public:
    Dog(std::string name) : Animal(name) {}

    void speak() {
        // Dog barks
    }
};

class Cat : public Animal {
public:
    Cat(std::string name) : Animal(name) {}

    void speak() {
        // Cat meows
    }
};
