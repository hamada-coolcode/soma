// compile with: g++ -std=c++17 -pthread main.cpp -o main
#include <iostream>
#include <numeric>
#include <thread>
#include <vector>

long long worker(int id) {
    long long sum = 0;
    for (long long j = 0; j < 1000000; j++) sum += j;
    std::cout << "worker " << id << " sum = " << sum << "\n";
    return sum;
}

int main() {
    std::vector<std::thread> threads;
    std::vector<long long> results(4);

    for (int i = 0; i < 4; i++) {
        threads.emplace_back([i, &results] { results[i] = worker(i); });
    }
    for (auto& t : threads) t.join();

    long long total = std::accumulate(results.begin(), results.end(), 0LL);
    std::cout << "total = " << total << "\n";
}
