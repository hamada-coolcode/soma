#include <algorithm>
#include <iostream>
#include <numeric>
#include <vector>

int apply(int x, int f(int)) { return f(x); }

int main() {
    std::vector<int> numbers = {5, 2, 9, 1, 7, 3};

    std::sort(numbers.begin(), numbers.end());
    std::cout << "sorted asc:  ";
    for (int x : numbers) std::cout << x << " ";
    std::cout << "\n";

    std::sort(numbers.begin(), numbers.end(), [](int a, int b) { return a > b; });
    std::cout << "sorted desc: ";
    for (int x : numbers) std::cout << x << " ";
    std::cout << "\n";

    // map
    std::vector<int> doubled;
    std::transform(numbers.begin(), numbers.end(), std::back_inserter(doubled),
                   [](int x) { return x * 2; });

    // filter
    std::vector<int> evens;
    std::copy_if(numbers.begin(), numbers.end(), std::back_inserter(evens),
                 [](int x) { return x % 2 == 0; });

    // reduce
    int sum = std::accumulate(numbers.begin(), numbers.end(), 0);

    std::cout << "doubled: ";
    for (int x : doubled) std::cout << x << " ";
    std::cout << "\n";
    std::cout << "evens:   ";
    for (int x : evens) std::cout << x << " ";
    std::cout << "\n";
    std::cout << "sum:     " << sum << "\n";

    // lambda capturing environment
    int factor = 3;
    auto scale = [factor](int x) { return x * factor; };
    std::cout << "scale(5): " << scale(5) << "\n";

    std::cout << "apply(5, double): " << apply(5, [](int x) { return x * 2; }) << "\n";
}
