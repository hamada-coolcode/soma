#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

int parseAndSum(const std::vector<std::string>& values) {
    int total = 0;
    for (const auto& v : values) {
        try {
            total += std::stoi(v);
        } catch (...) {
            throw std::runtime_error("bad int: " + v);
        }
    }
    return total;
}

int safeDiv(int a, int b) {
    if (b == 0) throw std::runtime_error("division by zero");
    return a / b;
}

int main() {
    std::vector<std::string> good = {"10", "20", "30"};
    std::vector<std::string> bad = {"10", "oops", "30"};

    try {
        int total = parseAndSum(good);
        std::cout << "sum = " << total << "\n";
        int q = safeDiv(total, 3);
        std::cout << "sum / 3 = " << q << "\n";
    } catch (const std::exception& e) {
        std::cout << "error: " << e.what() << "\n";
    }

    try {
        int q0 = safeDiv(60, 0);
        std::cout << "sum / 0 = " << q0 << "\n";
    } catch (const std::exception& e) {
        std::cout << "error: " << e.what() << "\n";
    }

    try {
        parseAndSum(bad);
    } catch (const std::exception& e) {
        std::cout << "error: " << e.what() << "\n";
    }
}
