#include <cmath>
#include <iostream>
#include <memory>
#include <vector>

class Shape {
public:
    virtual double area() const = 0;
    virtual const char* name() const = 0;
    virtual ~Shape() = default;
};

class Circle : public Shape {
    double radius;

public:
    explicit Circle(double r) : radius(r) {}
    double area() const override { return M_PI * radius * radius; }
    const char* name() const override { return "circle"; }
};

class Rectangle : public Shape {
    double width, height;

public:
    Rectangle(double w, double h) : width(w), height(h) {}
    double area() const override { return width * height; }
    const char* name() const override { return "rectangle"; }
};

int main() {
    std::vector<std::unique_ptr<Shape>> shapes;
    shapes.push_back(std::make_unique<Circle>(2.0));
    shapes.push_back(std::make_unique<Rectangle>(3.0, 4.0));

    for (const auto& s : shapes) {
        std::cout << s->name() << " area = " << s->area() << "\n";
    }
}
